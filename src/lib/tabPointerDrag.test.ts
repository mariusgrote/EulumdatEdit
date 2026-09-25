import { describe, expect, it } from 'vitest';
import { TabPointerDragSession } from './tabPointerDrag';

function point(clientX: number, clientY: number, screenX = clientX + 100): PointerEvent {
  return { pointerId: 1, clientX, clientY, screenX, screenY: clientY + 100 } as PointerEvent;
}

describe('tab pointer drag', () => {
  it('keeps a click below the movement threshold from moving a tab', () => {
    const session = new TabPointerDragSession();
    session.start('tab-1', point(20, 20));
    expect(session.finish(point(23, 23))).toBeNull();
  });

  it('uses the release coordinates after crossing the threshold, including outside the window', () => {
    const session = new TabPointerDragSession();
    session.start('tab-1', point(20, 20));
    session.move(point(26, 20));
    expect(session.finish(point(-100, 40, 0))).toMatchObject({
      tabId: 'tab-1', screenX: 0, screenY: 140, dragging: true
    });
  });

  it.each(['Escape', 'pointercancel'])('does not commit after %s', () => {
    const session = new TabPointerDragSession();
    session.start('tab-1', point(20, 20));
    session.move(point(30, 20));
    session.cancel();
    expect(session.finish(point(40, 20))).toBeNull();
  });

  it('ignores a release from another pointer', () => {
    const session = new TabPointerDragSession();
    session.start('tab-1', point(20, 20));
    session.move(point(30, 20));
    expect(session.finish({ ...point(40, 20), pointerId: 2 })).toBeNull();
    expect(session.current?.dragging).toBe(true);
  });
});
