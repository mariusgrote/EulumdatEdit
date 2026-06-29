import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';

/** Installs the app menu so macOS routes Cmd+W to close-document, not window-close. */
export async function setupAppMenu(handlers: {
  onNew: () => void;
  onOpen: () => void;
  onClose: () => void;
}): Promise<void> {
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
