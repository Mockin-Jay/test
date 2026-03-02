/**
 * AvatarCanvas - Transparent placeholder component.
 * The actual avatar is rendered by the Rust wgpu renderer behind the webview.
 * IPC sync is handled by useRendererSync hook at the App level.
 */

interface AvatarCanvasProps {
  className?: string;
}

export default function AvatarCanvas({ className }: AvatarCanvasProps) {
  return (
    <div
      className={className}
      style={{ width: "100%", height: "100%", pointerEvents: "none" }}
    />
  );
}
