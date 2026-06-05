import Ajv2020 from "ajv/dist/2020.js";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const requested = new Set(process.argv.slice(2));
const runAll = requested.size === 0;

const ignoredDirs = new Set([
  ".git",
  ".codegraph",
  "node_modules",
  "target",
  ".venv",
  "dist",
  "build"
]);

function toAbs(relPath) {
  return path.join(root, relPath);
}

function toRel(absPath) {
  return path.relative(root, absPath).replaceAll(path.sep, "/");
}

function walk(dir, predicate = () => true) {
  const results = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) {
        results.push(...walk(path.join(dir, entry.name), predicate));
      }
      continue;
    }

    const filePath = path.join(dir, entry.name);
    if (predicate(filePath)) {
      results.push(filePath);
    }
  }
  return results;
}

function readJson(relPath) {
  return JSON.parse(fs.readFileSync(toAbs(relPath), "utf8"));
}

function listExampleJson(pattern) {
  return fs
    .readdirSync(toAbs("docs/examples/json"))
    .filter((name) => pattern.test(name))
    .map((name) => `docs/examples/json/${name}`)
    .sort();
}

function listSchemaJson() {
  return walk(toAbs("docs/schemas"), (filePath) => filePath.endsWith(".schema.json"))
    .map(toRel)
    .sort();
}

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function assertDeepEqual(actual, expected, message) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}

function charLength(value) {
  return [...value].length;
}

function formatAjvErrors(validate) {
  return (validate.errors ?? [])
    .map((error) => `${error.instancePath || "/"} ${error.message}`)
    .join("; ");
}

function validateWithSchema(ajv, schemaRelPath, dataRelPaths) {
  const validate = ajv.compile(readJson(schemaRelPath));
  for (const dataRelPath of dataRelPaths) {
    const data = readJson(dataRelPath);
    if (!validate(data)) {
      throw new Error(`${dataRelPath} failed ${schemaRelPath}: ${formatAjvErrors(validate)}`);
    }
  }
  console.log(`schema ok: ${schemaRelPath} (${dataRelPaths.length} file(s))`);
}

function validateStrictSchemaCompilation() {
  const schemaRelPaths = listSchemaJson();
  const expectedSchemas = [
    "docs/schemas/protocol-request.schema.json",
    "docs/schemas/protocol-response.schema.json",
    "docs/schemas/manifest.schema.json",
    "docs/schemas/probe-result.schema.json",
    "docs/schemas/readable-outline.schema.json",
    "docs/schemas/readable-read.schema.json",
    "docs/schemas/readable-find.schema.json",
    "docs/schemas/readable-info.schema.json",
    "docs/schemas/readable-error.schema.json"
  ];

  for (const expected of expectedSchemas) {
    assert(schemaRelPaths.includes(expected), `missing expected schema ${expected}`);
  }

  const ajv = new Ajv2020({ allErrors: true, strict: true });
  for (const schemaRelPath of schemaRelPaths) {
    ajv.compile(readJson(schemaRelPath));
  }
  console.log(`schema strict compile ok: ${schemaRelPaths.length} schema file(s)`);
}

function validateProtocolResponseBindingSchema() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  const validate = ajv.compile(readJson("docs/schemas/protocol-response.schema.json"));
  const mismatched = JSON.parse(JSON.stringify(readJson("docs/examples/json/protocol-read-response.json")));
  mismatched.operation = "outline";

  assert(
    !validate(mismatched),
    "protocol response schema must reject operation/result mismatches"
  );
  console.log("protocol response operation/result binding ok");
}

function validateJsonSyntax() {
  const jsonFiles = walk(toAbs("docs"), (filePath) => filePath.endsWith(".json"));
  for (const filePath of jsonFiles) {
    JSON.parse(fs.readFileSync(filePath, "utf8"));
  }
  console.log(`json syntax ok: ${jsonFiles.length} file(s)`);
}

function validateSchemas() {
  validateStrictSchemaCompilation();
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  const cases = [
    {
      schema: "docs/schemas/protocol-request.schema.json",
      data: listExampleJson(/^protocol-.*-request\.json$/)
    },
    {
      schema: "docs/schemas/protocol-response.schema.json",
      data: [
        ...listExampleJson(/^protocol-.*-response\.json$/),
        ...listExampleJson(/^error-.*\.json$/)
      ]
    },
    {
      schema: "docs/schemas/manifest.schema.json",
      data: ["docs/examples/json/manifest.json"]
    },
    {
      schema: "docs/schemas/probe-result.schema.json",
      data: ["docs/examples/json/probe-result.json"]
    },
    {
      schema: "docs/schemas/readable-outline.schema.json",
      data: ["docs/examples/json/readable-outline.json"]
    },
    {
      schema: "docs/schemas/readable-read.schema.json",
      data: ["docs/examples/json/readable-read.json"]
    },
    {
      schema: "docs/schemas/readable-find.schema.json",
      data: ["docs/examples/json/readable-find.json"]
    },
    {
      schema: "docs/schemas/readable-info.schema.json",
      data: ["docs/examples/json/readable-info.json"]
    },
    {
      schema: "docs/schemas/readable-error.schema.json",
      data: ["docs/examples/json/readable-error.json"]
    }
  ];

  for (const item of cases) {
    validateWithSchema(ajv, item.schema, item.data);
  }
  validateProtocolResponseBindingSchema();
}

