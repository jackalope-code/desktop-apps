interface RangeBarProps {
  rangeStart: number;
  rangeEnd: number;
  totalBytes: number;
  onRangeChange: (start: number, end: number) => void;
  onGoToOffset: (offset: number) => void;
}

import { useState } from "react";

/** Parse a string as a number, treating 0x prefix as hex, otherwise decimal. */
function parseIntAuto(s: string): number {
  const t = s.trim();
  if (t.startsWith("0x") || t.startsWith("0X")) return parseInt(t, 16);
  if (t.startsWith("-0x") || t.startsWith("-0X")) return -parseInt(t.slice(1), 16);
  return parseInt(t, 10);
}

export default function RangeBar({
  rangeStart,
  rangeEnd,
  totalBytes,
  onRangeChange,
  onGoToOffset,
}: RangeBarProps) {
  const [startInput, setStartInput] = useState(String(rangeStart));
  const [endInput, setEndInput] = useState(String(rangeEnd));
  const [gotoInput, setGotoInput] = useState("");

  function applyRange() {
    const s = parseIntAuto(startInput);
    const e = parseIntAuto(endInput);
    if (!isNaN(s) && !isNaN(e)) {
      onRangeChange(s, e);
    }
  }

  function handleGoto() {
    const v = parseIntAuto(gotoInput);
    if (!isNaN(v)) {
      onGoToOffset(v);
    }
  }

  return (
    <div className="flex items-center gap-3 border-b border-gray-800 bg-gray-900/80 px-4 py-1.5 text-xs">
      <span className="text-gray-500">Range:</span>
      <input
        type="text"
        value={startInput}
        onChange={(e) => setStartInput(e.target.value)}
        placeholder="0 or 0x0"
        className="w-24 rounded bg-gray-800 border border-gray-700 px-2 py-0.5 text-gray-300 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
      />
      <span className="text-gray-600">–</span>
      <input
        type="text"
        value={endInput}
        onChange={(e) => setEndInput(e.target.value)}
        placeholder="0xFF or 255"
        className="w-24 rounded bg-gray-800 border border-gray-700 px-2 py-0.5 text-gray-300 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
      />
      <button
        onClick={applyRange}
        className="rounded bg-gray-700 px-2 py-0.5 text-gray-300 hover:bg-gray-600 transition-colors"
      >
        Go
      </button>

      <div className="mx-2 h-4 border-l border-gray-700" />

      <span className="text-gray-500">Goto (hex/dec):</span>
      <input
        type="text"
        value={gotoInput}
        onChange={(e) => setGotoInput(e.target.value)}
        placeholder="0x1A or 26"
        onKeyDown={(e) => e.key === "Enter" && handleGoto()}
        className="w-28 rounded bg-gray-800 border border-gray-700 px-2 py-0.5 text-gray-300 placeholder:text-gray-600 focus:border-emerald-500 focus:outline-none"
      />
      <button
        onClick={handleGoto}
        className="rounded bg-gray-700 px-2 py-0.5 text-gray-300 hover:bg-gray-600 transition-colors"
      >
        Jump
      </button>

      <span className="ml-auto text-gray-600">
        {totalBytes > 0 && `${totalBytes.toLocaleString()} bytes`}
      </span>
    </div>
  );
}
