export interface TabPointerDrag {
  tabId: string;
  pointerId: number;
  startX: number;
  startY: number;
  clientX: number;
  clientY: number;
  screenX: number;
  screenY: number;
  dragging: boolean;
}

export class TabPointerDragSession {
  current: TabPointerDrag | null = null;

  start(tabId: string, event: PointerEvent) {
    this.current = {
      tabId, pointerId: event.pointerId, startX: event.clientX, startY: event.clientY,
      clientX: event.clientX, clientY: event.clientY,
      screenX: event.screenX, screenY: event.screenY, dragging: false
    };
  }

  move(event: PointerEvent): TabPointerDrag | null {
    const drag = this.current;
    if (!drag || event.pointerId !== drag.pointerId) return drag;
    this.current = {
      ...drag,
      clientX: event.clientX,
      clientY: event.clientY,
      screenX: event.screenX,
      screenY: event.screenY,
      dragging: drag.dragging || Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) >= 5
    };
    return this.current;
  }

  cancel() {
    this.current = null;
  }

  finish(event: PointerEvent): TabPointerDrag | null {
    if (!this.current || event.pointerId !== this.current.pointerId) return null;
    const drag = this.move(event);
    this.cancel();
    return drag?.dragging ? drag : null;
  }
}
