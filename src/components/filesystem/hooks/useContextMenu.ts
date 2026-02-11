import { clipboard } from '$lib/stores/clipboardStore';

export interface MenuOption {
  label: string;
  action: string;
  danger?: boolean;
  disabled?: boolean;
  shortcut?: string;
}

export function getMenuOptions(
  menuTargetFile: string | null,
  files: any[]
): MenuOption[] {
  const clipboardState = clipboard.getState();
  const hasClipboard = clipboardState.files.length > 0;

  if (menuTargetFile) {
    const targetFile = files.find(f => f.name === menuTargetFile);

    if (targetFile?.is_dir) {
      return [
        { label: 'Open', action: 'open' },
        { label: 'Open in New Tab', action: 'open_new_tab' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Cut', action: 'cut', shortcut: 'Ctrl+X' },
        { label: 'Copy', action: 'copy', shortcut: 'Ctrl+C' },
        { label: 'Paste Into', action: 'paste_into', disabled: !hasClipboard, shortcut: 'Ctrl+V' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Rename', action: 'rename', shortcut: 'F2' },
        { label: 'Delete', action: 'delete', danger: true, shortcut: 'Del' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Details', action: 'details' },
      ];
    } else {
      return [
        { label: 'Open', action: 'open' },
        { label: 'Open in Editor', action: 'open_in_editor' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Cut', action: 'cut', shortcut: 'Ctrl+X' },
        { label: 'Copy', action: 'copy', shortcut: 'Ctrl+C' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Rename', action: 'rename', shortcut: 'F2' },
        { label: 'Delete', action: 'delete', danger: true, shortcut: 'Del' },
        { label: 'SEPARATOR', action: '' },
        { label: 'Details', action: 'details' },
      ];
    }
  } else {
    return [
      { label: 'New Folder', action: 'new_folder' },
      { label: 'New File', action: 'new_file' },
      { label: 'SEPARATOR', action: '' },
      { label: 'Paste', action: 'paste', disabled: !hasClipboard, shortcut: 'Ctrl+V' },
      { label: 'SEPARATOR', action: '' },
      { label: 'Refresh', action: 'refresh', shortcut: 'F5' },
    ];
  }
}