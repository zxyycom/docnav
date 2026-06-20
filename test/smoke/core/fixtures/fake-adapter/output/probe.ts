import type { FakeAdapterOptions } from "../types.ts";
import { writeJson } from "./response.ts";

export function writeProbe(options: FakeAdapterOptions, documentPath: string) {
  writeJson(createProbe(options, documentPath));
}

function createProbe(options: FakeAdapterOptions, documentPath: string) {
  if (options.mode === "probe-exit") {
    console.error(`${options.id} probe failed intentionally`);
    process.exit(8);
  }
  if (options.mode === "probe-invalid") {
    return {
      probe_version: "0.1",
      adapter_id: options.id,
      path: documentPath,
      format: "fake",
      confidence: 1,
      reasons: [{ code: "EXTENSION_MATCH", detail: "intentionally missing supported" }]
    };
  }
  const supported = options.mode !== "probe-unsupported";
  return {
    probe_version: "0.1",
    adapter_id: options.id,
    path: documentPath,
    supported,
    format: supported ? "fake" : null,
    confidence: supported ? 1 : 0,
    reasons: [
      {
        code: supported ? "EXTENSION_MATCH" : "CONTENT_CONFLICT",
        detail: supported ? "fake adapter accepts the document" : "fake adapter intentionally declined the document"
      }
    ]
  };
}
