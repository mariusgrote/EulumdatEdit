import { endTabPreview, moveTabPreview, startTabPreview } from './api';

export type Point = Pick<PointerEvent, 'pointerId' | 'clientX' | 'clientY' | 'screenX' | 'screenY'>;
export type Bounds = { left: number; top: number; right: number; bottom: number };
export type TabLayout = { strip: Bounds; tabs: { left: number; right: number }[] };
export type DragState = { tabId: string; pointerId: number; dragging: boolean; dropIndex: number | null };
export type DragOutcome = 'click' | 'cancelled' | 'reordered' | 'moved' | 'detached' | 'failed';

export interface TabDragWindows {
  current(): Promise<{ label: string; bounds: Bounds }>;
  others(): Promise<{ label: string; bounds: Bounds }[]>;
  closeCurrent(): Promise<void>;
}

export interface TabDragActions {
  title(tabId: string): string;
  move(tabId: string, window: string, index?: number): Promise<boolean>;
  detach(tabId: string, x: number, y: number): Promise<boolean>;
  hasTabs(): boolean;
  reportError(message: string): void;
}

type Preview = { move(): void; end(): void };
type Drag = DragState & { startX: number; startY: number; clientX: number; screenX: number; screenY: number };

const contains = (bounds: Bounds, x: number, y: number) =>
  x >= bounds.left && x <= bounds.right && y >= bounds.top && y <= bounds.bottom;

function insertionIndex(x: number, tabs: TabLayout['tabs']) {
  const index = tabs.findIndex((tab) => x < (tab.left + tab.right) / 2);
  return index < 0 ? tabs.length : index;
}

// Serialize native preview creation and destruction across gestures.
let previewLifecycle: Promise<void> = Promise.resolve();

class NativePreview implements Preview {
  private readonly id = crypto.randomUUID();
  private readonly started: Promise<void>;
  private active = true;
  private ready = false;
  private moving = false;
  private pendingMove = false;
  private timer: ReturnType<typeof setInterval> | null = null;

  constructor(title: string, private readonly reportError: (message: string) => void) {
    this.started = previewLifecycle.then(() => startTabPreview(this.id, title));
    previewLifecycle = this.started.catch(() => {});
    void this.started.then(() => {
      if (!this.active) return;
      this.ready = true;
      this.move();
      this.timer = setInterval(() => this.move(), 30);
    }).catch((error: unknown) => {
      if (this.active) this.reportError(`Could not show tab drag preview: ${String(error)}`);
    });
  }

  move() {
    if (!this.active || !this.ready) return;
    this.pendingMove = true;
    if (!this.moving) void this.flushMoves();
  }

  end() {
    if (!this.active) return;
    this.active = false;
    this.pendingMove = false;
    if (this.timer) clearInterval(this.timer);
    const cleanup = this.started.then(() => endTabPreview(this.id), () => {});
    previewLifecycle = cleanup.catch(() => {});
    void cleanup.catch((error: unknown) => {
      this.reportError(`Could not close tab drag preview: ${String(error)}`);
    });
  }

  private async flushMoves() {
    this.moving = true;
    while (this.active && this.pendingMove) {
      this.pendingMove = false;
      try {
        await moveTabPreview(this.id);
      } catch (error) {
        this.reportError(`Could not move tab drag preview: ${String(error)}`);
        this.end();
      }
    }
    this.moving = false;
  }
}

export class TabDragFlow {
  private drag: Drag | null = null;
  private preview: Preview | null = null;

  constructor(
    private readonly actions: TabDragActions,
    private readonly windows: TabDragWindows,
    private readonly createPreview: (title: string, reportError: (message: string) => void) => Preview =
      (title, reportError) => new NativePreview(title, reportError)
  ) {}

  get state(): DragState | null {
    if (!this.drag) return null;
    const { tabId, pointerId, dragging, dropIndex } = this.drag;
    return { tabId, pointerId, dragging, dropIndex };
  }

  start(tabId: string, point: Point) {
    this.cancel();
    this.drag = { tabId, pointerId: point.pointerId, startX: point.clientX, startY: point.clientY,
      clientX: point.clientX, screenX: point.screenX, screenY: point.screenY,
      dragging: false, dropIndex: null };
  }

  move(point: Point, layout: TabLayout): DragState | null {
    const drag = this.drag;
    if (!drag || point.pointerId !== drag.pointerId) return this.state;
    drag.clientX = point.clientX;
    drag.screenX = point.screenX;
    drag.screenY = point.screenY;
    drag.dragging ||= Math.hypot(point.clientX - drag.startX, point.clientY - drag.startY) >= 5;
    if (drag.dragging) {
      if (!this.preview) this.preview = this.createPreview(this.actions.title(drag.tabId), this.actions.reportError);
      else this.preview.move();
      drag.dropIndex = contains(layout.strip, point.clientX, point.clientY)
        ? insertionIndex(point.clientX, layout.tabs) : null;
    }
    return this.state;
  }

  cancel() {
    this.preview?.end();
    this.preview = null;
    this.drag = null;
  }

  async finish(point: Point, layout: TabLayout): Promise<DragOutcome> {
    if (!this.drag || point.pointerId !== this.drag.pointerId) return 'cancelled';
    this.move(point, layout);
    const drag = this.drag!;
    this.cancel();
    if (!drag.dragging) return 'click';
    try {
      const current = await this.windows.current();
      if (contains(current.bounds, drag.screenX, drag.screenY)) {
        const index = drag.dropIndex ?? insertionIndex(drag.clientX, layout.tabs);
        return await this.actions.move(drag.tabId, current.label, index) ? 'reordered' : 'failed';
      }
      const target = (await this.windows.others()).find((window) =>
        window.label !== current.label && !window.label.startsWith('tab-preview-') &&
        contains(window.bounds, drag.screenX, drag.screenY));
      const succeeded = target
        ? await this.actions.move(drag.tabId, target.label)
        : await this.actions.detach(drag.tabId, drag.screenX - 180, drag.screenY - 18);
      if (!succeeded) return 'failed';
      if (!this.actions.hasTabs()) await this.windows.closeCurrent();
      return target ? 'moved' : 'detached';
    } catch (error) {
      this.actions.reportError(`Could not finish tab drag: ${String(error)}`);
      return 'failed';
    }
  }
}
