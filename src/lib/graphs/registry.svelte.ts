// Graph-type registry. Each graph is a self-contained descriptor: its own
// reactive control state, a controls component, and a render function. Adding a
// new visualization (Cartesian intensity, cone diagram, …) means appending one
// descriptor here — the VisualizationPanel host iterates this list and needs no
// changes.

import type { Component } from 'svelte';
import { renderPolarSvg } from '$lib/api';
import PolarControls from './PolarControls.svelte';

/**
 * A visualization the panel can display.
 *
 * `toOptions` reads the reactive control state *synchronously* and returns a
 * plain snapshot; `render` is async and takes that snapshot plus a pixel size.
 * Splitting the two keeps reactive dependency-tracking out of the async render.
 */
export interface GraphType<S = unknown, O = unknown> {
  id: string;
  label: string;
  /** Suggested file basename (no extension) for downloads. */
  fileBase: string;
  /** Type-specific controls; receives the graph's state instance. */
  Controls: Component<{ state: S }>;
  /** Creates a fresh reactive state instance for this graph. */
  createState: () => S;
  /** Synchronously snapshots the reactive options from state. */
  toOptions: (state: S) => O;
  /** Renders to SVG markup at the given square pixel size. */
  render: (options: O, size: number) => Promise<string>;
}

// --- Polar luminous-intensity diagram -------------------------------------

export const POLAR_PLANES = [
  { id: 'c0c180', label: 'C0/C180', color: '#ff6b6b' },
  { id: 'c90c270', label: 'C90/C270', color: '#7375ff' },
  { id: 'c45c225', label: 'C45/C225', color: '#2f9e62' },
  { id: 'c135c315', label: 'C135/C315', color: '#e89032' }
] as const;

export class PolarState {
  planes = $state<Record<string, boolean>>({
    c0c180: true,
    c90c270: true,
    c45c225: false,
    c135c315: false
  });
  intensityMode = $state<'stored' | 'converted'>('stored');
}

interface PolarRenderOptions {
  planes: string[];
  intensityMode: 'stored' | 'converted';
}

const polarGraph: GraphType<PolarState, PolarRenderOptions> = {
  id: 'polar',
  label: 'Polar',
  fileBase: 'polar-diagram',
  Controls: PolarControls,
  createState: () => new PolarState(),
  toOptions: (s) => ({
    planes: POLAR_PLANES.filter((p) => s.planes[p.id]).map((p) => p.id),
    intensityMode: s.intensityMode
  }),
  render: (opts, size) =>
    renderPolarSvg({
      width: size,
      height: size,
      planes: opts.planes,
      showGrid: true,
      showLegend: true,
      showAxisLabels: true,
      intensityMode: opts.intensityMode,
      presentation: 'focused',
      title: null
    })
};

/** All registered graph types, in display order. The erased `any` parameters
 *  let the host hold heterogeneous descriptors; each descriptor stays strongly
 *  typed at its own definition site. */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const GRAPHS: GraphType<any, any>[] = [polarGraph];
