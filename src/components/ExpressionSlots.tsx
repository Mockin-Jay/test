import { useMinikoStore } from "../store/minikoStore";

export default function ExpressionSlots() {
  const activeSlot = useMinikoStore((s) => s.activeExpressionSlot);
  const setActiveSlot = useMinikoStore((s) => s.setActiveExpressionSlot);

  return (
    <div className="flex flex-col gap-2">
      {[1, 2, 3, 4].map((slot) => {
        const isActive = activeSlot === slot;
        return (
          <button
            key={slot}
            onClick={() => setActiveSlot(slot)}
            className="relative rounded-xl w-16 h-16 flex items-center justify-center
                       cursor-pointer border-2 transition-all duration-150
                       hover:scale-105 active:scale-95"
            style={{
              backgroundColor: isActive
                ? "var(--color-expr-active)"
                : "var(--color-warm-card)",
              borderColor: isActive
                ? "var(--color-expr-active)"
                : "var(--color-warm-border)",
            }}
          >
            <img
              src="/icons/DefaultStateIcon.png"
              alt={`Expression ${slot}`}
              className="w-8 h-8"
              style={{
                filter: isActive
                  ? "brightness(10)"
                  : "brightness(0) opacity(0.3)",
              }}
            />
            {/* Number badge */}
            <span
              className="absolute bottom-1 right-1.5 text-xs font-bold rounded-full
                         w-5 h-5 flex items-center justify-center"
              style={{
                backgroundColor: isActive
                  ? "rgba(255,255,255,0.3)"
                  : "var(--color-warm-border)",
                color: isActive ? "white" : "var(--color-text-muted)",
              }}
            >
              {slot}
            </span>
          </button>
        );
      })}
    </div>
  );
}
