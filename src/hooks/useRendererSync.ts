/**
 * useRendererSync - Syncs Zustand store state to the Rust wgpu renderer via IPC.
 * Must be mounted at the App level so it runs regardless of which screen is active.
 *
 * On first render, captures the current state as baseline (Rust already loaded
 * defaults at startup). Only subsequent changes trigger IPC calls.
 */

import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMinikoStore } from "../store/minikoStore";

interface LayerState {
  assetId: string;
  colors: Record<string, string>;
  offset: [number, number];
}

function buildLayerStates(
  equippedAssets: Record<number, string[]>,
  colorOverrides: Record<string, Record<string, string>>,
  userOffsets: Record<string, { x: number; y: number }>
): Record<number, LayerState | null> {
  const states: Record<number, LayerState | null> = {};
  for (let layerNum = 1; layerNum <= 41; layerNum++) {
    const assetIds = equippedAssets[layerNum];
    if (!assetIds || assetIds.length === 0) {
      states[layerNum] = null;
    } else {
      const assetId = assetIds[0];
      const colors = colorOverrides[assetId] || {};
      const offset: [number, number] = userOffsets[assetId]
        ? [userOffsets[assetId].x, userOffsets[assetId].y]
        : [0, 0];
      states[layerNum] = { assetId, colors, offset };
    }
  }
  return states;
}

export function useRendererSync() {
  const equippedAssets = useMinikoStore((s) => s.equippedAssets);
  const colorOverrides = useMinikoStore((s) => s.colorOverrides);
  const userOffsets = useMinikoStore((s) => s.userOffsets);

  const prevLayerStates = useRef<Record<number, LayerState | null> | null>(null);

  useEffect(() => {
    const next = buildLayerStates(equippedAssets, colorOverrides, userOffsets);

    // First render: capture baseline, don't send IPC (Rust loaded defaults)
    if (prevLayerStates.current === null) {
      prevLayerStates.current = next;
      return;
    }

    const prev = prevLayerStates.current;

    // Send only changed layers
    for (let layerNum = 1; layerNum <= 41; layerNum++) {
      const prevState = prev[layerNum];
      const nextState = next[layerNum];

      const prevKey = prevState ? JSON.stringify(prevState) : null;
      const nextKey = nextState ? JSON.stringify(nextState) : null;

      if (prevKey === nextKey) continue;

      if (!nextState) {
        invoke("clear_layer", { layerNum }).catch((err) =>
          console.warn(`Failed to clear layer ${layerNum}:`, err)
        );
      } else {
        const t0 = performance.now();
        invoke("update_layer", {
          layerNum,
          assetId: nextState.assetId,
          colors: nextState.colors,
          offset: nextState.offset,
        })
          .then(() => {
            console.log(
              `[TIMING] IPC update_layer layer=${layerNum} (${nextState.assetId}): ${(performance.now() - t0).toFixed(1)}ms`
            );
          })
          .catch((err) =>
            console.warn(
              `Failed to update layer ${layerNum} (${nextState.assetId}):`,
              err
            )
          );
      }
    }

    prevLayerStates.current = next;
  }, [equippedAssets, colorOverrides, userOffsets]);
}
