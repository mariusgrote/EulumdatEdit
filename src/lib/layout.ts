// Below this window width the inspector no longer fits beside the editor, so it
// auto-collapses and opens as an overlay drawer instead of a third column.
export const INSPECTOR_COLLAPSE_WIDTH = 1120;

export function isNarrowLayout(width: number): boolean {
  return width < INSPECTOR_COLLAPSE_WIDTH;
}
