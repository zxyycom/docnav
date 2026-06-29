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
        label: `${adapterId} outline for ${documentPath}`,
        kind: "document",
        cost: cost("entry")
      }
    ],
    page: null
  }),
  read: ({ adapterId, args, documentPath }) => ({
    ref: typeof args.ref === "string" ? args.ref : "fake:root",
    content: `# Fake ${adapterId}\n\nRead ${documentPath}`,
    content_type: "text/markdown",
    cost: cost("selection"),
    page: null
  }),
  find: ({ adapterId, args }) => ({
    matches: [
      {
        ref: "fake:root",
        label: `${adapterId} match for ${typeof args.query === "string" ? args.query : ""}`,
        kind: "match"
      }
    ],
    page: null
  }),
  info: ({ adapterId }) => ({
    capabilities: ["outline", "read", "find", "info"],
    document: {
      content_type: "text/markdown"
    },
    adapter: {
      id: adapterId,
      format: "fake"
    }
  })
};

function cost(scope: string) {
  return {
    measurements: [
      { unit: "lines", value: 1, scope },
      { unit: "bytes", value: 64, scope }
    ]
  };
}

export function resultFor(options: FakeAdapterOptions, value: AdapterRequest) {
  const document = isRecord(value.document) ? value.document : {};
  const args = isRecord(value.arguments) ? value.arguments : {};
  const documentPath = typeof document.path === "string" ? document.path : "(missing)";
  const resultBuilder = typeof value.operation === "string" ? operationResultBuilders[value.operation] : undefined;
  return resultBuilder?.({ adapterId: options.id, args, documentPath }) ?? {};
}
