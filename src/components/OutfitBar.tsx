import { useMinikoStore } from "../store/minikoStore";

export default function OutfitBar() {
  const currentOutfit = useMinikoStore((s) => s.currentOutfit);
  const setCurrentOutfit = useMinikoStore((s) => s.setCurrentOutfit);

  const outfits = [
    { value: "Default", label: "outfit 1" },
    { value: "Outfit 2", label: "outfit 2" },
    { value: "Outfit 3", label: "outfit 3" },
  ];

  return (
    <div className="flex items-center gap-2">
      {/* Edit button */}
      <button
        className="w-8 h-8 rounded-lg flex items-center justify-center cursor-pointer
                   bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                   hover:bg-[var(--color-accent)] hover:text-white transition-colors"
        title="Edit outfit"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        >
          <path d="M8.5 2.5l3 3M1.5 9.5l6-6 3 3-6 6H1.5v-3z" />
        </svg>
      </button>

      {/* Add button */}
      <button
        className="w-8 h-8 rounded-lg flex items-center justify-center cursor-pointer
                   bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                   hover:bg-[var(--color-expr-active)] hover:text-white transition-colors"
        title="Add outfit"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
        >
          <path d="M7 2v10M2 7h10" />
        </svg>
      </button>

      {/* Delete button */}
      <button
        className="w-8 h-8 rounded-lg flex items-center justify-center cursor-pointer
                   bg-[var(--color-warm-card)] border border-[var(--color-warm-border)]
                   hover:bg-red-400 hover:text-white transition-colors"
        title="Delete outfit"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          strokeLinecap="round"
        >
          <path d="M2 4h10M5 4V2.5h4V4M3.5 4v8h7V4" />
          <path d="M5.5 6.5v3M8.5 6.5v3" />
        </svg>
      </button>

      {/* Outfit selector */}
      <select
        value={currentOutfit}
        onChange={(e) => setCurrentOutfit(e.target.value)}
        className="appearance-none bg-white rounded-lg px-3 py-1.5 pr-7 text-sm
                   text-[var(--color-text)] border border-[var(--color-warm-border)]
                   cursor-pointer outline-none hover:border-[var(--color-accent)]
                   transition-colors"
        style={{
          backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath d='M3 5l3 3 3-3' stroke='%238a7d72' fill='none' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E")`,
          backgroundRepeat: "no-repeat",
          backgroundPosition: "right 8px center",
        }}
      >
        {outfits.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}
