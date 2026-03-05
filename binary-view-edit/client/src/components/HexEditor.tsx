import { useRef, useState, useEffect } from "react";
import type { HexRow } from "../api";

/** Map of absolute byte offset → new byte value (0–255) for pending edits. */
export type PendingEdits = Map<number, number>;

interface HexEditorProps {
  rows: HexRow[];
  totalBytes: number;
  rangeStart: number;
  rangeEnd: number;
  filePath: string | null;
  pendingEdits: PendingEdits;
  onCellEdit: (absoluteOffset: number, newByte: number) => void;
}

export default function HexEditor({
  rows,
  totalBytes,
  rangeStart,
  rangeEnd,
  filePath,
  pendingEdits,
  onCellEdit,
}: HexEditorProps) {
  const [editingOffset, setEditingOffset] = useState<number | null>(null);
  const [editValue, setEditValue] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input whenever we enter edit mode for a cell
  useEffect(() => {
    if (editingOffset !== null) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editingOffset]);

  function startEdit(absoluteOffset: number) {
    const pending = pendingEdits.get(absoluteOffset);
    setEditValue(
      pending !== undefined
        ? pending.toString(16).padStart(2, "0").toUpperCase()
        : ""
    );
    setEditingOffset(absoluteOffset);
  }

  function commitEdit(off: number, val: string) {
    const trimmed = val.trim();
    if (trimmed.length > 0) {
      const parsed = parseInt(trimmed, 16);
      if (!isNaN(parsed) && parsed >= 0 && parsed <= 255) {
        onCellEdit(off, parsed);
      }
    }
    setEditingOffset(null);
    setEditValue("");
  }

  function cancelEdit() {
    setEditingOffset(null);
    setEditValue("");
  }

  /** Advance to the next hex cell after committing, if it exists. */
  function advanceToNext(absoluteOffset: number) {
    const nextOffset = absoluteOffset + 1;
    // Check if next offset is in view
    const inRange = rows.some((row) => {
      const base = parseInt(row.offset, 16);
      return nextOffset >= base && nextOffset < base + row.hex.length;
    });
    if (inRange) {
      startEdit(nextOffset);
    } else {
      setEditingOffset(null);
      setEditValue("");
    }
  }

  function handleKeyDown(
    e: React.KeyboardEvent<HTMLInputElement>,
    absoluteOffset: number
  ) {
    if (e.key === "Escape") {
      cancelEdit();
    } else if (e.key === "Tab") {
      e.preventDefault();
      commitEdit(absoluteOffset, editValue);
      advanceToNext(absoluteOffset);
    } else if (e.key === "Enter") {
      e.preventDefault();
      commitEdit(absoluteOffset, editValue);
    }
  }

  function handleInputChange(val: string, absoluteOffset: number) {
    // Strip non-hex chars, limit to 2 digits
    const filtered = val.replace(/[^0-9a-fA-F]/g, "").slice(0, 2) .toUpperCase();
    setEditValue(filtered);
    // Auto-commit and advance after 2 hex digits
    if (filtered.length === 2) {
      const parsed = parseInt(filtered, 16);
      onCellEdit(absoluteOffset, parsed);
      advanceToNext(absoluteOffset);
    }
  }

  if (rows.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center text-gray-600">
        <p className="text-sm">
          {filePath
            ? "No data in range"
            : "Open a file to view and edit its contents"}
        </p>
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
      <div className="flex-1 overflow-y-auto px-4 py-1 font-mono">
        {rows.map((row, ridx) => {
          const rowBaseOffset = parseInt(row.offset, 16);
          return (
            <div
              key={ridx}
              className="flex gap-0 leading-6 hover:bg-gray-900/30"
            >
              {/* Offset */}
              <span className="w-[76px] shrink-0 text-emerald-600 text-xs select-none">
                {row.offset}
              </span>

              {/* Hex bytes */}
              <span className="flex-1 text-xs">
                {row.hex.map((byteHex, bi) => {
                  const absoluteOffset = rowBaseOffset + bi;
                  const isDirty = pendingEdits.has(absoluteOffset);
                  const isEditing = editingOffset === absoluteOffset;
                  const displayByte = isDirty
                    ? pendingEdits
                        .get(absoluteOffset)!
                        .toString(16)
                        .padStart(2, "0")
                        .toUpperCase()
                    : byteHex;

                  if (isEditing) {
                    return (
                      <span
                        key={bi}
                        className="inline-block w-[28px] text-center"
                      >
                        <input
                          ref={inputRef}
                          type="text"
                          value={editValue}
                          onChange={(e) =>
                            handleInputChange(e.target.value, absoluteOffset)
                          }
                          onKeyDown={(e) => handleKeyDown(e, absoluteOffset)}
                          onBlur={() => commitEdit(absoluteOffset, editValue)}
                          maxLength={2}
                          className="w-[26px] bg-blue-950 border border-blue-400 rounded-sm text-center text-blue-100 text-xs outline-none px-0 uppercase"
                          placeholder={byteHex}
                        />
                      </span>
                    );
                  }

                  return (
                    <span
                      key={bi}
                      onClick={() => startEdit(absoluteOffset)}
                      className={`inline-block w-[28px] text-center rounded cursor-text transition-colors text-xs ${
                        isDirty
                          ? "text-amber-300 bg-amber-900/40 hover:bg-amber-800/60"
                          : "text-gray-300 hover:text-white hover:bg-gray-800"
                      }`}
                      title={`Offset: 0x${absoluteOffset
                        .toString(16)
                        .toUpperCase()
                        .padStart(8, "0")} (${absoluteOffset})`}
                    >
                      {displayByte}
                    </span>
                  );
                })}
                {/* Pad remaining cells if row < 16 bytes */}
                {row.hex.length < 16 &&
                  Array.from(
                    { length: 16 - row.hex.length },
                    (_, pi) => (
                      <span
                        key={`pad-${pi}`}
                        className="inline-block w-[28px] text-center"
                      />
                    )
                  )}
              </span>

              {/* ASCII — reflects pending edits */}
              <span className="w-[160px] shrink-0 pl-3 text-xs text-amber-400/70 select-text tracking-wider">
                {row.hex
                  .map((byteHex, bi) => {
                    const absoluteOffset = rowBaseOffset + bi;
                    const byteVal = pendingEdits.has(absoluteOffset)
                      ? pendingEdits.get(absoluteOffset)!
                      : parseInt(byteHex, 16);
                    return byteVal >= 0x20 && byteVal <= 0x7e
                      ? String.fromCharCode(byteVal)
                      : ".";
                  })
                  .join("")}
              </span>
            </div>
          );
        })}
      </div>

      {/* Status bar */}
      <div className="border-t border-gray-800 bg-gray-900/60 px-4 py-1 text-[11px] text-gray-500 flex justify-between select-none">
        <span>
          Showing 0x{rangeStart.toString(16).toUpperCase()} – 0x
          {rangeEnd.toString(16).toUpperCase()}
        </span>
        <span className="flex items-center gap-3">
          {pendingEdits.size > 0 && (
            <span className="text-amber-400">
              {pendingEdits.size} unsaved change
              {pendingEdits.size !== 1 ? "s" : ""}
            </span>
          )}
          <span>{totalBytes.toLocaleString()} bytes total</span>
        </span>
      </div>
    </div>
  );
}
