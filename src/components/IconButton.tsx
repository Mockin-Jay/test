interface IconButtonProps {
  bg: string;
  size?: number;
  onClick?: () => void;
  title?: string;
  children: React.ReactNode;
}

export default function IconButton({
  bg,
  size = 56,
  onClick,
  title,
  children,
}: IconButtonProps) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="rounded-full flex items-center justify-center cursor-pointer
                 hover:brightness-110 hover:scale-105 active:scale-95
                 transition-all duration-150 border-0 outline-none"
      style={{ width: size, height: size, backgroundColor: bg }}
    >
      {children}
    </button>
  );
}
