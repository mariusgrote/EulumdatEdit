import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';

/** Whether we're running on macOS, where the global menu bar lives at the top
 *  of the screen and Cmd+W is routed through the menu rather than the webview. */
function isMacOS(): boolean {
  if (typeof navigator === 'undefined') return false;
  return /Mac/i.test(navigator.userAgent);
}

/** Installs the app menu so macOS routes Cmd+W to close-document, not
 *  window-close.
 *
 *  macOS only: on Windows/Linux the menu would be rendered as a per-window
 *  menu bar, which clashes with the app's custom title bar. Those platforms
 *  rely on the webview keyboard handler for the same shortcuts instead. */
export async function setupAppMenu(handlers: {
  onNew: () => void;
  onOpen: () => void;
  onClose: () => void;
}): Promise<void> {
  if (!isMacOS()) return;

  const appMenu = await Submenu.new({
    text: 'EulumdatEdit',
    items: [
      await PredefinedMenuItem.new({ item: { About: { name: 'EulumdatEdit' } } }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Hide' }),
      await PredefinedMenuItem.new({ item: 'HideOthers' }),
      await PredefinedMenuItem.new({ item: 'ShowAll' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Quit' })
    ]
  });

  const fileMenu = await Submenu.new({
    text: 'File',
    items: [
      await MenuItem.new({
        text: 'New',
        accelerator: 'CmdOrCtrl+N',
        action: handlers.onNew
      }),
      await MenuItem.new({
        text: 'Open…',
        accelerator: 'CmdOrCtrl+O',
        action: handlers.onOpen
      }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        text: 'Close',
        accelerator: 'CmdOrCtrl+W',
        action: handlers.onClose
      }),
      await PredefinedMenuItem.new({ text: 'Close Window', item: 'CloseWindow' })
    ]
  });

  const editMenu = await Submenu.new({
    text: 'Edit',
    items: [
      await PredefinedMenuItem.new({ item: 'Undo' }),
      await PredefinedMenuItem.new({ item: 'Redo' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'Cut' }),
      await PredefinedMenuItem.new({ item: 'Copy' }),
      await PredefinedMenuItem.new({ item: 'Paste' }),
      await PredefinedMenuItem.new({ item: 'SelectAll' })
    ]
  });

  const viewMenu = await Submenu.new({
    text: 'View',
    items: [await PredefinedMenuItem.new({ item: 'Fullscreen' })]
  });

  const windowMenu = await Submenu.new({
    text: 'Window',
    items: [
      await PredefinedMenuItem.new({ item: 'Minimize' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await PredefinedMenuItem.new({ item: 'CloseWindow' })
    ]
  });

  const menu = await Menu.new({
    items: [appMenu, fileMenu, editMenu, viewMenu, windowMenu]
  });

  await menu.setAsAppMenu();
}
