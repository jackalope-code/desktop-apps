import { open } from "@tauri-apps/plugin-dialog";

interface ToolbarProps {
  fileInfo: { path: string; size: number; file_type: string } | null;
  onFileOpened: (path: string) => void;
  onWriteClick: () => void;
  onInsertClick: () => void;
  onStringsClick: () => void;
}

export default function Toolbar({
  fileInfo,
  onFileOpened,
  onWriteClick,
  onInsertClick,
  onStringsClick,
}: ToolbarProps) {
  async function handleOpen() {
    const selected = await open({
      multiple: false,
      title: "Open Binary File",
    });
    if (selected) {
      onFileOpened(selected as string);
    }
  }

  return (
    <header className="flex items-center gap-3 border-b border-gray-800 bg-gray-900 px-4 py-2">
      {/* Logo / title */}
      <h1 className="text-lg font-bold text-emerald-400 mr-4 select-none">
        BinRW
      </h1>

      {/* Actions */}
      <button
        onClick={handleOpen}
        className="rounded bg-emerald-600 px-3 py-1 text-sm font-medium hover:bg-emerald-500 transition-colors"
      >
        Open File
      </button>

      {fileInfo && (
        <>
          <button
            onClick={onWriteClick}
            className="rounded bg-blue-600 px-3 py-1 text-sm font-medium hover:bg-blue-500 transition-colors"
          >
            Overwrite
          </button>
          <button
            onClick={onInsertClick}
            className="rounded bg-violet-600 px-3 py-1 text-sm font-medium hover:bg-violet-500 transition-colors"
          >
            Insert
          </button>
          <button
            onClick={onStringsClick}
            className="rounded bg-amber-600 px-3 py-1 text-sm font-medium hover:bg-amber-500 transition-colors"
          >
            Strings
          </button>
        </>
      )}

      {/* File info */}
      {fileInfo && (
        <div className="ml-auto flex items-center gap-4 text-xs text-gray-400">
          <span
            className="max-w-[350px] truncate"
            title={fileInfo.path}
          >
            {fileInfo.path}
          </span>
          <span className="whitespace-nowrap">
            {fileInfo.size.toLocaleString()} bytes
          </span>
          <span className="rounded bg-gray-800 px-2 py-0.5 text-emerald-300 uppercase">
            {fileInfo.file_type}
          </span>
        </div>
      )}
    </header>
  );
}
