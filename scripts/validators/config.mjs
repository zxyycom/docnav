// 文档验证脚本的配置中心：这里只集中任务名、验证材料路径和稳定字段名。
// 这些值用于校验 README 指向的主规范和 schema，不把脚本变成新的业务规则来源。
export const TASK_NAMES = {
  json: "json",
  schema: "schema",
  mcp: "mcp",
  semantics: "semantics",
  links: "links"
};

// 文件系统扫描配置，保持验证范围可控，避免遍历构建产物和依赖目录。
export const FILE_SYSTEM = {
  docsDir: "docs",
  examplesJsonDir: "docs/examples/json",
  schemasDir: "docs/schemas",
  ignoredDirs: [".git", ".codegraph", "node_modules", "target", ".venv", "dist", "build"],
  jsonExtension: ".json",
  markdownExtension: ".md",
  schemaExtension: ".schema.json"
};

// schema 和示例路径来自 docs/schemas 与 docs/examples，路径集中后便于审计校验材料增减。
export const SCHEMAS = {
  manifest: "docs/schemas/manifest.schema.json",
  probeResult: "docs/schemas/probe-result.schema.json",
  protocolRequest: "docs/schemas/protocol-request.schema.json",
  protocolResponse: "docs/schemas/protocol-response.schema.json",
  readableError: "docs/schemas/readable-error.schema.json",
  readableFind: "docs/schemas/readable-find.schema.json",
  readableInfo: "docs/schemas/readable-info.schema.json",
  readableOutline: "docs/schemas/readable-outline.schema.json",
  readableRead: "docs/schemas/readable-read.schema.json"
};

export const EXAMPLES = {
  manifest: "docs/examples/json/manifest.json",
  probeResult: "docs/examples/json/probe-result.json",
  protocolReadResponse: "docs/examples/json/protocol-read-response.json",
  readableError: "docs/examples/json/readable-error.json",
  readableFind: "docs/examples/json/readable-find.json",
  readableInfo: "docs/examples/json/readable-info.json",
  readableOutline: "docs/examples/json/readable-outline.json",
  readableRead: "docs/examples/json/readable-read.json"
};

export const OPERATION_NAMES = {
  outline: "outline",
  read: "read",
  find: "find",
  info: "info"
};

export const OPERATIONS = Object.values(OPERATION_NAMES);

export const READABLE_SCHEMA_BY_OPERATION = {
  [OPERATION_NAMES.outline]: SCHEMAS.readableOutline,
  [OPERATION_NAMES.read]: SCHEMAS.readableRead,
  [OPERATION_NAMES.find]: SCHEMAS.readableFind,
  [OPERATION_NAMES.info]: SCHEMAS.readableInfo
};

export const PROTOCOL_EXAMPLE_FILE = {
  request: (operation) => `docs/examples/json/protocol-${operation}-request.json`,
  response: (operation) => `docs/examples/json/protocol-${operation}-response.json`,
  responseName: (operation) => `protocol-${operation}-response.json`
};

export const READABLE_EXAMPLE_FILE = {
  result: (operation) => `docs/examples/json/readable-${operation}.json`
};

export const MCP_EXAMPLE_FILE = {
  response: (operation) => `docs/examples/json/mcp-${operation}-response.json`
};

// 协议与 readable/MCP 示例中反复出现的字段名，集中后避免局部拼写漂移。
export const FIELDS = {
  adapter: "adapter",
  arguments: "arguments",
  capabilities: "capabilities",
  code: "code",
  content: "content",
  contentType: "content_type",
  contentTypes: "content_types",
  cost: "cost",
  details: "details",
  display: "display",
  entries: "entries",
  error: "error",
  extensions: "extensions",
  formats: "formats",
  id: "id",
  limitChars: "limit_chars",
  manifestVersion: "manifest_version",
  matches: "matches",
  ok: "ok",
  operation: "operation",
  page: "page",
  protocolVersion: "protocol_version",
  ref: "ref",
  requestId: "request_id",
  result: "result",
  structuredContent: "structuredContent"
};

export const MCP_STRUCTURED_CONTENT_FORBIDDEN_FIELDS = [
  FIELDS.protocolVersion,
  FIELDS.requestId,
  FIELDS.operation,
  FIELDS.ok
];

// 稳定错误语义由 docs/protocol.md 拥有；这里复用 error-rules.json 生成的 required details 常量。
export { REQUIRED_ERROR_DETAILS_BY_CODE } from "./generated/error-rules.mjs";

// Markdown manifest 示例的语义期望，来源是 v0 首期 adapter 契约。
export const MARKDOWN_MANIFEST_EXPECTED = {
  adapterId: "docnav-markdown",
  contentType: "text/markdown",
  extension: ".md",
  formatId: "markdown",
  capabilities: OPERATIONS
};
