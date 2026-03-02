import { useState } from "react";
import { useMinikoStore } from "../store/minikoStore";
import AssetCategoryBar from "../components/AssetCategoryBar";
import AssetGrid from "../components/AssetGrid";
import ColorPalette from "../components/ColorPalette";
import ColorPicker from "../components/ColorPicker";
import DPadControl from "../components/DPadControl";
import ExpressionButtons from "../components/ExpressionButtons";
import ExpressionSlots from "../components/ExpressionSlots";
import OutfitBar from "../components/OutfitBar";
import IconButton from "../components/IconButton";

export default function WardrobeScreen() {
  const setScreen = useMinikoStore((s) => s.setScreen);
  const selectedCategory = useMinikoStore((s) => s.selectedCategory);
  const setSelectedCategory = useMinikoStore((s) => s.setSelectedCategory);
  const selectedAssetForColor = useMinikoStore((s) => s.selectedAssetForColor);
  const colorOverrides = useMinikoStore((s) => s.colorOverrides);
  const setColorOverride = useMinikoStore((s) => s.setColorOverride);

  const [showColorPicker, setShowColorPicker] = useState(false);
  const [activeColorSlot, setActiveColorSlot] = useState<string | null>(null);

  const handleOpenColorPicker = (slotId: string) => {
    setActiveColorSlot(slotId);
    setShowColorPicker(true);
  };

  const handleColorChange = (color: string) => {
    if (selectedAssetForColor && activeColorSlot) {
      setColorOverride(selectedAssetForColor, activeColorSlot, color);
    }
  };

  const handleResetColor = () => {
    if (selectedAssetForColor && activeColorSlot) {
      // Remove color override (Phase 4: implement proper reset to default)
      setColorOverride(selectedAssetForColor, activeColorSlot, "#000000");
    }
    setShowColorPicker(false);
  };

  const currentColor =
    selectedAssetForColor && activeColorSlot
      ? colorOverrides[selectedAssetForColor]?.[activeColorSlot] || "#000000"
      : "#000000";

  return (
    <div className="relative h-full overflow-hidden flex flex-col">
      {/* ===== TOP BAR ===== */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--color-warm-border)] bg-[var(--color-warm-bg)]">
        <IconButton
          bg="var(--color-accent)"
          size={48}
          onClick={() => setScreen("home")}
          title="Back to Home"
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="white"
            strokeWidth="2"
          >
            <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
            <polyline points="9 22 9 12 15 12 15 22" />
          </svg>
        </IconButton>

        <h1 className="text-lg font-semibold text-[var(--color-accent)]">
          Wardrobe
        </h1>

        <div className="w-12" /> {/* Spacer for centering */}
      </div>

      {/* ===== CATEGORY BAR ===== */}
      <div className="bg-[var(--color-warm-bg)]">
        <AssetCategoryBar
          selectedCategory={selectedCategory}
          onSelectCategory={setSelectedCategory}
        />

        {/* ===== EXPRESSION BUTTONS ===== */}
        <ExpressionButtons />

        {/* ===== COLOR PALETTE ===== */}
        <ColorPalette onOpenColorPicker={handleOpenColorPicker} />
      </div>

      {/* ===== MAIN CONTENT AREA ===== */}
      <div className="flex-1 flex gap-4 px-4 py-4 min-h-0">
        {/* Left: Asset Grid */}
        <div className="w-80 flex flex-col bg-[var(--color-warm-card)] rounded-xl border border-[var(--color-warm-border)] overflow-hidden">
          <AssetGrid selectedCategory={selectedCategory} />
        </div>

        {/* Center: Avatar Preview (wgpu renders behind the transparent webview) */}
        <div className="flex-1 flex flex-col items-center justify-end pb-4">
          {/* D-Pad positioned at bottom of preview area */}
          <div className="bg-[var(--color-warm-card)] rounded-xl border border-[var(--color-warm-border)] p-2">
            <DPadControl />
          </div>
        </div>

        {/* Right: Expression Slots + Outfit Bar */}
        <div className="w-48 flex flex-col gap-4">
          <div className="bg-[var(--color-warm-card)] rounded-xl border border-[var(--color-warm-border)] p-3">
            <ExpressionSlots />
          </div>
          <div className="bg-[var(--color-warm-card)] rounded-xl border border-[var(--color-warm-border)] p-3">
            <OutfitBar />
          </div>
        </div>
      </div>

      {/* ===== COLOR PICKER POPUP ===== */}
      {showColorPicker && (
        <div className="absolute inset-0 flex items-center justify-center bg-black bg-opacity-20 z-40">
          <ColorPicker
            isOpen={showColorPicker}
            onClose={() => setShowColorPicker(false)}
            currentColor={currentColor}
            onColorChange={handleColorChange}
            onReset={handleResetColor}
          />
        </div>
      )}
    </div>
  );
}