function validateMcpStructuredContent() {
  const ajv = new Ajv2020({ allErrors: true, strict: true });
  const cases = [
    ["docs/examples/json/mcp-outline-response.json", "docs/schemas/readable-outline.schema.json"],
    ["docs/examples/json/mcp-read-response.json", "docs/schemas/readable-read.schema.json"],
    ["docs/examples/json/mcp-find-response.json", "docs/schemas/readable-find.schema.json"],
    ["docs/examples/json/mcp-info-response.json", "docs/schemas/readable-info.schema.json"]
  ];
  const forbidden = ["protocol_version", "request_id", "operation", "ok"];

  for (const [responseRelPath, schemaRelPath] of cases) {
    const response = readJson(responseRelPath);
    const structuredContent = response?.result?.structuredContent;
    assert(structuredContent && typeof structuredContent === "object", `${responseRelPath} missing structuredContent`);

    for (const field of forbidden) {
      assert(!(field in structuredContent), `${responseRelPath} leaks ${field} in structuredContent`);
    }

    const validate = ajv.compile(readJson(schemaRelPath));
    if (!validate(structuredContent)) {
      throw new Error(`${responseRelPath} structuredContent failed ${schemaRelPath}: ${formatAjvErrors(validate)}`);
    }
  }
  console.log(`mcp structuredContent ok: ${cases.length} response file(s)`);
}

function toReadablePayload(operation, protocolResult) {
  return protocolResult;
}

function validateProtocolPair(operation) {
  const request = readJson(`docs/examples/json/protocol-${operation}-request.json`);
  const response = readJson(`docs/examples/json/protocol-${operation}-response.json`);

  assert(
    response.protocol_version === request.protocol_version,
    `${operation} response protocol_version must match request`
  );
  assert(
    response.request_id === request.request_id,
    `${operation} response request_id must match request`
  );
  assert(response.operation === request.operation, `${operation} response operation must match request`);
  assert(response.ok === true, `${operation} protocol response must be successful`);
  validateProtocolResultBinding(operation, response, `protocol-${operation}-response.json`);

  const requestedPage = request.arguments.page;
  const returnedPage = response.result.page;
  if (typeof requestedPage === "number" && returnedPage !== null) {
    assert(
      returnedPage === requestedPage + 1,
      `${operation} response page must be request page + 1`
    );
  }

  return { request, response };
}

function validateProtocolResultBinding(operation, response, label) {
  const result = response.result;
  assert(result && typeof result === "object", `${label} missing result object`);

  const resultKindChecks = {
    outline: () => Array.isArray(result.entries) && "page" in result && !("matches" in result),
    read: () => "ref" in result && "content" in result && "content_type" in result && "cost" in result,
    find: () => Array.isArray(result.matches) && "page" in result && !("entries" in result),
    info: () => "display" in result && Array.isArray(result.capabilities) && !("page" in result)
  };

  assert(resultKindChecks[operation]?.(), `${label} result does not match ${operation}`);
}

function validateExampleBudget(operation, request, result) {
  const limit = request.arguments.limit_chars;
  if (typeof limit !== "number") {
    return;
  }

  if (operation === "read") {
    assert(
      charLength(result.content) <= limit,
      `${operation} content exceeds limit_chars in example`
    );
    return;
  }

  const records = operation === "outline" ? result.entries : result.matches;
  const totalChars = records.reduce(
    (sum, record) => sum + charLength(record.ref) + charLength(record.display),
    0
  );
  assert(totalChars <= limit, `${operation} ref + display exceeds limit_chars in example`);
}

function validateProtocolReadableMappings() {
  const operations = ["outline", "read", "find", "info"];

  for (const operation of operations) {
    const { request, response } = validateProtocolPair(operation);
    validateExampleBudget(operation, request, response.result);

    const readable = readJson(`docs/examples/json/readable-${operation}.json`);
    assertDeepEqual(
      readable,
      toReadablePayload(operation, response.result),
      `${operation} readable JSON must preserve protocol result semantics`
    );

    const mcp = readJson(`docs/examples/json/mcp-${operation}-response.json`);
    assertDeepEqual(
      mcp.result.structuredContent,
      readable,
      `${operation} MCP structuredContent must match readable JSON example`
    );
  }

  console.log(`protocol/readable mapping ok: ${operations.length} operation(s)`);
}

