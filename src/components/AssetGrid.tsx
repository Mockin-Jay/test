import { useEffect, useState } from "react";
import { useMinikoStore } from "../store/minikoStore";
import { LAYER_DEFS } from "../utils/assetUtils";
import { loadManifest, type Manifest, type AssetDef } from "../engine/manifest";
import { svgToDataUrl, recolorSVG } from "../engine/SVGRecolorer";

interface AssetGridProps {
  selectedCategory: string;
}

interface AssetWithLayer extends AssetDef {
  layerNum: number;
  oneItemLimit: boolean;
}

export default function AssetGrid({ selectedCategory }: AssetGridProps) {
  const [manifest, setManifest] = useState<Manifest | null>(null);
  const [filteredAssets, setFilteredAssets] = useState<AssetWithLayer[]>([]);
  const [thumbnails, setThumbnails] = useState<Record<string, string>>({});
  const [failedThumbnails, setFailedThumbnails] = useState<Set<string>>(new Set());

  const equippedAssets = useMinikoStore((s) => s.equippedAssets);
  const equipAsset = useMinikoStore((s) => s.equipAsset);
  const deselect = useMinikoStore((s) => s.deselect);
  const clearLayer = useMinikoStore((s) => s.clearLayer);
  const setSelectedAssetForColor = useMinikoStore((s) => s.setSelectedAssetForColor);

  // Load manifest
  useEffect(() => {
    loadManifest().then(setManifest);
  }, []);

  // Filter assets by selected category
  useEffect(() => {
    if (!manifest) {
      setFilteredAssets([]);
      return;
    }

    const layersInCategory = LAYER_DEFS.filter(
      (layer) => layer.category === selectedCategory
    );

    const assets: AssetWithLayer[] = [];
    const seenIds = new Set<string>();

    for (const layerDef of layersInCategory) {
      const layer = manifest.layers.find((l) => l.layerNum === layerDef.num);
      if (layer) {
        for (const asset of layer.assets) {
          // Prevent duplicates
          if (!seenIds.has(asset.id)) {
            seenIds.add(asset.id);
            assets.push({
              ...asset,
              layerNum: layer.layerNum,
              oneItemLimit: layerDef.oneItemLimit,
            });
          }
        }
      }
    }

    setFilteredAssets(assets);
  }, [manifest, selectedCategory]);

  // Generate thumbnails for visible assets
  useEffect(() => {
    if (filteredAssets.length === 0) {
      setThumbnails({});
      setFailedThumbnails(new Set());
      return;
    }

    const loadThumbnails = async () => {
      console.log(`[AssetGrid] Loading thumbnails for ${filteredAssets.length} assets`);
      const newThumbnails: Record<string, string> = {};
      const newFailed = new Set<string>();

      // Load all thumbnails in parallel
      const loadPromises = filteredAssets.map(async (asset) => {
        // Skip if already loaded
        if (thumbnails[asset.id]) {
          newThumbnails[asset.id] = thumbnails[asset.id];
          console.log(`[AssetGrid] Skipping ${asset.id} - already loaded`);
          return;
        }

        try {
          let thumbnailUrl = "";

          if (asset.type === "svg") {
            // Load SVG and convert to data URL (path already includes .svg)
            const pathSegments = asset.path.split("/").map(encodeURIComponent);
            const svgPath = `/assets/${pathSegments.join("/")}`;
            const response = await fetch(svgPath);

            if (!response.ok) {
              throw new Error(`HTTP ${response.status}: ${svgPath}`);
            }

            const svgText = await response.text();

            // Use default colors (no recoloring for thumbnail)
            const recolored = recolorSVG(svgText, undefined);
            thumbnailUrl = svgToDataUrl(recolored);
            console.log(`[AssetGrid] Loaded SVG thumbnail for ${asset.id}`);
          } else if (asset.type === "png-stack") {
            // Load first PNG from stack - just use the URL directly
            const pathSegments = asset.path.split("/").map(encodeURIComponent);
            thumbnailUrl = `/assets/${pathSegments.join("/")}/1.png`;
            console.log(`[AssetGrid] Set PNG-stack thumbnail for ${asset.id}: ${thumbnailUrl}`);
          } else if (asset.type === "png-animation") {
            // Load first frame from first sublayer
            if (asset.subLayers && asset.subLayers.length > 0) {
              const firstSub = asset.subLayers[0];
              const pathSegments = asset.path.split("/").map(encodeURIComponent);
              const subName = encodeURIComponent(firstSub.name);
              thumbnailUrl = `/assets/${pathSegments.join("/")}/${subName}/${firstSub.prefix}_00000.png`;
              console.log(`[AssetGrid] Set PNG-animation thumbnail for ${asset.id}: ${thumbnailUrl}`);
            }
          }

          if (thumbnailUrl) {
            newThumbnails[asset.id] = thumbnailUrl;
          } else {
            console.warn(`[AssetGrid] No thumbnail URL for ${asset.id}`);
            newFailed.add(asset.id);
          }
        } catch (error) {
          console.error(`Failed to load thumbnail for ${asset.id} (${asset.path}):`, error);
          newFailed.add(asset.id);
        }
      });

      // Wait for all thumbnails to load in parallel
      await Promise.all(loadPromises);

      console.log(`[AssetGrid] Finished loading. Success: ${Object.keys(newThumbnails).length}, Failed: ${newFailed.size}`);
      setThumbnails((prev) => ({ ...prev, ...newThumbnails }));
      setFailedThumbnails((prev) => new Set([...prev, ...newFailed]));
    };

    loadThumbnails();
  }, [filteredAssets]);

  const handleAssetClick = (asset: AssetWithLayer) => {
    const isEquipped = equippedAssets[asset.layerNum]?.includes(asset.id);

    if (isEquipped) {
      // Deselect asset
      deselect(asset.layerNum, asset.id);
      setSelectedAssetForColor(null);
    } else {
      // Equip asset
      if (asset.oneItemLimit) {
        // Replace existing asset on this layer
        equipAsset(asset.layerNum, asset.id);
      } else {
        // Add to existing assets on this layer
        equipAsset(asset.layerNum, asset.id);
      }
      setSelectedAssetForColor(asset.id);
    }
  };

  const handleNoneClick = () => {
    // Clear all layers in this category
    const layersInCategory = LAYER_DEFS.filter(
      (layer) => layer.category === selectedCategory
    );
    for (const layerDef of layersInCategory) {
      clearLayer(layerDef.num);
    }
    setSelectedAssetForColor(null);
  };

  if (!manifest) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--color-text-muted)]">
        Loading...
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto px-2 py-2">
      <div className="grid grid-cols-4 gap-2">
        {/* None option */}
        <button
          onClick={handleNoneClick}
          className="aspect-square rounded-lg border-2 border-dashed border-[var(--color-warm-border)]
                     flex flex-col items-center justify-center gap-1 cursor-pointer
                     hover:border-[var(--color-accent)] hover:bg-[var(--color-warm-card)]
                     transition-colors"
        >
          <svg
            width="32"
            height="32"
            viewBox="0 0 32 32"
            fill="none"
            className="text-[var(--color-text-muted)]"
          >
            <path
              d="M8 8L24 24M24 8L8 24"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
          <span className="text-xs text-[var(--color-text-muted)]">None</span>
        </button>

        {/* Asset thumbnails */}
        {filteredAssets.map((asset) => {
          const isEquipped = equippedAssets[asset.layerNum]?.includes(asset.id);
          const thumbnail = thumbnails[asset.id];
          const hasFailed = failedThumbnails.has(asset.id);

          return (
            <button
              key={asset.id}
              onClick={() => handleAssetClick(asset)}
              className={`
                aspect-square rounded-lg border-2 flex flex-col items-center justify-center
                cursor-pointer transition-all overflow-hidden
                ${
                  isEquipped
                    ? "border-[var(--color-expr-active)] ring-2 ring-[var(--color-expr-active)] ring-opacity-50"
                    : "border-[var(--color-warm-border)] hover:border-[var(--color-accent)]"
                }
              `}
              title={asset.name}
            >
              {thumbnail ? (
                <div className="relative w-full h-full p-1">
                  <img
                    src={thumbnail}
                    alt={asset.name}
                    className="w-full h-full object-contain"
                    onError={(e) => {
                      console.error(`Thumbnail image failed to load: ${thumbnail}`);
                      setFailedThumbnails((prev) => new Set([...prev, asset.id]));
                      // Hide the broken image
                      e.currentTarget.style.display = "none";
                    }}
                  />
                  {isEquipped && (
                    <div className="absolute top-1 right-1 w-4 h-4 rounded-full bg-[var(--color-expr-active)] flex items-center justify-center">
                      <svg
                        width="12"
                        height="12"
                        viewBox="0 0 12 12"
                        fill="none"
                      >
                        <path
                          d="M2 6L5 9L10 3"
                          stroke="white"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                      </svg>
                    </div>
                  )}
                </div>
              ) : hasFailed ? (
                <div className="w-full h-full flex flex-col items-center justify-center gap-1 bg-[var(--color-warm-bg)]">
                  <svg
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    className="text-red-400"
                  >
                    <circle
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="2"
                    />
                    <path
                      d="M12 8v4m0 4h.01"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                    />
                  </svg>
                  <span className="text-xs text-red-400">Failed</span>
                </div>
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-[var(--color-warm-bg)]">
                  <div className="w-4 h-4 border-2 border-[var(--color-text-muted)] border-t-transparent rounded-full animate-spin" />
                </div>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
