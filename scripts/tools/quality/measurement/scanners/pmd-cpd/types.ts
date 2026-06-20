import type { DuplicateCodeFragment } from "../../../model/schema.ts";

export type PmdCpdScanResult =
  | { fragments: DuplicateCodeFragment[]; ok: true }
  | { error: string; ok: false; reason?: string; skipped: boolean };
