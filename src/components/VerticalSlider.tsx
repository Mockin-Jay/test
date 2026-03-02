interface VerticalSliderProps {
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  trackColor: string;
  thumbColor?: string;
  height?: number;
}

export default function VerticalSlider({
  value,
  onChange,
  min = 0,
  max = 1,
  step = 0.01,
  trackColor,
  thumbColor = "var(--color-slider-thumb)",
  height = 160,
}: VerticalSliderProps) {
  const pct = ((value - min) / (max - min)) * 100;

  return (
    <div
      className="relative flex items-center justify-center"
      style={{ height, width: 32 }}
    >
      {/* Track background */}
      <div
        className="absolute rounded-full"
        style={{
          width: 8,
          height: height - 16,
          backgroundColor: "var(--color-warm-border)",
        }}
      />
      {/* Filled portion */}
      <div
        className="absolute rounded-full"
        style={{
          width: 8,
          height: `${((height - 16) * pct) / 100}px`,
          backgroundColor: trackColor,
          bottom: 8,
          left: "50%",
          transform: "translateX(-50%)",
        }}
      />
      {/* Native input (rotated) */}
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="vertical-slider"
        style={
          {
            height: 8,
            width: height - 16,
            "--thumb-color": thumbColor,
            "--track-color": "transparent",
          } as React.CSSProperties
        }
      />
    </div>
  );
}
