/**
 * Asset utility functions for naming convention parsing and manifest handling.
 *
 * To be implemented in Phase 2/3.
 */

/** Layer definitions mapping the 41 layers to categories and rules */
export interface LayerDef {
  num: number;
  name: string;
  category: string;
  oneItemLimit: boolean;
}

export const LAYER_DEFS: LayerDef[] = [
  { num: 1, name: "FRONT SFX", category: "sfx", oneItemLimit: true },
  { num: 2, name: "FRONT Ribbon", category: "bows", oneItemLimit: false },
  { num: 3, name: "FRONT Hat", category: "hats", oneItemLimit: false },
  { num: 4, name: "RIGHT Hand", category: "hands", oneItemLimit: true },
  { num: 5, name: "LEFT Hand", category: "hands", oneItemLimit: true },
  { num: 6, name: "BOTH Hand", category: "hands", oneItemLimit: true },
  { num: 7, name: "Eyebrow", category: "eyebrows", oneItemLimit: true },
  { num: 8, name: "Cowlick & Animal Ear", category: "hair", oneItemLimit: false },
  { num: 9, name: "Glasses", category: "glasses", oneItemLimit: true },
  { num: 10, name: "RIGHT Iris", category: "face", oneItemLimit: false },
  { num: 11, name: "LEFT Iris", category: "face", oneItemLimit: false },
  { num: 12, name: "RIGHT Eye Shape", category: "face", oneItemLimit: true },
  { num: 13, name: "LEFT Eye Shape", category: "face", oneItemLimit: true },
  { num: 14, name: "RIGHT Pigtail", category: "hair", oneItemLimit: false },
  { num: 15, name: "RIGHT Bang", category: "hair", oneItemLimit: true },
  { num: 16, name: "BACK Hair Assist A", category: "hair", oneItemLimit: true },
  { num: 17, name: "FRONT Hair A", category: "hair", oneItemLimit: true },
  { num: 18, name: "LEFT Bang", category: "hair", oneItemLimit: true },
  { num: 19, name: "FRONT Hair B", category: "hair", oneItemLimit: true },
  { num: 20, name: "Mask & Marking", category: "glasses", oneItemLimit: false },
  { num: 21, name: "BACK Hair Assist B", category: "hair", oneItemLimit: true },
  { num: 22, name: "RIGHT Earring", category: "earrings", oneItemLimit: true },
  { num: 23, name: "TOP Outfit", category: "outfit", oneItemLimit: true },
  { num: 24, name: "MID Outfit", category: "outfit", oneItemLimit: true },
  { num: 25, name: "Choker", category: "earrings", oneItemLimit: true },
  { num: 26, name: "BASE Outfit", category: "outfit", oneItemLimit: true },
  { num: 27, name: "RIGHT Sclera", category: "face", oneItemLimit: true },
  { num: 28, name: "LEFT Sclera", category: "face", oneItemLimit: true },
  { num: 29, name: "Ear", category: "face", oneItemLimit: true },
  { num: 30, name: "Nose", category: "face", oneItemLimit: true },
  { num: 31, name: "Mouth", category: "mouth", oneItemLimit: true },
  { num: 32, name: "Body", category: "face", oneItemLimit: true },
  { num: 33, name: "LEFT Earring", category: "earrings", oneItemLimit: true },
  { num: 34, name: "TOP Outfit Cover", category: "outfit", oneItemLimit: true },
  { num: 35, name: "Wing & Tail", category: "wings", oneItemLimit: false },
  { num: 36, name: "BACK Hair", category: "hair", oneItemLimit: true },
  { num: 37, name: "BACK Ribbon", category: "bows", oneItemLimit: true },
  { num: 38, name: "BACK Hat & Animal Ear", category: "hats", oneItemLimit: false },
  { num: 39, name: "LEFT Pigtail", category: "hair", oneItemLimit: false },
  { num: 40, name: "Ponytail", category: "hair", oneItemLimit: false },
  { num: 41, name: "BACK SFX", category: "sfx", oneItemLimit: true },
];

/** User-facing categories for the wardrobe UI */
export const CATEGORIES = [
  { id: "hair", label: "Hair" },
  { id: "face", label: "Face" },
  { id: "mouth", label: "Mouth" },
  { id: "eyebrows", label: "Eyebrows" },
  { id: "hands", label: "Hands" },
  { id: "outfit", label: "Outfit" },
  { id: "glasses", label: "Glasses" },
  { id: "earrings", label: "Earrings" },
  { id: "hats", label: "Hats" },
  { id: "bows", label: "Bows" },
  { id: "wings", label: "Wings & Tails" },
  { id: "sfx", label: "SFX" },
] as const;
