import { useState, useCallback } from "react";
import Toolbar from "./components/Toolbar";
import HexViewer from "./components/HexViewer";
import RangeBar from "./components/RangeBar";
import WriteModal from "./components/WriteModal";
import StringsPanel from "./components/StringsPanel";
import {
  openFile,
  readBytes,
  writeOverwrite,
  writeInsert,
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

      <HexViewer
        rows={rows}
        totalBytes={totalBytes}
        rangeStart={rangeStart}
        rangeEnd={rangeEnd}
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
