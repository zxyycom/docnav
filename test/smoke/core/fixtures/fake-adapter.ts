import fs from "node:fs";
import path from "node:path";

const options = parseOptions(process.argv.slice(2));
const stdin = options.command === "invoke" ? await readStdin() : "";
let request = null;
if (stdin.trim().length > 0) {
  try {
    request = JSON.parse(stdin);
  } catch {
    request = null;
  }
}

recordCall({ stdin: request ?? stdin });

switch (options.command) {
  case "manifest":
    writeJson(manifest());
    break;
  case "probe":
    writeJson(probe(options.commandArgs[0]));
    break;
  case "invoke":
    handleInvoke(request);
    break;
  default:
    console.error(`ExternalValue command ${options.command ?? "(missing)"}`);
    process.exit(2);
}

function manifest() {
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

function probe(documentPath: ExternalValue) {
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

function handleInvoke(value: ExternalValue) {
  if (options.mode === "invoke-exit") {
    console.error(`${options.id} invoke failed intentionally`);
    process.exit(9);
  }
  if (options.mode === "invoke-invalid-json") {
    process.stdout.write("{ invalid json");
    return;
  }
  if (!value || typeof value !== "object") {
    writeJson(failure("ExternalValue", null, "INVALID_REQUEST", { field: "stdin", reason: "missing request JSON" }));
    process.exit(2);
  }
  if (options.mode === "invoke-schema-invalid") {
    writeJson({
      protocol_version: "0.1",
      request_id: value.request_id,
      operation: value.operation,
      ok: true,
      result: {
        entries: "not an array"
      }
    });
    return;
  }

  writeJson({
    protocol_version: "0.1",
    request_id: value.request_id,
    operation: value.operation,
    ok: true,
    result: resultFor(value)
  });
}

function resultFor(value: ExternalValue) {
  const documentPath = value.document?.path ?? "(missing)";
  switch (value.operation) {
    case "outline":
      return {
        entries: [
          {
            ref: "fake:root",
            display: `${options.id} outline for ${documentPath}`
          }
        ],
        page: null
      };
    case "read":
      return {
        ref: value.arguments?.ref ?? "fake:root",
        content: `# Fake ${options.id}\n\nRead ${documentPath}`,
        content_type: "text/markdown",
        cost: "0.1 KB",
        page: null
      };
    case "find":
      return {
        matches: [
          {
            ref: "fake:root",
            display: `${options.id} match for ${value.arguments?.query ?? ""}`
          }
        ],
        page: null
      };
    case "info":
      return {
        display: `Fake ${options.id} | text/markdown`,
        capabilities: ["outline", "read", "find", "info"]
      };
    default:
      return {};
  }
}

function failure(requestId: ExternalValue, operation: ExternalValue, code: ExternalValue, details: ExternalValue) {
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

function recordCall(extra: ExternalValue) {
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

function writeJson(value: ExternalValue) {
  process.stdout.write(`${JSON.stringify(value)}\n`);
}

async function readStdin() {
  let content = "";
  for await (const chunk of process.stdin) {
    content += chunk;
  }
  return content;
}

function parseOptions(args: ExternalValue) {
  const parsed: {
    id: string;
    mode: string;
    log: string | null;
    extensions: string[];
    command: string | null;
    commandArgs: string[];
  } = {
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
    if (token === "--id") {
      parsed.id = args[index + 1];
      index += 2;
    } else if (token === "--mode") {
      parsed.mode = args[index + 1];
      index += 2;
    } else if (token === "--log") {
      parsed.log = args[index + 1];
      index += 2;
    } else if (token === "--extensions") {
      parsed.extensions = args[index + 1].split(",").filter(Boolean);
      index += 2;
    } else {
      parsed.command = token;
      parsed.commandArgs = args.slice(index + 1);
      break;
    }
  }
  return parsed;
}
