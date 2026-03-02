/** Types matching the structure of public/manifest.json */

export interface SubLayerDef {
  name: string;
  frameCount: number;
  /** Filename prefix for frames (e.g. "butterflybase" for "butterflybase_00000.png") */
  prefix: string;
  /** Cropped bounds for optimized animations (from animation-bounds.json) */
  bounds?: {
    width: number;
    height: number;
    offsetX: number;
    offsetY: number;
  };
}

export interface AssetDef {
  id: string;
  name: string;
  type: "svg" | "png-stack" | "png-animation" | "png";
  path: string;
  /** Number of sub-layer PNGs (png-stack only) */
  subLayerCount?: number;
  /** Sub-layer definitions (png-animation only) */
  subLayers?: SubLayerDef[];
}

export interface LayerManifest {
  layerNum: number;
  folderName: string;
  assets: AssetDef[];
}

export interface Manifest {
  layers: LayerManifest[];
}

let cachedManifest: Manifest | null = null;

export async function loadManifest(): Promise<Manifest> {
  if (cachedManifest) return cachedManifest;
  const resp = await fetch("/manifest.json");
  cachedManifest = (await resp.json()) as Manifest;
  return cachedManifest;
}

export function getLayerAssets(manifest: Manifest, layerNum: number): AssetDef[] {
  const layer = manifest.layers.find((l) => l.layerNum === layerNum);
  return layer?.assets ?? [];
}

export function getAssetById(
  manifest: Manifest,
  layerNum: number,
  assetId: string
): AssetDef | undefined {
  return getLayerAssets(manifest, layerNum).find((a) => a.id === assetId);
}
