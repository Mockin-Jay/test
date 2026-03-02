interface StyledSelectProps {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  label?: string;
}

export default function StyledSelect({
  value,
  onChange,
  options,
  label,
}: StyledSelectProps) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      title={label}
      className="appearance-none bg-white rounded-lg px-4 py-2 pr-8 text-sm
                 text-[var(--color-text)] border border-[var(--color-warm-border)]
                 cursor-pointer outline-none hover:border-[var(--color-accent)]
                 transition-colors"
      style={{
        backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath d='M3 5l3 3 3-3' stroke='%238a7d72' fill='none' stroke-width='1.5' stroke-linecap='round'/%3E%3C/svg%3E")`,
        backgroundRepeat: "no-repeat",
        backgroundPosition: "right 10px center",
      }}
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  );
}
