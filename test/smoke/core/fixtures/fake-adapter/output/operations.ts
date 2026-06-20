import type {
  AdapterRequest,
  FakeAdapterOptions,
  OperationResultBuilder
} from "../types.ts";
import { isRecord } from "./response.ts";

const operationResultBuilders: Partial<Record<string, OperationResultBuilder>> = {
  outline: ({ adapterId, documentPath }) => ({
    entries: [
      {
        ref: "fake:root",
        display: `${adapterId} outline for ${documentPath}`
      }
    ],
    page: null
  }),
  read: ({ adapterId, args, documentPath }) => ({
    ref: typeof args.ref === "string" ? args.ref : "fake:root",
    content: `# Fake ${adapterId}\n\nRead ${documentPath}`,
    content_type: "text/markdown",
    cost: "0.1 KB",
    page: null
  }),
  find: ({ adapterId, args }) => ({
    matches: [
      {
        ref: "fake:root",
        display: `${adapterId} match for ${typeof args.query === "string" ? args.query : ""}`
      }
    ],
    page: null
  }),
  info: ({ adapterId }) => ({
    display: `Fake ${adapterId} | text/markdown`,
    capabilities: ["outline", "read", "find", "info"]
  })
};

export function resultFor(options: FakeAdapterOptions, value: AdapterRequest) {
  const document = isRecord(value.document) ? value.document : {};
  const args = isRecord(value.arguments) ? value.arguments : {};
  const documentPath = typeof document.path === "string" ? document.path : "(missing)";
  const resultBuilder = typeof value.operation === "string" ? operationResultBuilders[value.operation] : undefined;
  return resultBuilder?.({ adapterId: options.id, args, documentPath }) ?? {};
}
