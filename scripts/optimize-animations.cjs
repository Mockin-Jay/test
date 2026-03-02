const sharp = require('sharp');
const fs = require('fs/promises');
const path = require('path');

const ASSETS_DIR = path.join('C:/Users/derec/Desktop/mini-ko assets');

async function analyzeFrame(imagePath) {
  const image = sharp(imagePath);
  const metadata = await image.metadata();

  // Trim transparent edges to find content bounds
  const trimmed = await image
    .trim({ threshold: 10 })
    .toBuffer({ resolveWithObject: true });

  // trimOffsetLeft/Top are negative (pixels removed from left/top edge)
  const left = Math.abs(trimmed.info.trimOffsetLeft || 0);
  const top = Math.abs(trimmed.info.trimOffsetTop || 0);

  return {
    original: { width: metadata.width, height: metadata.height },
    bounds: {
      left,
      top,
      right: left + trimmed.info.width,
      bottom: top + trimmed.info.height,
      width: trimmed.info.width,
      height: trimmed.info.height
    }
  };
}

async function processAnimationFolder(folderPath) {
  const files = await fs.readdir(folderPath);
  const pngFiles = files.filter(f => f.endsWith('.png')).sort();

  if (pngFiles.length === 0) return null;

  // Analyze ALL frames to find the union bounding box
  // (different frames may have content in different positions)
  console.log(`  Analyzing ${pngFiles.length} frames for union bounds...`);

  let unionLeft = Infinity, unionTop = Infinity;
  let unionRight = 0, unionBottom = 0;
  let originalWidth = 0, originalHeight = 0;

  const ANALYZE_BATCH = 10;
  for (let i = 0; i < pngFiles.length; i += ANALYZE_BATCH) {
    const batch = pngFiles.slice(i, i + ANALYZE_BATCH);
    const results = await Promise.all(
      batch.map(file => analyzeFrame(path.join(folderPath, file)))
    );

    for (const result of results) {
      originalWidth = result.original.width;
      originalHeight = result.original.height;
      unionLeft = Math.min(unionLeft, result.bounds.left);
      unionTop = Math.min(unionTop, result.bounds.top);
      unionRight = Math.max(unionRight, result.bounds.right);
      unionBottom = Math.max(unionBottom, result.bounds.bottom);
    }
  }

  // Clamp to image dimensions
  unionLeft = Math.max(0, unionLeft);
  unionTop = Math.max(0, unionTop);
  unionRight = Math.min(originalWidth, unionRight);
  unionBottom = Math.min(originalHeight, unionBottom);

  const unionWidth = unionRight - unionLeft;
  const unionHeight = unionBottom - unionTop;

  const analysis = {
    original: { width: originalWidth, height: originalHeight },
    bounds: {
      left: unionLeft,
      top: unionTop,
      width: unionWidth,
      height: unionHeight
    }
  };

  // Skip if no significant cropping possible (less than 10% savings)
  const originalPixels = originalWidth * originalHeight;
  const croppedPixels = unionWidth * unionHeight;
  if (croppedPixels > originalPixels * 0.9) {
    console.log(`  Skipping (minimal savings: ${Math.round(croppedPixels / originalPixels * 100)}% of original)`);
    return null;
  }

  // Create output directory
  const outputDir = path.join(folderPath, 'optimized');
  await fs.mkdir(outputDir, { recursive: true });

  // Process all frames with the union crop bounds
  const BATCH_SIZE = 10;
  for (let i = 0; i < pngFiles.length; i += BATCH_SIZE) {
    const batch = pngFiles.slice(i, i + BATCH_SIZE);
    await Promise.all(batch.map(async (file) => {
      const inputPath = path.join(folderPath, file);
      const outputPath = path.join(outputDir, file);

      await sharp(inputPath)
        .extract({
          left: analysis.bounds.left,
          top: analysis.bounds.top,
          width: analysis.bounds.width,
          height: analysis.bounds.height
        })
        .toFile(outputPath);
    }));

    const processed = Math.min(i + BATCH_SIZE, pngFiles.length);
    process.stdout.write(`\r  Cropping: ${processed}/${pngFiles.length} frames`);
  }
  console.log('');

  return {
    path: folderPath,
    frameCount: pngFiles.length,
    ...analysis
  };
}

async function findAnimationFolders(baseDir) {
  const folders = [];

  async function scan(dir) {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    const pngFiles = entries.filter(e => e.isFile() && e.name.endsWith('.png'));

    // Only include folders with animation frames (prefix_00000.png pattern),
    // not png-stacks (1.png, 2.png, etc.)
    const animFrames = pngFiles.filter(e => /^.+_\d{5,}\.png$/i.test(e.name));
    if (animFrames.length > 5) {
      folders.push(dir);
    }

    for (const entry of entries) {
      if (entry.isDirectory() && entry.name !== 'optimized') {
        await scan(path.join(dir, entry.name));
      }
    }
  }

  await scan(baseDir);
  return folders;
}

async function main() {
  console.log('Scanning for animation folders...');
  const animFolders = await findAnimationFolders(ASSETS_DIR);
  console.log(`Found ${animFolders.length} animation folders\n`);

  const metadata = {
    processedAt: new Date().toISOString(),
    animations: []
  };

  for (const folder of animFolders) {
    const relPath = path.relative(ASSETS_DIR, folder).replace(/\\/g, '/');
    console.log(`Processing: ${relPath}`);
    const result = await processAnimationFolder(folder);

    if (result) {
      metadata.animations.push({
        path: path.relative(ASSETS_DIR, result.path).replace(/\\/g, '/'),
        originalWidth: result.original.width,
        originalHeight: result.original.height,
        croppedWidth: result.bounds.width,
        croppedHeight: result.bounds.height,
        offsetX: result.bounds.left,
        offsetY: result.bounds.top,
        frameCount: result.frameCount
      });

      const savings = Math.round((1 - (result.bounds.width * result.bounds.height) / (result.original.width * result.original.height)) * 100);
      console.log(`  ${result.frameCount} frames: ${result.original.width}x${result.original.height} -> ${result.bounds.width}x${result.bounds.height} (${savings}% smaller)\n`);
    }
  }

  // Write metadata file
  const metadataPath = path.join(ASSETS_DIR, 'animation-bounds.json');
  await fs.writeFile(metadataPath, JSON.stringify(metadata, null, 2));

  console.log(`\nComplete! Processed ${metadata.animations.length} animations`);
  console.log(`Metadata saved to: ${metadataPath}`);
}

main().catch(console.error);
