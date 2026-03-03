import { invoke } from "@tauri-apps/api/core";

// ── Types matching the Rust backend ────────────────────────────────

export interface FileInfo {
  path: string;
  size: number;
  file_type: string;
}

export interface HexRow {
  offset: string;
  hex: string[];
  ascii: string;
}

export interface ReadResult {
  rows: HexRow[];
  total_bytes: number;
}

export interface StringMatch {
  offset: number;
  value: string;
}

// ── API wrappers ───────────────────────────────────────────────────

export async function openFile(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("open_file", { path });
}

export async function readBytes(
  path: string,
  start: number,
  end: number
): Promise<ReadResult> {
  return invoke<ReadResult>("read_bytes", { path, start, end });
}

export async function getFileSize(path: string): Promise<number> {
  return invoke<number>("get_file_size", { path });
}

export async function getFileType(path: string): Promise<string> {
  return invoke<string>("get_file_type", { path });
}

export async function writeOverwrite(
  path: string,
  offset: number,
  data: string,
  opts?: {
    endOffset?: number;
    dataFile?: string;
    dataFileOffset?: number;
    appendZeroPastEof?: boolean;
    reverse?: boolean;
  }
): Promise<string> {
  return invoke<string>("write_overwrite", {
    path,
    offset,
    endOffset: opts?.endOffset ?? null,
    data,
    dataFile: opts?.dataFile ?? null,
    dataFileOffset: opts?.dataFileOffset ?? null,
    appendZeroPastEof: opts?.appendZeroPastEof ?? false,
    reverse: opts?.reverse ?? false,
  });
}

export async function writeInsert(
  path: string,
  offset: number,
  data: string,
  reverse?: boolean,
  dataFile?: string,
  dataFileOffset?: number
): Promise<string> {
  return invoke<string>("write_insert", {
    path,
    offset,
    data,
    dataFile: dataFile ?? null,
    dataFileOffset: dataFileOffset ?? null,
    reverse: reverse ?? false,
  });
}

export async function copyFile(
  src: string,
  dest: string
): Promise<string> {
  return invoke<string>("copy_file", { src, dest });
}

export async function scanStrings(
  path: string,
  minLength?: number
): Promise<StringMatch[]> {
  return invoke<StringMatch[]>("scan_strings", {
    path,
    minLength: minLength ?? null,
  });
}
