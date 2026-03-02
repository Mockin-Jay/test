/**
 * SVGRecolorer - Replaces sentinel colors in SVG strings with user-chosen colors.
 *
 * Sentinel color mapping (CSS named colors used in SVGs):
 *   red   -> base color
 *   lime  -> shadow  (~18% darker lightness)
 *   blue  -> shadow2 (~33% darker lightness)
 *   #000 / black -> outline (kept as-is by default, or replaced if user provides)
 *
 * The SVGs use CSS named colors in their <style> blocks (e.g. `fill: red;`),
 * so we replace those named color strings.
 */

import { computeVariants } from "../utils/colorUtils";

export interface ColorSlots {
  /** User's chosen base color for this sentinel group */
  base: string;
}

/**
 * Given an SVG string and a base hex color, replace sentinel colors
 * with the user's color + computed shadow/highlight variants.
 *
 * If no color is provided, returns the SVG unchanged.
 */
export function recolorSVG(svgText: string, baseHex?: string): string {
  if (!baseHex) return svgText;

  const { base, shadow, shadow2 } = computeVariants(baseHex);

  // Replace CSS named color values in style blocks and inline styles.
  // Match word-boundary "red", "lime", "blue" used as fill/stroke values.
  let result = svgText;

  // Replace fill/stroke values: "red" -> base color
  result = result.replace(
    /(\bfill\s*:\s*)red(\s*[;"])/gi,
    `$1${base}$2`
  );
  result = result.replace(
    /(\bstroke\s*:\s*)red(\s*[;"])/gi,
    `$1${base}$2`
  );

  // Replace fill/stroke values: "lime" -> shadow
  result = result.replace(
    /(\bfill\s*:\s*)lime(\s*[;"])/gi,
    `$1${shadow}$2`
  );
  result = result.replace(
    /(\bstroke\s*:\s*)lime(\s*[;"])/gi,
    `$1${shadow}$2`
  );

  // Replace fill/stroke values: "blue" -> shadow2
  result = result.replace(
    /(\bfill\s*:\s*)blue(\s*[;"])/gi,
    `$1${shadow2}$2`
  );
  result = result.replace(
    /(\bstroke\s*:\s*)blue(\s*[;"])/gi,
    `$1${shadow2}$2`
  );

  // Also handle XML attribute forms: fill="red", stroke="red"
  result = result.replace(/(\bfill=")red(")/gi, `$1${base}$2`);
  result = result.replace(/(\bstroke=")red(")/gi, `$1${base}$2`);
  result = result.replace(/(\bfill=")lime(")/gi, `$1${shadow}$2`);
  result = result.replace(/(\bstroke=")lime(")/gi, `$1${shadow}$2`);
  result = result.replace(/(\bfill=")blue(")/gi, `$1${shadow2}$2`);
  result = result.replace(/(\bstroke=")blue(")/gi, `$1${shadow2}$2`);

  return result;
}

/**
 * Convert an SVG string to a data URL for use in <img> tags.
 */
export function svgToDataUrl(svgText: string): string {
  return `data:image/svg+xml,${encodeURIComponent(svgText)}`;
}

/**
 * Revoke a previously created blob URL to free memory.
 * No-op for data URLs (they don't need revoking).
 */
export function revokeBlobUrl(url: string): void {
  if (url.startsWith("blob:")) {
    URL.revokeObjectURL(url);
  }
}
