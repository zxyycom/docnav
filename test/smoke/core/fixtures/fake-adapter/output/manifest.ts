import type { FakeAdapterOptions } from "../types.ts";
import { writeJson } from "./response.ts";

export function writeManifest(options: FakeAdapterOptions) {
  writeJson(createManifest(options));
}

function createManifest(options: FakeAdapterOptions) {
  if (options.mode === "manifest-exit") {
    console.error(`${options.id} manifest failed intentionally`);
    process.exit(7);
  }
  if (options.mode === "manifest-invalid") {
    return {
      manifest_version: "0.1",
      adapter: {
        id: options.id
      }
    };
  }
  return {
    manifest_version: "0.1",
    adapter: {
      id: options.id,
      name: `Fake ${options.id}`,
      version: "0.0.0"
    },
    formats: [
      {
        id: "fake",
        extensions: options.extensions,
        content_types: ["text/markdown"]
      }
    ],
    capabilities: ["outline", "read", "find", "info"]
  };
}
