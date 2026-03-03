import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

/** Parse a string as a number, treating 0x prefix as hex, otherwise decimal. */
function parseIntAuto(s: string): number {
  const t = s.trim();
  if (t.startsWith("0x") || t.startsWith("0X")) return parseInt(t, 16);
  if (t.startsWith("-0x") || t.startsWith("-0X")) return -parseInt(t.slice(1), 16);
  return parseInt(t, 10);
}

interface WriteModalProps {
  mode: "overwrite" | "insert";
  onSubmit: (params: {
    offset: number;
    endOffset?: number;
    data: string;
    dataFile?: string;
    dataFileOffset?: number;
    appendZeroPastEof: boolean;
    reverse: boolean;
  }) => void;
  onClose: () => void;
}

export default function WriteModal({ mode, onSubmit, onClose }: WriteModalProps) {
  const [offset, setOffset] = useState("0");
  const [endOffset, setEndOffset] = useState("");
  const [data, setData] = useState("");
  const [appendZero, setAppendZero] = useState(false);
  const [reverse, setReverse] = useState(false);
  const [useFile, setUseFile] = useState(false);
  const [dataFilePath, setDataFilePath] = useState("");
  const [dataFileOffset, setDataFileOffset] = useState("");

  async function handlePickFile() {
    const selected = await open({
      multiple: false,
      title: "Select data source file",
    });
    if (selected) {
      setDataFilePath(selected as string);
    }
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const parsed: Parameters<typeof onSubmit>[0] = {
      offset: parseIntAuto(offset),
      data: useFile ? "" : data,
      appendZeroPastEof: appendZero,
      reverse,
    };
    if (endOffset.trim() !== "") {
      parsed.endOffset = parseIntAuto(endOffset);
    }
    if (useFile && dataFilePath) {
      parsed.dataFile = dataFilePath;
      if (dataFileOffset.trim() !== "") {
        parsed.dataFileOffset = parseIntAuto(dataFileOffset);
      }
    }
    onSubmit(parsed);
  }

  const title = mode === "overwrite" ? "Overwrite Bytes" : "Insert (Splice) Bytes";
  const accent = mode === "overwrite" ? "blue" : "violet";

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60">
      <form
        onSubmit={handleSubmit}
        className="w-[400px] rounded-lg bg-gray-900 border border-gray-700 p-6 shadow-2xl"
      >
        <h2 className={`text-lg font-bold mb-4 ${mode === "overwrite" ? "text-blue-400" : "text-violet-400"}`}>{title}</h2>

        {/* Offset */}
        <label className="block mb-3">
          <span className="text-xs text-gray-400">Offset (decimal or 0x hex, negative = from end)</span>
          <input
            type="text"
            value={offset}
            onChange={(e) => setOffset(e.target.value)}
            className="mt-1 block w-full rounded bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-200 focus:border-emerald-500 focus:outline-none"
          />
        </label>

        {/* End offset (overwrite range only) */}
        {mode === "overwrite" && (
          <label className="block mb-3">
            <span className="text-xs text-gray-400">End offset (optional, decimal or 0x hex)</span>
            <input
              type="text"
              value={endOffset}
              onChange={(e) => setEndOffset(e.target.value)}
              placeholder="leave empty for single-offset"
              className="mt-1 block w-full rounded bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-200 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
            />
          </label>
        )}

        {/* Data source toggle */}
        <div className="flex items-center gap-4 mb-3">
          <span className="text-xs text-gray-400">Data source:</span>
          <label className="flex items-center gap-1.5 text-xs text-gray-400 cursor-pointer">
            <input
              type="radio"
              name="dataSource"
              checked={!useFile}
              onChange={() => setUseFile(false)}
              className="accent-emerald-500"
            />
            Text
          </label>
          <label className="flex items-center gap-1.5 text-xs text-gray-400 cursor-pointer">
            <input
              type="radio"
              name="dataSource"
              checked={useFile}
              onChange={() => setUseFile(true)}
              className="accent-emerald-500"
            />
            File
          </label>
        </div>

        {/* Data: text string or file picker */}
        {!useFile ? (
          <label className="block mb-3">
            <span className="text-xs text-gray-400">Data (text string to write)</span>
            <input
              type="text"
              value={data}
              onChange={(e) => setData(e.target.value)}
              placeholder="e.g. FOOBAR"
              className="mt-1 block w-full rounded bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-200 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
            />
          </label>
        ) : (
          <div className="mb-3">
            <span className="text-xs text-gray-400 block mb-1">Data file (read bytes from file)</span>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={dataFilePath}
                readOnly
                placeholder="No file selected"
                className="flex-1 rounded bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-200 placeholder:text-gray-600 focus:outline-none"
              />
              <button
                type="button"
                onClick={handlePickFile}
                className="rounded bg-gray-700 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-600 transition-colors"
              >
                Browse
              </button>
            </div>
            <label className="block mt-2">
              <span className="text-xs text-gray-400">File read offset (decimal or 0x hex, bytes to skip)</span>
              <input
                type="text"
                value={dataFileOffset}
                onChange={(e) => setDataFileOffset(e.target.value)}
                placeholder="0 or 0x0"
                className="mt-1 block w-full rounded bg-gray-800 border border-gray-700 px-3 py-1.5 text-sm text-gray-200 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
              />
            </label>
          </div>
        )}

        {/* Flags */}
        <div className="flex items-center gap-6 mb-5">
          {mode === "overwrite" && (
            <label className="flex items-center gap-2 text-xs text-gray-400 cursor-pointer">
              <input
                type="checkbox"
                checked={appendZero}
                onChange={(e) => setAppendZero(e.target.checked)}
                className="accent-emerald-500"
              />
              Append-zero past EOF
            </label>
          )}
          <label className="flex items-center gap-2 text-xs text-gray-400 cursor-pointer">
            <input
              type="checkbox"
              checked={reverse}
              onChange={(e) => setReverse(e.target.checked)}
              className="accent-emerald-500"
            />
            Reverse
          </label>
        </div>

        {/* Buttons */}
        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="rounded px-4 py-1.5 text-sm text-gray-400 hover:text-gray-200 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            className={`rounded px-4 py-1.5 text-sm font-medium transition-colors ${mode === "overwrite" ? "bg-blue-600 hover:bg-blue-500" : "bg-violet-600 hover:bg-violet-500"}`}
          >
            {mode === "overwrite" ? "Overwrite" : "Insert"}
          </button>
        </div>
      </form>
    </div>
  );
}
