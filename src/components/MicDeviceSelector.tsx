import { useMinikoStore } from "../store/minikoStore";

const MOCK_DEVICES = [
  { id: "default", label: "blue yeti usb microphone" },
  { id: "device2", label: "microphone device 2" },
  { id: "device3", label: "microphone device 3" },
];

interface MicDeviceSelectorProps {
  onClose: () => void;
}

export default function MicDeviceSelector({ onClose }: MicDeviceSelectorProps) {
  const selectedMicId = useMinikoStore((s) => s.selectedMicId);
  const setSelectedMicId = useMinikoStore((s) => s.setSelectedMicId);

  const effectiveId = selectedMicId || MOCK_DEVICES[0].id;

  return (
    <div
      className="absolute bottom-full left-0 mb-2 bg-white rounded-lg shadow-lg
                 border border-[var(--color-warm-border)] py-1 z-50 min-w-[200px]"
    >
      {MOCK_DEVICES.map((device) => (
        <button
          key={device.id}
          onClick={() => {
            setSelectedMicId(device.id);
            onClose();
          }}
          className="w-full text-left px-3 py-2 text-sm hover:bg-[var(--color-warm-bg)]
                     cursor-pointer transition-colors flex items-center justify-between"
          style={{ color: "var(--color-text)" }}
        >
          <span className="truncate">{device.label}</span>
          {effectiveId === device.id && (
            <span className="text-[var(--color-expr-active)] ml-2">&#10003;</span>
          )}
        </button>
      ))}
    </div>
  );
}
