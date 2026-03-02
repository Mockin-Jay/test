import { useState } from "react";
import { useMinikoStore } from "../store/minikoStore";

interface ColorPaletteProps {
  onOpenColorPicker: (slotId: string) => void;
}

const SENTINEL_SLOTS = [
  { id: "base", label: "Base", defaultColor: "#ff0000" },
  { id: "shadow", label: "Shadow", defaultColor: "#00ff00" },
  { id: "shadow2", label: "Shadow 2", defaultColor: "#0000ff" },
  { id: "outline", label: "Outline", defaultColor: "#000000" },
];

export default function ColorPalette({ onOpenColorPicker }: ColorPaletteProps) {
  const [page, setPage] = useState(0);
  const selectedAssetForColor = useMinikoStore((s) => s.selectedAssetForColor);
  const colorOverrides = useMinikoStore((s) => s.colorOverrides);

  if (!selectedAssetForColor) {
    return (
      <div className="px-4 py-2 border-b border-[var(--color-warm-border)]">
        <div className="flex items-center justify-center gap-2 h-12">
          <span className="text-sm text-[var(--color-text-muted)]">
            Select an asset to customize colors
          </span>
        </div>
      </div>
    );
  }

  const assetColors = colorOverrides[selectedAssetForColor] || {};
  const swatches = SENTINEL_SLOTS;
  const maxVisible = 10;
  const totalPages = Math.ceil(swatches.length / maxVisible);
  const visibleSwatches = swatches.slice(
    page * maxVisible,
    (page + 1) * maxVisible
  );

  return (
    <div className="px-4 py-2 border-b border-[var(--color-warm-border)]">
      <div className="flex items-center justify-center gap-2">
        {/* Left arrow */}
        {totalPages > 1 && (
          <button
            onClick={() => setPage((p) => Math.max(0, p - 1))}
            disabled={page === 0}
            className="w-8 h-8 rounded-full bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                       flex items-center justify-center cursor-pointer
                       hover:bg-[var(--color-accent)] hover:text-white
                       disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                d="M10 4L6 8L10 12"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}

        {/* Color swatches */}
        <div className="flex gap-2">
          {visibleSwatches.map((swatch) => {
            const color = assetColors[swatch.id] || swatch.defaultColor;

            return (
              <button
                key={swatch.id}
                onClick={() => onOpenColorPicker(swatch.id)}
                className="w-10 h-10 rounded-full border-2 border-[var(--color-warm-border)]
                           cursor-pointer transition-transform hover:scale-110
                           flex items-center justify-center"
                style={{ backgroundColor: color }}
                title={swatch.label}
              />
            );
          })}
        </div>

        {/* Right arrow */}
        {totalPages > 1 && (
          <button
            onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
            disabled={page === totalPages - 1}
            className="w-8 h-8 rounded-full bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                       flex items-center justify-center cursor-pointer
                       hover:bg-[var(--color-accent)] hover:text-white
                       disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path
                d="M6 4L10 8L6 12"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}
