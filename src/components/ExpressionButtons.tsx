import { useState } from "react";
import { useMinikoStore } from "../store/minikoStore";

export default function ExpressionButtons() {
  const [showDuplicateMenu, setShowDuplicateMenu] = useState(false);
  const activeExpressionSlot = useMinikoStore((s) => s.activeExpressionSlot);

  const handleDuplicate = (targetSlot: number) => {
    // Copy current expression to target slot (Phase 4 implementation)
    console.log(`Duplicate expression ${activeExpressionSlot} to ${targetSlot}`);
    setShowDuplicateMenu(false);
  };

  const handleReset = () => {
    // Reset current expression slot (Phase 4 implementation)
    console.log(`Reset expression slot ${activeExpressionSlot}`);
  };

  return (
    <div className="px-4 py-2 border-b border-[var(--color-warm-border)]">
      <div className="flex items-center justify-center gap-3">
        {/* Duplicate button with dropdown */}
        <div className="relative">
          <button
            onClick={() => setShowDuplicateMenu(!showDuplicateMenu)}
            className="px-6 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)]
                       text-white text-sm font-medium transition-colors cursor-pointer
                       flex items-center gap-2"
          >
            duplicate expressions
            <svg
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="none"
              className={`transform transition-transform ${
                showDuplicateMenu ? "rotate-180" : ""
              }`}
            >
              <path
                d="M2 4L6 8L10 4"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          </button>

          {showDuplicateMenu && (
            <div
              className="absolute top-full left-0 mt-1 bg-white rounded-lg shadow-lg
                         border border-[var(--color-warm-border)] py-1 z-50 min-w-[180px]"
            >
              {[1, 2, 3, 4].map((slot) => (
                <button
                  key={slot}
                  onClick={() => handleDuplicate(slot)}
                  disabled={slot === activeExpressionSlot}
                  className="w-full px-4 py-2 text-left text-sm hover:bg-[var(--color-warm-bg)]
                             disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
                >
                  Copy to Slot {slot}
                  {slot === activeExpressionSlot && " (current)"}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Reset button */}
        <button
          onClick={handleReset}
          className="px-6 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)]
                     text-white text-sm font-medium transition-colors cursor-pointer"
        >
          reset expression
        </button>

        {/* Eye icon (Phase 4: visibility toggle) */}
        <button
          className="w-10 h-10 rounded-full bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                     hover:bg-[var(--color-accent)] hover:text-white
                     transition-colors flex items-center justify-center cursor-pointer"
          title="Toggle Visibility (Phase 4)"
        >
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
            <path
              d="M10 5C5 5 2 10 2 10s3 5 8 5 8-5 8-5-3-5-8-5z"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
            <circle
              cx="10"
              cy="10"
              r="2"
              stroke="currentColor"
              strokeWidth="1.5"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
