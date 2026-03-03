import type { HexRow } from "../api";

interface HexViewerProps {
  rows: HexRow[];
  totalBytes: number;
  /** currently visible byte range for the status bar */
  rangeStart: number;
  rangeEnd: number;
}

export default function HexViewer({
  rows,
  totalBytes,
  rangeStart,
  rangeEnd,
}: HexViewerProps) {
  if (rows.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center text-gray-600">
        <p className="text-sm">Open a file to view its contents</p>
      </div>
    );
  }

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Column headers */}
      <div className="flex gap-0 border-b border-gray-800 bg-gray-900/60 px-4 py-1 text-[11px] text-gray-500 select-none">
        <span className="w-[76px] shrink-0">Offset</span>
        <span className="flex-1">
          {Array.from({ length: 16 }, (_, i) => (
            <span key={i} className="inline-block w-[28px] text-center">
              {i.toString(16).toUpperCase().padStart(2, "0")}
            </span>
          ))}
        </span>
        <span className="w-[160px] shrink-0 pl-3">ASCII</span>
      </div>

      {/* Hex rows */}
      <div className="flex-1 overflow-y-auto px-4 py-1">
        {rows.map((row, idx) => (
          <div
            key={idx}
            className="flex gap-0 leading-6 hover:bg-gray-900/50"
          >
            {/* Offset */}
            <span className="w-[76px] shrink-0 text-emerald-600 text-xs select-none">
              {row.offset}
            </span>

            {/* Hex bytes */}
            <span className="flex-1 text-xs">
              {row.hex.map((byte, bi) => (
                <span
                  key={bi}
                  className="inline-block w-[28px] text-center text-gray-300 hover:text-white hover:bg-gray-800 rounded cursor-default transition-colors"
                >
                  {byte}
                </span>
              ))}
              {/* Pad remaining cells if row < 16 bytes */}
              {row.hex.length < 16 &&
                Array.from({ length: 16 - row.hex.length }, (_, pi) => (
                  <span
                    key={`pad-${pi}`}
                    className="inline-block w-[28px] text-center"
                  />
                ))}
            </span>

            {/* ASCII */}
            <span className="w-[160px] shrink-0 pl-3 text-xs text-amber-400/70 select-text tracking-wider">
              {row.ascii}
            </span>
          </div>
        ))}
      </div>

      {/* Status bar */}
      <div className="border-t border-gray-800 bg-gray-900/60 px-4 py-1 text-[11px] text-gray-500 flex justify-between select-none">
        <span>
          Showing 0x{rangeStart.toString(16).toUpperCase()} – 0x
          {rangeEnd.toString(16).toUpperCase()}
        </span>
        <span>{totalBytes.toLocaleString()} bytes total</span>
      </div>
    </div>
  );
}
