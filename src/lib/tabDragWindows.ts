import { getAllWebviewWindows, getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import type { Bounds, TabDragWindows } from './tabDragFlow';

type Window = ReturnType<typeof getCurrentWebviewWindow>;

async function describe(window: Window): Promise<{ label: string; bounds: Bounds }> {
  const [position, size, scale] = await Promise.all([
    window.outerPosition(), window.outerSize(), window.scaleFactor()
  ]);
  return { label: window.label, bounds: {
    left: position.x / scale, top: position.y / scale,
    right: (position.x + size.width) / scale,
    bottom: (position.y + size.height) / scale
  } };
}

export const tabDragWindows: TabDragWindows = {
  current: () => describe(getCurrentWebviewWindow()),
  others: async () => Promise.all((await getAllWebviewWindows()).map(describe)),
  closeCurrent: () => getCurrentWebviewWindow().close()
};
