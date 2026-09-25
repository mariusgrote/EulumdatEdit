import { endTabPreview, moveTabPreview, startTabPreview } from './api';

// Finish the old preview's cleanup before starting a later drag.
let lifecycle: Promise<void> = Promise.resolve();

export class TabDragPreview {
  private readonly id = crypto.randomUUID();
  private readonly started: Promise<void>;
  private active = true;
  private ready = false;
  private moving = false;
  private pendingMove = false;
  private timer: ReturnType<typeof setInterval> | null = null;

  constructor(title: string, private readonly reportError: (message: string) => void) {
    this.started = lifecycle.then(() => startTabPreview(this.id, title));
    lifecycle = this.started.catch(() => {});
    void this.started.then(() => {
      this.ready = true;
      if (this.active) {
        this.move();
        this.timer = setInterval(() => this.move(), 30);
      }
    }).catch((error: unknown) => {
      console.error('Could not show tab drag preview', error);
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
    this.timer = null;
    const cleanup = this.started.then(() => endTabPreview(this.id), () => {});
    lifecycle = cleanup.catch(() => {});
    void cleanup.catch((error: unknown) => {
      console.error('Could not close tab drag preview', error);
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
        console.error('Could not move tab drag preview', error);
        this.reportError(`Could not move tab drag preview: ${String(error)}`);
        this.end();
      }
    }
    this.moving = false;
  }
}
