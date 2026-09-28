import { afterEach, describe, expect, it, vi } from 'vitest';
import * as api from './api';
import { TabDragFlow, type Point, type TabLayout, type TabDragActions, type TabDragWindows } from './tabDragFlow';

vi.mock('./api', () => ({
  startTabPreview: vi.fn(async () => {}),
  moveTabPreview: vi.fn(async () => {}),
  endTabPreview: vi.fn(async () => {})
}));

afterEach(() => vi.clearAllMocks());

const settle = async () => {
  for (let i = 0; i < 8; i++) await Promise.resolve();
};

const layout: TabLayout = {
  strip: { left: 0, top: 0, right: 300, bottom: 40 },
  tabs: [{ left: 0, right: 100 }, { left: 100, right: 200 }, { left: 200, right: 300 }]
};
const point = (clientX: number, clientY = 20, screenX = clientX + 100, screenY = clientY + 100): Point =>
  ({ pointerId: 1, clientX, clientY, screenX, screenY });

function setup() {
  let hasTabs = true;
  const actions: TabDragActions = {
    title: vi.fn(() => 'a.ldt'),
    move: vi.fn(async () => true),
    detach: vi.fn(async () => true),
    hasTabs: () => hasTabs,
    reportError: vi.fn()
  };
  const windows: TabDragWindows = {
    current: vi.fn(async () => ({ label: 'main', bounds: { left: 100, top: 100, right: 500, bottom: 500 } })),
    others: vi.fn(async () => [
      { label: 'tab-preview-1', bounds: { left: 600, top: 100, right: 900, bottom: 500 } },
      { label: 'doc-2', bounds: { left: 600, top: 100, right: 900, bottom: 500 } }
    ]),
    closeCurrent: vi.fn(async () => {})
  };
  const preview = { move: vi.fn(), end: vi.fn() };
  const flow = new TabDragFlow(actions, windows, () => preview);
  return { flow, actions, windows, preview, empty: () => { hasTabs = false; } };
}

describe('tab drag decisions', () => {
  it('leaves a click below the threshold alone', async () => {
    const { flow, actions, windows, preview } = setup();
    flow.start('a', point(20));
    expect(await flow.finish(point(23, 23), layout)).toBe('click');
    expect(actions.move).not.toHaveBeenCalled();
    expect(windows.current).not.toHaveBeenCalled();
    expect(preview.end).not.toHaveBeenCalled();
  });

  it('reorders at the release midpoint and cleans up the preview', async () => {
    const { flow, actions, preview } = setup();
    flow.start('a', point(20));
    flow.move(point(26), layout);
    expect(flow.state).toMatchObject({ dragging: true, dropIndex: 0 });
    expect(await flow.finish(point(170), layout)).toBe('reordered');
    expect(actions.move).toHaveBeenCalledWith('a', 'main', 2);
    expect(preview.end).toHaveBeenCalledOnce();
    expect(flow.state).toBeNull();
  });

  it('moves into another document window and closes an empty source', async () => {
    const { flow, actions, windows, empty } = setup();
    empty();
    flow.start('a', point(20));
    flow.move(point(30), layout);
    expect(await flow.finish(point(700, 20, 700, 200), layout)).toBe('moved');
    expect(actions.move).toHaveBeenCalledWith('a', 'doc-2');
    expect(windows.closeCurrent).toHaveBeenCalledOnce();
  });

  it('detaches at the scaled screen point and keeps a nonempty source open', async () => {
    const { flow, actions, windows } = setup();
    flow.start('a', point(20));
    expect(await flow.finish(point(-100, 40, 0, 700), layout)).toBe('detached');
    expect(actions.detach).toHaveBeenCalledWith('a', -180, 682);
    expect(windows.closeCurrent).not.toHaveBeenCalled();
  });

  it('does not close the source after a failed move', async () => {
    const { flow, actions, windows, empty, preview } = setup();
    empty();
    vi.mocked(actions.move).mockResolvedValue(false);
    flow.start('a', point(20));
    expect(await flow.finish(point(700, 20, 700, 200), layout)).toBe('failed');
    expect(windows.closeCurrent).not.toHaveBeenCalled();
    expect(preview.end).toHaveBeenCalledOnce();
  });

  it('reports failed window queries after ending the preview', async () => {
    const { flow, actions, windows, preview } = setup();
    vi.mocked(windows.current).mockRejectedValue(new Error('window closed'));
    flow.start('a', point(20));
    expect(await flow.finish(point(30), layout)).toBe('failed');
    expect(preview.end).toHaveBeenCalledOnce();
    expect(actions.reportError).toHaveBeenCalledWith(expect.stringContaining('window closed'));
  });

  it('cancels on Escape, pointercancel, or lost capture', async () => {
    for (const reason of ['Escape', 'pointercancel', 'lostpointercapture']) {
      const { flow, actions, preview } = setup();
      flow.start('a', point(20));
      flow.move(point(30), layout);
      flow.cancel();
      expect(await flow.finish(point(700, 20, 700, 200), layout), reason).toBe('cancelled');
      expect(actions.move).not.toHaveBeenCalled();
      expect(actions.detach).not.toHaveBeenCalled();
      expect(preview.end).toHaveBeenCalledOnce();
    }
  });

  it('ignores another pointer', async () => {
    const { flow, actions } = setup();
    flow.start('a', point(20));
    expect(await flow.finish({ ...point(40), pointerId: 2 }, layout)).toBe('cancelled');
    expect(flow.state).not.toBeNull();
    expect(actions.move).not.toHaveBeenCalled();
  });

  it('ends a native preview even when cancellation precedes its creation', async () => {
    let releaseStart!: () => void;
    vi.mocked(api.startTabPreview).mockImplementationOnce(() => new Promise((resolve) => { releaseStart = resolve; }));
    const { actions, windows } = setup();
    const flow = new TabDragFlow(actions, windows);
    flow.start('a', point(20));
    flow.move(point(30), layout);
    await settle();
    flow.cancel();
    expect(api.endTabPreview).not.toHaveBeenCalled();
    releaseStart();
    await settle();
    expect(api.endTabPreview).toHaveBeenCalledWith(vi.mocked(api.startTabPreview).mock.calls.at(-1)![0]);
  });

  it('waits for previous native preview cleanup before starting the next one', async () => {
    let releaseEnd!: () => void;
    vi.mocked(api.endTabPreview).mockImplementationOnce(() => new Promise((resolve) => { releaseEnd = resolve; }));
    const { actions, windows } = setup();
    const flow = new TabDragFlow(actions, windows);
    flow.start('a', point(20));
    flow.move(point(30), layout);
    await settle();
    flow.cancel();
    await settle();
    flow.start('b', point(20));
    flow.move(point(30), layout);
    await settle();
    expect(api.startTabPreview).toHaveBeenCalledTimes(1);
    releaseEnd();
    await settle();
    expect(api.startTabPreview).toHaveBeenCalledTimes(2);
    flow.cancel();
    await settle();
  });
});
