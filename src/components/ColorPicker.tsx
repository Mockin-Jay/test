import { useEffect, useRef, useState } from "react";
import { hexToHSL, hslToHex } from "../utils/colorUtils";

interface ColorPickerProps {
  isOpen: boolean;
  onClose: () => void;
  currentColor: string;
  onColorChange: (color: string) => void;
  onReset: () => void;
}

const PRESET_COLORS = [
  "#FF6B6B",
  "#FFA726",
  "#FFEB3B",
  "#66BB6A",
  "#42A5F5",
  "#AB47BC",
  "#8D6E63",
  "#BDBDBD",
];

export default function ColorPicker({
  isOpen,
  onClose,
  currentColor,
  onColorChange,
  onReset,
}: ColorPickerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const gradientRef = useRef<HTMLDivElement>(null);
  const hueRef = useRef<HTMLDivElement>(null);

  const [hsl, setHsl] = useState(hexToHSL(currentColor));
  const [isDraggingGradient, setIsDraggingGradient] = useState(false);
  const [isDraggingHue, setIsDraggingHue] = useState(false);

  // Update HSL when currentColor changes externally
  useEffect(() => {
    setHsl(hexToHSL(currentColor));
  }, [currentColor]);

  // Update color when HSL changes
  useEffect(() => {
    const hex = hslToHex(hsl.h / 360, hsl.s / 100, hsl.l / 100);
    if (hex !== currentColor) {
      onColorChange(hex);
    }
  }, [hsl, currentColor, onColorChange]);

  // Click outside to close
  useEffect(() => {
    if (!isOpen) return;

    const handleClickOutside = (e: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        onClose();
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen, onClose]);

  // Gradient dragging
  const handleGradientInteraction = (e: React.MouseEvent | React.TouchEvent) => {
    if (!gradientRef.current) return;
    const rect = gradientRef.current.getBoundingClientRect();
    const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
    const x = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const y = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));

    setHsl({
      h: hsl.h,
      s: x * 100,
      l: (1 - y) * 100,
    });
  };

  const handleGradientMouseDown = (e: React.MouseEvent) => {
    setIsDraggingGradient(true);
    handleGradientInteraction(e);
  };

  // Hue dragging
  const handleHueInteraction = (e: React.MouseEvent | React.TouchEvent) => {
    if (!hueRef.current) return;
    const rect = hueRef.current.getBoundingClientRect();
    const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
    const y = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
    const hue = y * 360;

    setHsl({
      h: hue,
      s: hsl.s,
      l: hsl.l,
    });
  };

  const handleHueMouseDown = (e: React.MouseEvent) => {
    setIsDraggingHue(true);
    handleHueInteraction(e);
  };

  // Mouse move and up handlers
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDraggingGradient) {
        handleGradientInteraction(e as any);
      } else if (isDraggingHue) {
        handleHueInteraction(e as any);
      }
    };

    const handleMouseUp = () => {
      setIsDraggingGradient(false);
      setIsDraggingHue(false);
    };

    if (isDraggingGradient || isDraggingHue) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);
      return () => {
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
      };
    }
  }, [isDraggingGradient, isDraggingHue, hsl]);

  const handleHexInput = (value: string) => {
    if (/^#[0-9A-Fa-f]{6}$/.test(value)) {
      setHsl(hexToHSL(value));
    }
  };

  if (!isOpen) return null;

  const pureHue = hslToHex(hsl.h / 360, 1, 0.5);
  const gradientX = hsl.s / 100;
  const gradientY = 1 - hsl.l / 100;

  return (
    <div
      ref={containerRef}
      className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2
                 bg-white rounded-lg shadow-2xl border border-[var(--color-warm-border)]
                 p-4 z-50 flex flex-col gap-3"
      style={{ width: "280px" }}
    >
      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-2 right-2 w-6 h-6 rounded-full
                   bg-[var(--color-warm-card)] hover:bg-[var(--color-accent)]
                   hover:text-white transition-colors flex items-center justify-center"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path
            d="M2 2L10 10M10 2L2 10"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </svg>
      </button>

      {/* Gradient square */}
      <div className="flex gap-2">
        <div
          ref={gradientRef}
          onMouseDown={handleGradientMouseDown}
          className="relative w-48 h-48 rounded-lg cursor-crosshair"
          style={{
            background: `
              linear-gradient(to top, #000, transparent),
              linear-gradient(to right, #fff, ${pureHue})
            `,
          }}
        >
          {/* Cursor indicator */}
          <div
            className="absolute w-4 h-4 border-2 border-white rounded-full shadow-lg pointer-events-none"
            style={{
              left: `${gradientX * 100}%`,
              top: `${gradientY * 100}%`,
              transform: "translate(-50%, -50%)",
            }}
          />
        </div>

        {/* Hue slider */}
        <div
          ref={hueRef}
          onMouseDown={handleHueMouseDown}
          className="relative w-6 h-48 rounded-lg cursor-pointer"
          style={{
            background: `linear-gradient(to bottom,
              #ff0000 0%, #ffff00 17%, #00ff00 33%, #00ffff 50%,
              #0000ff 67%, #ff00ff 83%, #ff0000 100%)`,
          }}
        >
          {/* Hue cursor */}
          <div
            className="absolute left-0 right-0 h-1 bg-white border border-gray-400 pointer-events-none"
            style={{
              top: `${(hsl.h / 360) * 100}%`,
              transform: "translateY(-50%)",
            }}
          />
        </div>
      </div>

      {/* Hex input */}
      <div className="flex items-center gap-2">
        <span className="text-sm text-[var(--color-text)] font-medium">#</span>
        <input
          type="text"
          value={currentColor.replace("#", "").toUpperCase()}
          onChange={(e) => handleHexInput(`#${e.target.value}`)}
          className="flex-1 px-2 py-1.5 rounded border border-[var(--color-warm-border)]
                     text-sm font-mono focus:outline-none focus:border-[var(--color-accent)]"
          maxLength={6}
        />
        <button
          onClick={onReset}
          className="px-3 py-1.5 rounded bg-[var(--color-warm-card)] hover:bg-[var(--color-accent)]
                     hover:text-white text-sm transition-colors border border-[var(--color-warm-border)]"
        >
          reset
        </button>
      </div>

      {/* Preset swatches */}
      <div className="grid grid-cols-8 gap-1.5">
        {PRESET_COLORS.map((color) => (
          <button
            key={color}
            onClick={() => setHsl(hexToHSL(color))}
            className="w-6 h-6 rounded border-2 border-[var(--color-warm-border)]
                       cursor-pointer hover:scale-110 transition-transform"
            style={{ backgroundColor: color }}
          />
        ))}
      </div>
    </div>
  );
}
