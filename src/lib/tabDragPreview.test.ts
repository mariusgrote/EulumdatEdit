import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as api from './api';
import { TabDragPreview } from './tabDragPreview';

vi.mock('./api', () => ({
  startTabPreview: vi.fn(),
  moveTabPreview: vi.fn(),
  endTabPreview: vi.fn()
}));

const settle = async () => {
  for (let i = 0; i < 5; i++) await Promise.resolve();
};

describe('native tab preview lifecycle', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.startTabPreview).mockResolvedValue();
    vi.mocked(api.moveTabPreview).mockResolvedValue();
    vi.mocked(api.endTabPreview).mockResolvedValue();
  });

  it('closes a preview when Escape ends the drag before creation finishes', async () => {
    let finishStart!: () => void;
    vi.mocked(api.startTabPreview).mockImplementation(() => new Promise((resolve) => {
      finishStart = resolve;
    }));
    const preview = new TabDragPreview('a.ldt', vi.fn());
    await settle();
    preview.end();
    expect(api.endTabPreview).not.toHaveBeenCalled();
    finishStart();
    await settle();
    expect(api.endTabPreview).toHaveBeenCalledWith(vi.mocked(api.startTabPreview).mock.calls[0][0]);
  });

  it('waits for old cleanup before creating another preview', async () => {
    let finishEnd!: () => void;
    vi.mocked(api.endTabPreview).mockImplementationOnce(() => new Promise((resolve) => {
      finishEnd = resolve;
    }));
    const first = new TabDragPreview('first.ldt', vi.fn());
    await settle();
    first.end();
    await settle();
    const second = new TabDragPreview('second.ldt', vi.fn());
    await settle();
    expect(api.startTabPreview).toHaveBeenCalledTimes(1);
    finishEnd();
    await settle();
    expect(api.startTabPreview).toHaveBeenCalledTimes(2);
    second.end();
    await settle();
  });

  it('removes the preview when a native move fails', async () => {
    const log = vi.spyOn(console, 'error').mockImplementation(() => {});
    vi.mocked(api.moveTabPreview).mockRejectedValueOnce(new Error('window was closed'));
    const reportError = vi.fn();
    const preview = new TabDragPreview('a.ldt', reportError);
    await settle();
    expect(reportError).toHaveBeenCalledWith(expect.stringContaining('window was closed'));
    expect(api.endTabPreview).toHaveBeenCalledWith(vi.mocked(api.startTabPreview).mock.calls[0][0]);
    preview.end();
    log.mockRestore();
  });
});
