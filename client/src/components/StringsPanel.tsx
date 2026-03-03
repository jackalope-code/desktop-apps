import { useState } from "react";
import { scanStrings, type StringMatch } from "../api";

interface StringsPanelProps {
  filePath: string;
  onGoToOffset: (offset: number) => void;
  onClose: () => void;
}

export default function StringsPanel({
  filePath,
  onGoToOffset,
  onClose,
}: StringsPanelProps) {
  const [minLength, setMinLength] = useState(4);
  const [results, setResults] = useState<StringMatch[] | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState("");

  async function handleScan() {
    setLoading(true);
    setError(null);
    try {
      const matches = await scanStrings(filePath, minLength);
      setResults(matches);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  const filtered = results
    ? results.filter((m) =>
        filter === "" || m.value.toLowerCase().includes(filter.toLowerCase())
      )
    : null;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-16">
      <div className="flex flex-col w-full max-w-2xl bg-gray-900 border border-gray-700 rounded-lg shadow-2xl overflow-hidden max-h-[80vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-700">
          <h2 className="text-sm font-semibold text-emerald-400">
            String Scanner
          </h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-100 text-xl leading-none"
          >
            ×
          </button>
        </div>

        {/* Controls */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-800 bg-gray-950/50">
          <label className="text-xs text-gray-400 whitespace-nowrap">
            Min length
          </label>
          <input
            type="number"
            min={1}
            value={minLength}
            onChange={(e) => setMinLength(Math.max(1, Number(e.target.value)))}
            className="w-16 rounded bg-gray-800 border border-gray-700 px-2 py-1 text-xs text-gray-100 focus:outline-none focus:border-emerald-500"
          />
          <button
            onClick={handleScan}
            disabled={loading}
            className="rounded bg-emerald-600 px-3 py-1 text-xs font-medium hover:bg-emerald-500 disabled:opacity-50 transition-colors"
          >
            {loading ? "Scanning…" : "Scan"}
          </button>

          {results !== null && (
            <>
              <input
                type="text"
                placeholder="Filter strings…"
                value={filter}
                onChange={(e) => setFilter(e.target.value)}
                className="flex-1 rounded bg-gray-800 border border-gray-700 px-2 py-1 text-xs text-gray-100 placeholder-gray-500 focus:outline-none focus:border-emerald-500"
              />
              <span className="text-xs text-gray-500 whitespace-nowrap">
                {filtered?.length ?? 0} / {results.length}
              </span>
            </>
          )}
        </div>

        {/* Error */}
        {error && (
          <div className="px-4 py-2 text-xs text-red-300 bg-red-900/40 border-b border-red-800">
            {error}
          </div>
        )}

        {/* Results */}
        <div className="flex-1 overflow-y-auto font-mono text-xs">
          {filtered === null && !loading && (
            <div className="px-4 py-6 text-center text-gray-500">
              Set a minimum length and click Scan.
            </div>
          )}
          {filtered !== null && filtered.length === 0 && (
            <div className="px-4 py-6 text-center text-gray-500">
              No strings found.
            </div>
          )}
          {filtered !== null &&
            filtered.map((m, idx) => (
              <div
                key={idx}
                className="flex items-baseline gap-3 px-4 py-1 hover:bg-gray-800 cursor-pointer border-b border-gray-800/50"
                onClick={() => onGoToOffset(m.offset)}
                title={`Jump to offset 0x${m.offset.toString(16).toUpperCase().padStart(8, "0")}`}
              >
                <span className="text-emerald-500 shrink-0 w-24">
                  0x{m.offset.toString(16).toUpperCase().padStart(8, "0")}
                </span>
                <span className="text-gray-200 break-all">{m.value}</span>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
}
