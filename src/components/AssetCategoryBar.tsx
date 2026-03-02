import { CATEGORIES } from "../utils/assetUtils";

interface AssetCategoryBarProps {
  selectedCategory: string;
  onSelectCategory: (categoryId: string) => void;
}

// Inline SVG icons for mouth and eyebrows
const MouthIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
    <path
      d="M7 14c1 2 2.5 3 5 3s4-1 5-3"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
    />
  </svg>
);

const EyebrowsIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
    <path
      d="M6 9c1.5-1 3-1 5-1M13 9c1.5-1 3-1 5-1"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
    />
  </svg>
);

const CATEGORY_ICONS: Record<string, string | (() => React.JSX.Element)> = {
  hair: "/icons/hairIcon.png",
  face: "/icons/BodyIcon.png",
  mouth: MouthIcon,
  eyebrows: EyebrowsIcon,
  hands: "/icons/armsicon.png",
  outfit: "/icons/outfitIcon.png",
  glasses: "/icons/glassesIcon.png",
  earrings: "/icons/EarringIcon.png",
  hats: "/icons/hatIcon.png",
  bows: "/icons/newribbon.png",
  wings: "/icons/WingsIcon.png",
  sfx: "/icons/effectsIcon.png",
};

export default function AssetCategoryBar({
  selectedCategory,
  onSelectCategory,
}: AssetCategoryBarProps) {
  return (
    <div className="px-4 py-2 border-b border-[var(--color-warm-border)]">
      <div className="flex gap-2 overflow-x-auto pb-2">
        {CATEGORIES.map((category) => {
          const icon = CATEGORY_ICONS[category.id];
          const isActive = selectedCategory === category.id;
          const IconComponent = typeof icon === "function" ? icon : null;

          return (
            <button
              key={category.id}
              onClick={() => onSelectCategory(category.id)}
              className={`
                relative w-14 h-14 rounded-full flex items-center justify-center
                transition-all cursor-pointer border-2
                ${
                  isActive
                    ? "bg-[var(--color-accent)] border-[var(--color-accent-hover)] scale-110"
                    : "bg-[var(--color-warm-card)] border-[var(--color-warm-border)] hover:scale-105"
                }
              `}
              title={category.label}
            >
              {IconComponent ? (
                <div className={isActive ? "text-white" : "text-[var(--color-text)]"}>
                  <IconComponent />
                </div>
              ) : (
                <img
                  src={icon as string}
                  alt={category.label}
                  className="w-8 h-8 object-contain"
                  style={{
                    filter: isActive ? "brightness(0) invert(1)" : "none",
                  }}
                />
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
