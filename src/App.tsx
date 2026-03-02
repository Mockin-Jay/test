import { useMinikoStore } from "./store/minikoStore";
import { useRendererSync } from "./hooks/useRendererSync";
import HomeScreen from "./screens/HomeScreen";
import WardrobeScreen from "./screens/WardrobeScreen";
import "./index.css";

function App() {
  const currentScreen = useMinikoStore((s) => s.currentScreen);

  // Sync Zustand state → Rust wgpu renderer (always active)
  useRendererSync();

  return (
    <div className="w-full h-full">
      {currentScreen === "home" && <HomeScreen />}
      {currentScreen === "wardrobe" && <WardrobeScreen />}
    </div>
  );
}

export default App;
