import { create } from "zustand";

export interface ExpressionSlot {
  /** Per-layer asset overrides for this expression (layerNum -> assetId) */
  layerOverrides: Record<number, string>;
}

export interface OutfitSnapshot {
  equippedAssets: Record<number, string[]>;
  colorOverrides: Record<string, Record<string, string>>;
  userOffsets: Record<string, { x: number; y: number }>;
  expressions: Record<number, ExpressionSlot>;
}

export type Screen = "home" | "wardrobe";

export interface MinikoState {
  // Navigation
  currentScreen: Screen;
  setScreen: (screen: Screen) => void;

  // Asset selections: layerNum -> equipped asset ID(s)
  equippedAssets: Record<number, string[]>;
  equipAsset: (layerNum: number, assetId: string) => void;
  deselect: (layerNum: number, assetId: string) => void;
  clearLayer: (layerNum: number) => void;

  // Color overrides: assetId -> slotId -> hex color
  colorOverrides: Record<string, Record<string, string>>;
  setColorOverride: (assetId: string, slot: string, hex: string) => void;

  // User position offsets: assetId -> { x, y }
  userOffsets: Record<string, { x: number; y: number }>;
  setUserOffset: (assetId: string, offset: { x: number; y: number }) => void;

  // Expressions: slotNum -> per-layer asset overrides
  expressions: Record<number, ExpressionSlot>;

  // Wardrobe UI state
  selectedCategory: string;
  setSelectedCategory: (category: string) => void;
  selectedAssetForColor: string | null;
  setSelectedAssetForColor: (assetId: string | null) => void;

  // Active state
  activeExpressionSlot: number;
  setActiveExpressionSlot: (slot: number) => void;
  isTalking: boolean;
  currentOutfit: string;
  setCurrentOutfit: (outfit: string) => void;

  // Outfits: name -> full snapshot
  outfits: Record<string, OutfitSnapshot>;

  // Settings
  selectedMicId: string;
  setSelectedMicId: (id: string) => void;
  volumeThreshold: number;
  setVolumeThreshold: (value: number) => void;
  holdTime: number;
  setHoldTime: (value: number) => void;
  stationaryEffect: string;
  setStationaryEffect: (effect: string) => void;
  talkingEffect: string;
  setTalkingEffect: (effect: string) => void;
  micEnabled: boolean;
  toggleMic: () => void;
  appVolume: number;
  setAppVolume: (value: number) => void;
  faceTrackingEnabled: boolean;
}

export const useMinikoStore = create<MinikoState>((set) => ({
  // Navigation
  currentScreen: "home",
  setScreen: (screen) => set({ currentScreen: screen }),

  // Asset selections
  equippedAssets: {
    // Default equipped assets for a full avatar look
    32: ["body"],                    // Body
    8:  ["animal-ear_front_1"],   
    38:  ["animal-ear_back_1"],       // Animal Ears (front and back)
    17: ["hair_front_28"],      // Front Hair
    36: ["hair_back_28"],             // Back Hair
    9: ["glasses_1"],              // Glasses
  },

  equipAsset: (layerNum, assetId) =>
    set((state) => ({
      equippedAssets: {
        ...state.equippedAssets,
        [layerNum]: [assetId],
      },
    })),

  deselect: (layerNum, assetId) =>
    set((state) => ({
      equippedAssets: {
        ...state.equippedAssets,
        [layerNum]: (state.equippedAssets[layerNum] ?? []).filter(
          (id) => id !== assetId
        ),
      },
    })),

  clearLayer: (layerNum) =>
    set((state) => ({
      equippedAssets: {
        ...state.equippedAssets,
        [layerNum]: [],
      },
    })),

  // Color overrides - default colors for a presentable avatar
  colorOverrides: {
    sclera_right_1: { base: "#ffffff" },
    sclera_left_1: { base: "#ffffff" },
    eye_shape_right_1_a: { base: "#4a90d9" },
    eye_shape_left_1_a: { base: "#4a90d9" },
    iris_right_1: { base: "#4a90d9" },
    iris_left_1: { base: "#4a90d9" },
    ear_human: { base: "#ffdfcf" },
    hair_front_1_human: { base: "#5c3a1e" },
    hair_back_1: { base: "#5c3a1e" },
    bang_right_1: { base: "#5c3a1e" },
    bang_left_1: { base: "#5c3a1e" },
    outfit_base_1: { base: "#e8837c" },
  },
  setColorOverride: (assetId, slot, hex) =>
    set((state) => ({
      colorOverrides: {
        ...state.colorOverrides,
        [assetId]: {
          ...state.colorOverrides[assetId],
          [slot]: hex,
        },
      },
    })),

  // User offsets
  userOffsets: {},
  setUserOffset: (assetId, offset) =>
    set((state) => ({
      userOffsets: {
        ...state.userOffsets,
        [assetId]: offset,
      },
    })),

  // Expressions (4 slots, initialized empty)
  expressions: {
    1: { layerOverrides: {} },
    2: { layerOverrides: {} },
    3: { layerOverrides: {} },
    4: { layerOverrides: {} },
  },

  // Wardrobe UI state
  selectedCategory: "hair",
  setSelectedCategory: (category) => set({ selectedCategory: category }),
  selectedAssetForColor: null,
  setSelectedAssetForColor: (assetId) => set({ selectedAssetForColor: assetId }),

  // Active state
  activeExpressionSlot: 1,
  setActiveExpressionSlot: (slot) => set({ activeExpressionSlot: slot }),
  isTalking: false,
  currentOutfit: "Default",
  setCurrentOutfit: (outfit) => set({ currentOutfit: outfit }),

  // Outfits
  outfits: {},

  // Settings
  selectedMicId: "",
  setSelectedMicId: (id) => set({ selectedMicId: id }),
  volumeThreshold: 0.15,
  setVolumeThreshold: (value) => set({ volumeThreshold: value }),
  holdTime: 150,
  setHoldTime: (value) => set({ holdTime: value }),
  stationaryEffect: "breathing",
  setStationaryEffect: (effect) => set({ stationaryEffect: effect }),
  talkingEffect: "bounce",
  setTalkingEffect: (effect) => set({ talkingEffect: effect }),
  micEnabled: false,
  toggleMic: () => set((state) => ({ micEnabled: !state.micEnabled })),
  appVolume: 0.5,
  setAppVolume: (value) => set({ appVolume: value }),
  faceTrackingEnabled: false,
}));
