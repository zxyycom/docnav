import type { DuplicateCodeFragment } from "../schema.ts";

export type CpdScanResult =
  | { fragments: DuplicateCodeFragment[]; ok: true }
  | { error: string; ok: false; reason?: string; skipped: boolean };
