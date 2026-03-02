import { useMinikoStore } from "../store/minikoStore";

const NUDGE_AMOUNT = 5; // pixels

export default function DPadControl() {
  const selectedAssetForColor = useMinikoStore((s) => s.selectedAssetForColor);
  const userOffsets = useMinikoStore((s) => s.userOffsets);
  const setUserOffset = useMinikoStore((s) => s.setUserOffset);

  if (!selectedAssetForColor) {
    return (
      <div className="flex flex-col items-center gap-2 opacity-30 pointer-events-none">
        <div className="text-xs text-[var(--color-text-muted)] text-center">
          Select asset to adjust position
        </div>
      </div>
    );
  }

  const currentOffset = userOffsets[selectedAssetForColor] || { x: 0, y: 0 };

  const nudge = (dx: number, dy: number) => {
    setUserOffset(selectedAssetForColor, {
      x: currentOffset.x + dx,
      y: currentOffset.y + dy,
    });
  };

  const reset = () => {
    setUserOffset(selectedAssetForColor, { x: 0, y: 0 });
  };

  return (
    <div className="flex flex-col items-center gap-2">
      {/* D-pad grid */}
      <div className="grid grid-cols-3 grid-rows-3 gap-1">
        {/* Top row */}
        <div />
        <button
          onClick={() => nudge(0, -NUDGE_AMOUNT)}
          className="w-10 h-10 rounded bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer"
          title="Move Up"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M8 12V4M8 4L4 8M8 4L12 8"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <div />

        {/* Middle row */}
        <button
          onClick={() => nudge(-NUDGE_AMOUNT, 0)}
          className="w-10 h-10 rounded bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer"
          title="Move Left"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M12 8H4M4 8L8 4M4 8L8 12"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <button
          onClick={reset}
          className="w-10 h-10 rounded-full bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer text-xs font-medium"
          title="Reset Position"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <circle
              cx="8"
              cy="8"
              r="3"
              fill="currentColor"
            />
          </svg>
        </button>
        <button
          onClick={() => nudge(NUDGE_AMOUNT, 0)}
          className="w-10 h-10 rounded bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer"
          title="Move Right"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M4 8H12M12 8L8 4M12 8L8 12"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>

        {/* Bottom row */}
        <div />
        <button
          onClick={() => nudge(0, NUDGE_AMOUNT)}
          className="w-10 h-10 rounded bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer"
          title="Move Down"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M8 4V12M8 12L4 8M8 12L12 8"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <div />
      </div>

      {/* Offset display */}
      <div className="text-xs text-[var(--color-text-muted)] font-mono">
        X: {currentOffset.x} Y: {currentOffset.y}
      </div>
    </div>
  );
}
