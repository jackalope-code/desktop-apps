import { useState, useCallback } from "react";
import Toolbar from "./components/Toolbar";
import HexEditor, { type PendingEdits } from "./components/HexEditor";
import RangeBar from "./components/RangeBar";
import WriteModal from "./components/WriteModal";
import StringsPanel from "./components/StringsPanel";
import {
  openFile,
  readBytes,
  writeOverwrite,
  writeInsert,
  patchBytes,
  type FileInfo,
  type HexRow,
} from "./api";

const PAGE_SIZE = 512; // bytes per page view

function App() {
  const [fileInfo, setFileInfo] = useState<FileInfo | null>(null);
  const [rows, setRows] = useState<HexRow[]>([]);
  const [rangeStart, setRangeStart] = useState(0);
  const [rangeEnd, setRangeEnd] = useState(PAGE_SIZE - 1);
  const [totalBytes, setTotalBytes] = useState(0);
  const [modal, setModal] = useState<"overwrite" | "insert" | null>(null);
  const [showStrings, setShowStrings] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingEdits, setPendingEdits] = useState<PendingEdits>(new Map());

  // Load a page of hex data
  const loadRange = useCallback(
    async (path: string, start: number, end: number) => {
      try {
        setError(null);
        const result = await readBytes(path, start, end);
        setRows(result.rows);
        setTotalBytes(result.total_bytes);
        setRangeStart(start);
        setRangeEnd(end);
      } catch (e) {
        setError(String(e));
      }
    },
    []
  );

  // Open file handler
  async function handleFileOpened(path: string) {
    try {
      setError(null);
      const info = await openFile(path);
      setFileInfo(info);
      const end = Math.min(PAGE_SIZE - 1, info.size - 1);
      await loadRange(path, 0, end);
    } catch (e) {
      setError(String(e));
    }
  }

  // Navigate to a byte offset (loads page around it)
  function handleGoToOffset(offset: number) {
    if (!fileInfo) return;
    const start = Math.max(0, offset);
    const end = Math.min(start + PAGE_SIZE - 1, fileInfo.size - 1);
    loadRange(fileInfo.path, start, end);
  }

  // Record a single byte edit in the pending map
  function handleCellEdit(absoluteOffset: number, newByte: number) {
    setPendingEdits((prev) => {
      const next = new Map(prev);
      next.set(absoluteOffset, newByte);
      return next;
    });
  }

  // Flush all pending edits to disk (one patch_bytes call per contiguous run)
  async function handleApplyEdits() {
    if (!fileInfo || pendingEdits.size === 0) return;
    try {
      setError(null);
      // Sort offsets and batch into contiguous runs
      const offsets = Array.from(pendingEdits.keys()).sort((a, b) => a - b);
      let runStart = offsets[0];
      let runBytes: number[] = [pendingEdits.get(offsets[0])!];

      async function flushRun() {
        await patchBytes(fileInfo!.path, runStart, runBytes);
      }

      for (let i = 1; i < offsets.length; i++) {
        if (offsets[i] === offsets[i - 1] + 1) {
          runBytes.push(pendingEdits.get(offsets[i])!);
        } else {
          await flushRun();
          runStart = offsets[i];
          runBytes = [pendingEdits.get(offsets[i])!];
        }
      }
      await flushRun();

      setPendingEdits(new Map());
      // Refresh view
      const info = await openFile(fileInfo.path);
      setFileInfo(info);
      await loadRange(fileInfo.path, rangeStart, rangeEnd);
    } catch (e) {
      setError(String(e));
    }
  }

  // Discard all pending edits
  function handleDiscardEdits() {
    setPendingEdits(new Map());
  }

  // Handle write submission
  async function handleWrite(params: {
    offset: number;
    endOffset?: number;
    data: string;
    dataFile?: string;
    dataFileOffset?: number;
    appendZeroPastEof: boolean;
    reverse: boolean;
  }) {
    if (!fileInfo) return;
    try {
      setError(null);
      if (modal === "overwrite") {
        await writeOverwrite(fileInfo.path, params.offset, params.data, {
          endOffset: params.endOffset,
          dataFile: params.dataFile,
          dataFileOffset: params.dataFileOffset,
          appendZeroPastEof: params.appendZeroPastEof,
          reverse: params.reverse,
        });
      } else {
        await writeInsert(
          fileInfo.path,
          params.offset,
          params.data,
          params.reverse,
          params.dataFile,
          params.dataFileOffset
        );
      }
      // Refresh view after write — re-open to get updated size
      const info = await openFile(fileInfo.path);
      setFileInfo(info);
      await loadRange(fileInfo.path, rangeStart, rangeEnd);
      setModal(null);
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex h-screen flex-col bg-gray-950 text-gray-100">
      <Toolbar
        fileInfo={fileInfo}
        onFileOpened={handleFileOpened}
        onWriteClick={() => setModal("overwrite")}
        onInsertClick={() => setModal("insert")}
        onStringsClick={() => setShowStrings(true)}
        pendingEditsCount={pendingEdits.size}
        onApplyEdits={handleApplyEdits}
        onDiscardEdits={handleDiscardEdits}
      />

      {fileInfo && (
        <RangeBar
          rangeStart={rangeStart}
          rangeEnd={rangeEnd}
          totalBytes={totalBytes}
          onRangeChange={(s, e) => loadRange(fileInfo.path, s, e)}
          onGoToOffset={handleGoToOffset}
        />
      )}

      {error && (
        <div className="bg-red-900/60 px-4 py-1.5 text-xs text-red-300 border-b border-red-700">
          {error}
        </div>
      )}

      <HexEditor
        rows={rows}
        totalBytes={totalBytes}
        rangeStart={rangeStart}
        rangeEnd={rangeEnd}
        filePath={fileInfo?.path ?? null}
        pendingEdits={pendingEdits}
        onCellEdit={handleCellEdit}
      />

      {modal && (
        <WriteModal
          mode={modal}
          onSubmit={handleWrite}
          onClose={() => setModal(null)}
        />
      )}

      {showStrings && fileInfo && (
        <StringsPanel
          filePath={fileInfo.path}
          onGoToOffset={(offset) => {
            setShowStrings(false);
            handleGoToOffset(offset);
          }}
          onClose={() => setShowStrings(false)}
        />
      )}
    </div>
  );
}

export default App;