function validateErrorDetails() {
  const requiredDetailsByCode = {
    INVALID_REQUEST: ["field", "reason"],
    PROTOCOL_INCOMPATIBLE: ["requested", "supported_min", "supported_max"],
    DOCUMENT_NOT_FOUND: ["path"],
    DOCUMENT_PATH_INVALID: ["path", "reason"],
    DOCUMENT_ENCODING_UNSUPPORTED: ["path", "encoding"],
    FORMAT_UNKNOWN: ["path", "reason", "candidates"],
    FORMAT_AMBIGUOUS: ["path", "candidates"],
    CAPABILITY_UNSUPPORTED: ["capability", "adapter_id"],
    REF_NOT_FOUND: ["ref"],
    REF_AMBIGUOUS: ["ref", "candidate_count"],
    ADAPTER_UNAVAILABLE: ["adapter_id", "reason"],
    ADAPTER_INVOKE_FAILED: ["adapter_id", "reason"],
    INTERNAL_ERROR: ["error_id"]
  };

  const errorFiles = listExampleJson(/^error-.*\.json$/);
  for (const errorRelPath of errorFiles) {
    const response = readJson(errorRelPath);
    assert(response.ok === false, `${errorRelPath} must be an error response`);
    assert(!("result" in response), `${errorRelPath} error response must not include result`);
    assert(
      response.operation === null || ["outline", "read", "find", "info"].includes(response.operation),
      `${errorRelPath} error operation must be known operation or null`
    );
    const details = response.error.details;
    const requiredDetails = requiredDetailsByCode[response.error.code];
    assert(requiredDetails, `${errorRelPath} uses unknown error code ${response.error.code}`);
    for (const field of requiredDetails) {
      assert(field in details, `${errorRelPath} missing error.details.${field}`);
    }
  }

  const readableError = readJson("docs/examples/json/readable-error.json");
  assert(readableError.code in requiredDetailsByCode, "readable-error.json uses unknown error code");
  for (const field of requiredDetailsByCode[readableError.code]) {
    assert(field in (readableError.details ?? {}), `readable-error.json missing details.${field}`);
  }

  console.log(`error details ok: ${errorFiles.length + 1} file(s)`);
}

function validateManifestSemantics() {
  const manifest = readJson("docs/examples/json/manifest.json");
  assert(manifest.adapter.id === "docnav-markdown", "manifest example must describe docnav-markdown");

  const requiredMarkdownCapabilities = ["outline", "read", "find", "info"];
  for (const capability of requiredMarkdownCapabilities) {
    assert(
      manifest.capabilities.includes(capability),
      `markdown manifest example missing capability ${capability}`
    );
  }

  const markdownFormat = manifest.formats.find((format) => format.id === "markdown");
  assert(markdownFormat, "manifest example missing markdown format");
  assert(markdownFormat.extensions.includes(".md"), "markdown manifest example missing .md extension");
  assert(
    markdownFormat.content_types.includes("text/markdown"),
    "markdown manifest example missing text/markdown content type"
  );

  console.log("manifest semantics ok: markdown capabilities and format");
}

function validateExampleSemantics() {
  validateProtocolReadableMappings();
  validateErrorDetails();
  validateManifestSemantics();
}

function validateMarkdownLinks() {
  const markdownFiles = walk(root, (filePath) => filePath.endsWith(".md"));
  const missing = [];
  const linkPattern = /\[[^\]]+\]\(([^)]+)\)/g;

  for (const filePath of markdownFiles) {
    const text = fs.readFileSync(filePath, "utf8");
    for (const match of text.matchAll(linkPattern)) {
      const rawTarget = match[1].trim().replace(/^<|>$/g, "");
      if (
        rawTarget === "" ||
        rawTarget.startsWith("#") ||
        /^(https?|mailto):/i.test(rawTarget)
      ) {
        continue;
      }

      const targetPath = rawTarget.split("#")[0];
      if (targetPath === "") {
        continue;
      }

      const resolved = path.resolve(path.dirname(filePath), targetPath);
      if (!fs.existsSync(resolved)) {
        missing.push(`${toRel(filePath)} -> ${rawTarget}`);
      }
    }
  }

  if (missing.length > 0) {
    throw new Error(`missing markdown links:\n${missing.join("\n")}`);
  }

  console.log(`markdown links ok: ${markdownFiles.length} file(s)`);
}

const tasks = {
  json: validateJsonSyntax,
  schema: validateSchemas,
  mcp: validateMcpStructuredContent,
  semantics: validateExampleSemantics,
  links: validateMarkdownLinks
};

const selectedTasks = runAll ? Object.keys(tasks) : [...requested];
for (const taskName of selectedTasks) {
  const task = tasks[taskName];
  assert(task, `unknown validation task: ${taskName}`);
  task();
}
