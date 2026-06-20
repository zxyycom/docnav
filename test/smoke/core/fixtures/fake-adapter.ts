import fs from "node:fs";
import path from "node:path";

interface AdapterRequest extends Record<string, unknown> {
  arguments?: unknown;
  document?: unknown;
  operation?: unknown;
  request_id?: unknown;
}

interface FakeAdapterOptions {
  command: string | null;
  commandArgs: string[];
  extensions: string[];
  id: string;
  log: string | null;
  mode: string;
}

interface InvokeInput {
  request: unknown;
  stdin: string;
}

interface OperationContext {
  adapterId: string;
  args: Record<string, unknown>;
  documentPath: string;
}

type OptionHandler = (options: FakeAdapterOptions, value: string) => void;
type OperationResultBuilder = (context: OperationContext) => Record<string, unknown>;

const optionHandlers: Record<string, OptionHandler> = {
  "--id": (parsed, value) => {
    parsed.id = value;
  },
  "--mode": (parsed, value) => {
    parsed.mode = value;
  },
  "--log": (parsed, value) => {
    parsed.log = value;
  },
  "--extensions": (parsed, value) => {
    parsed.extensions = value.split(",").filter(Boolean);
  }
};

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

const options = parseOptions(process.argv.slice(2));
const invokeInput = options.command === "invoke" ? await readInvokeInput() : emptyInvokeInput();
recordCall(options, { stdin: invokeInput.request ?? invokeInput.stdin });
runCommand(options, invokeInput.request);

function runCommand(options: FakeAdapterOptions, request: unknown) {
  switch (options.command) {
    case "manifest":
      writeJson(createManifest(options));
      return;
    case "probe":
      writeJson(createProbe(options, options.commandArgs[0] ?? ""));
      return;
    case "invoke":
      handleInvoke(options, request);
      return;
    default:
      console.error(`Unknown command ${options.command ?? "(missing)"}`);
      process.exit(2);
  }
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

function handleInvoke(options: FakeAdapterOptions, value: unknown) {
  if (options.mode === "invoke-exit") {
    console.error(`${options.id} invoke failed intentionally`);
    process.exit(9);
  }
  if (options.mode === "invoke-invalid-json") {
    process.stdout.write("{ invalid json");
    return;
  }
  if (!isRecord(value)) {
    writeJson(createFailure("unknown", null, "INVALID_REQUEST", { field: "stdin", reason: "missing request JSON" }));
    process.exit(2);
  }
  const request = value as AdapterRequest;
  if (options.mode === "invoke-schema-invalid") {
    writeJson({
      protocol_version: "0.1",
      request_id: request.request_id,
      operation: request.operation,
      ok: true,
      result: {
        entries: "not an array"
      }
    });
    return;
  }

  writeJson({
    protocol_version: "0.1",
    request_id: request.request_id,
    operation: request.operation,
    ok: true,
    result: resultFor(options, request)
  });
}

function resultFor(options: FakeAdapterOptions, value: AdapterRequest) {
  const document = isRecord(value.document) ? value.document : {};
  const args = isRecord(value.arguments) ? value.arguments : {};
  const documentPath = typeof document.path === "string" ? document.path : "(missing)";
  const resultBuilder = typeof value.operation === "string" ? operationResultBuilders[value.operation] : undefined;
  return resultBuilder?.({ adapterId: options.id, args, documentPath }) ?? {};
}

function createFailure(requestId: unknown, operation: unknown, code: string, details: Record<string, unknown>) {
  return {
    protocol_version: "0.1",
    request_id: requestId,
    operation,
    ok: false,
    error: {
      code,
      message: code,
      details
    }
  };
}

function recordCall(options: FakeAdapterOptions, extra: Record<string, unknown>) {
  if (!options.log) {
    return;
  }
  fs.mkdirSync(path.dirname(options.log), { recursive: true });
  fs.appendFileSync(
    options.log,
    `${JSON.stringify({
      adapter_id: options.id,
      mode: options.mode,
      command: options.command,
      argv: [options.command, ...options.commandArgs],
      cwd: process.cwd(),
      ...extra
    })}\n`,
    "utf8"
  );
}

function writeJson(value: unknown) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

function emptyInvokeInput(): InvokeInput {
  return {
    request: null,
    stdin: ""
  };
}

async function readInvokeInput(): Promise<InvokeInput> {
  const stdin = await readStdin();
  return {
    request: parseRequestJson(stdin),
    stdin
  };
}

function parseRequestJson(stdin: string): unknown {
  if (stdin.trim().length === 0) {
    return null;
  }
  try {
    return JSON.parse(stdin) as unknown;
  } catch {
    return null;
  }
}

async function readStdin() {
  let content = "";
  for await (const chunk of process.stdin) {
    content += chunk;
  }
  return content;
}

function parseOptions(args: string[]): FakeAdapterOptions {
  const parsed: FakeAdapterOptions = {
    id: "fake-adapter",
    mode: "valid",
    log: null,
    extensions: [".md", ".core"],
    command: null,
    commandArgs: []
  };
  let index = 0;
  while (index < args.length) {
    const token = args[index];
    const handler = optionHandlers[token];
    if (handler === undefined) {
      parsed.command = token ?? null;
      parsed.commandArgs = args.slice(index + 1);
      break;
    }

    handler(parsed, args[index + 1] ?? "");
    index += 2;
  }
  return parsed;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
