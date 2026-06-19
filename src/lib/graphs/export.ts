// Graph download helpers: write SVG markup directly, or rasterize it to PNG in
// the webview before handing the bytes to the Rust `write_bytes` command.

import { save as saveDialog } from '@tauri-apps/plugin-dialog';
import { writeBytes } from '$lib/api';
import type { GraphType } from './registry.svelte';

/** Resolution (px) graphs are rendered at when exporting. */
const EXPORT_SIZE = 1200;

/** Rasterizes SVG markup to PNG bytes via an offscreen canvas. */
function svgToPng(svg: string, size: number): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const img = new Image();
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('Canvas 2D context unavailable');
        // Fill with the themed sunken background so the PNG matches the on-screen
        // diagram instead of being transparent.
        const bg = getComputedStyle(document.documentElement)
          .getPropertyValue('--bg-sunken')
          .trim();
        if (bg) {
          ctx.fillStyle = bg;
          ctx.fillRect(0, 0, size, size);
        }
        ctx.drawImage(img, 0, 0, size, size);
        canvas.toBlob(async (out) => {
          if (!out) {
            reject(new Error('PNG encoding failed'));
            return;
          }
          resolve(new Uint8Array(await out.arrayBuffer()));
        }, 'image/png');
      } catch (e) {
        reject(e instanceof Error ? e : new Error(String(e)));
      } finally {
        URL.revokeObjectURL(url);
      }
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('Failed to load SVG for rasterization'));
    };
    img.src = url;
  });
}

/**
 * Prompts for a save location and writes the graph as SVG or PNG. The format is
 * chosen by the extension of the path the user picks in the dialog. Returns
 * false if the user cancelled.
 */
export async function downloadGraph(
  graph: GraphType,
  state: unknown
): Promise<boolean> {
  const path = await saveDialog({
    defaultPath: `${graph.fileBase}.svg`,
    filters: [
      { name: 'SVG image', extensions: ['svg'] },
      { name: 'PNG image', extensions: ['png'] }
    ]
  });
  if (!path) return false;

  const svg = await graph.render(graph.toOptions(state), EXPORT_SIZE);
  const bytes = path.toLowerCase().endsWith('.png')
    ? await svgToPng(svg, EXPORT_SIZE)
    : new TextEncoder().encode(svg);
  await writeBytes(path, bytes);
  return true;
}
