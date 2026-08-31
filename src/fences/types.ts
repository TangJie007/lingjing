export type FenceGroupKey =
  | "app"
  | "image"
  | "document"
  | "folder"
  | "media"
  | "archive";

export interface DesktopItem {
  path: string;
  name: string;
  kind?: string;
  isDir?: boolean;
  builtin?: boolean;
  icon?: string | null;
}

export interface ShellMenuEntry {
  id?: number;
  label?: string;
  icon?: string | null;
  disabled?: boolean;
  separator?: boolean;
  children?: ShellMenuEntry[];
  menuPath?: number[];
  loading?: boolean;
  /** Win11 top icon-strip action */
  pin?: boolean;
  /** Destructive action (delete) */
  destructive?: boolean;
}

export interface FenceGroupState {
  key: FenceGroupKey;
  orderKey: string;
  items: DesktopItem[];
  emptyText: string;
  native: boolean;
  compact?: boolean;
  title?: string;
}

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T = unknown>(
          cmd: string,
          args?: Record<string, unknown>,
        ) => Promise<T>;
      };
      event: {
        listen: <T = unknown>(
          event: string,
          handler: (e: { payload: T }) => void,
        ) => Promise<() => void>;
      };
    };
    __fenceApply?: (items: DesktopItem[]) => void;
  }
}

export {};
