import { useState } from "react";
import { useMinikoStore } from "../store/minikoStore";
import AvatarCanvas from "../components/AvatarCanvas";
import IconButton from "../components/IconButton";
import StyledSelect from "../components/StyledSelect";
import VerticalSlider from "../components/VerticalSlider";
import ExpressionSlots from "../components/ExpressionSlots";
import MicDeviceSelector from "../components/MicDeviceSelector";
import OutfitBar from "../components/OutfitBar";

const STATIONARY_EFFECTS = [
  { value: "none", label: "none" },
  { value: "breathing", label: "breathing" },
  { value: "float", label: "float" },
];

const TALKING_EFFECTS = [
  { value: "none", label: "none" },
  { value: "bounce", label: "bounce" },
  { value: "shake", label: "shake" },
];

export default function HomeScreen() {
  const setScreen = useMinikoStore((s) => s.setScreen);
  const stationaryEffect = useMinikoStore((s) => s.stationaryEffect);
  const setStationaryEffect = useMinikoStore((s) => s.setStationaryEffect);
  const talkingEffect = useMinikoStore((s) => s.talkingEffect);
  const setTalkingEffect = useMinikoStore((s) => s.setTalkingEffect);
  const volumeThreshold = useMinikoStore((s) => s.volumeThreshold);
  const setVolumeThreshold = useMinikoStore((s) => s.setVolumeThreshold);
  const holdTime = useMinikoStore((s) => s.holdTime);
  const setHoldTime = useMinikoStore((s) => s.setHoldTime);
  const micEnabled = useMinikoStore((s) => s.micEnabled);
  const appVolume = useMinikoStore((s) => s.appVolume);
  const setAppVolume = useMinikoStore((s) => s.setAppVolume);

  const [showMicDevices, setShowMicDevices] = useState(false);
  const [showVolumeSlider, setShowVolumeSlider] = useState(false);

  return (
    <div className="relative h-full overflow-hidden">
      {/* ===== AVATAR CANVAS (full window, bottom-anchored) ===== */}
      <div className="absolute inset-0 flex items-end justify-center">
        <AvatarCanvas className="w-full h-full" />
      </div>

      {/* ===== UI OVERLAY ===== */}
      <div className="relative z-10 flex flex-col h-full pointer-events-none">
      {/* ===== TOP BAR ===== */}
      <div className="flex items-center justify-between px-4 py-3 pointer-events-auto">
        {/* Left: wardrobe + screenshot */}
        <div className="flex gap-3">
          <IconButton
            bg="var(--color-btn-coral)"
            size={52}
            onClick={() => setScreen("wardrobe")}
            title="Wardrobe"
          >
            <img
              src="/icons/charactercustomicon.png"
              alt="Wardrobe"
              className="w-full h-full rounded-full"
            />
          </IconButton>
          <IconButton bg="var(--color-btn-green)" size={52} title="Screenshot">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="white"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <rect x="2" y="5" width="20" height="15" rx="2" />
              <circle cx="12" cy="13" r="4" />
              <path d="M8 5l1-2h6l1 2" />
            </svg>
          </IconButton>
        </div>

        {/* Center: effect dropdowns */}
        <div className="flex gap-3">
          <StyledSelect
            value={stationaryEffect}
            onChange={setStationaryEffect}
            options={STATIONARY_EFFECTS}
            label="Stationary Effect"
          />
          <StyledSelect
            value={talkingEffect}
            onChange={setTalkingEffect}
            options={TALKING_EFFECTS}
            label="Talking Effect"
          />
        </div>

        {/* Right: hotkey button */}
        <div>
          <IconButton bg="var(--color-btn-blue)" size={52} title="Hotkeys">
            <img
              src="/icons/hotkeyIcon.png"
              alt="Hotkeys"
              className="w-full h-full rounded-full"
            />
          </IconButton>
        </div>
      </div>

      {/* ===== MIDDLE AREA (left sliders + right expressions) ===== */}
      <div className="flex-1 flex min-h-0 px-4 gap-4 justify-between">
        {/* Left sidebar: vertical sliders side by side */}
        <div className="flex flex-row items-center justify-center gap-1 py-4 pointer-events-auto">
          <VerticalSlider
            value={volumeThreshold}
            onChange={setVolumeThreshold}
            min={0}
            max={1}
            trackColor="#e8dfd3"
            height={140}
          />
          <VerticalSlider
            value={holdTime}
            onChange={setHoldTime}
            min={0}
            max={1000}
            step={10}
            trackColor="var(--color-slider-thumb)"
            height={140}
          />
        </div>

        {/* Spacer — canvas shows through from behind */}
        <div className="flex-1" />

        {/* Right sidebar: expression slots */}
        <div className="flex items-center py-4 pointer-events-auto">
          <ExpressionSlots />
        </div>
      </div>

      {/* ===== BOTTOM BAR ===== */}
      <div className="flex items-center justify-between px-4 py-3 pointer-events-auto">
        {/* Left: mic toggle (with floating device popup) + volume icon (with floating slider) */}
        <div className="flex items-center gap-3">
          {/* Mic button + floating device selector */}
          <div className="relative">
            <IconButton
              bg={
                micEnabled
                  ? "var(--color-btn-yellow)"
                  : "var(--color-warm-border)"
              }
              size={44}
              onClick={() => {
                setShowMicDevices(!showMicDevices);
                setShowVolumeSlider(false);
              }}
              title={micEnabled ? "Disable mic" : "Enable mic"}
            >
              <img
                src="/icons/micicon.png"
                alt="Microphone"
                className="w-full h-full rounded-full"
              />
            </IconButton>
            {showMicDevices && (
              <MicDeviceSelector onClose={() => setShowMicDevices(false)} />
            )}
          </div>

          {/* Volume button + floating slider */}
          <div className="relative">
            <IconButton
              bg="var(--color-btn-orange)"
              size={36}
              onClick={() => {
                setShowVolumeSlider(!showVolumeSlider);
                setShowMicDevices(false);
              }}
              title="Volume"
            >
              <img
                src="/icons/VolumeIcon.png"
                alt="Volume"
                className="w-full h-full rounded-full"
              />
            </IconButton>
            {showVolumeSlider && (
              <div
                className="absolute bottom-full left-0 mb-2 bg-white rounded-lg shadow-lg
                           border border-[var(--color-warm-border)] px-3 py-2 z-50
                           flex items-center gap-2"
              >
                <input
                  type="range"
                  min={0}
                  max={1}
                  step={0.01}
                  value={appVolume}
                  onChange={(e) => setAppVolume(parseFloat(e.target.value))}
                  className="volume-slider w-32"
                />
              </div>
            )}
          </div>
        </div>

        {/* Right: outfit controls */}
        <OutfitBar />
      </div>
      </div>{/* end UI overlay */}
    </div>
  );
}
