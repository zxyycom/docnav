// 文档验证脚本的配置中心：这里只集中任务名、验证材料路径和稳定字段名。
// 这些值用于校验文档导航指向的主规范和 schema，不把脚本变成新的业务规则来源。
export const TASK_NAMES = {
  cases: "cases",
  json: "json",
  schema: "schema",
  examples: "examples",
  links: "links"
};

// 文件系统扫描配置，保持验证范围可控，避免遍历构建产物和依赖目录。
export const FILE_SYSTEM = {
  docsDir: "docs",
  examplesJsonDir: "docs/examples/json",
  schemasDir: "docs/schemas",
  ignoredDirs: [".git", ".codegraph", "node_modules", "target", ".venv", "dist", "build"],
  markdownLinkRoots: ["README.md", "docs"],
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
  docnavMarkdownConfig: "docs/schemas/docnav-markdown-config.schema.json",
  readableError: "docs/schemas/readable-error.schema.json",
  readableFind: "docs/schemas/readable-find.schema.json",
  readableInfo: "docs/schemas/readable-info.schema.json",
  readableOutline: "docs/schemas/readable-outline.schema.json",
  readableCommon: "docs/schemas/readable-common.schema.json",
  readableRead: "docs/schemas/readable-read.schema.json"
};

export const EXAMPLES = {
  docnavMarkdownConfig: "docs/examples/json/docnav-markdown-config.json",
  manifest: "docs/examples/json/manifest.json",
  probeResult: "docs/examples/json/probe-result.json",
  protocolReadResponse: "docs/examples/json/protocol-read-response.json",
  errorInvalidRequest: "docs/examples/json/error-invalid-request.json",
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

export const DOCUMENT_OUTPUT_MODES = [
  "readable-view",
  "readable-json",
  "protocol-json"
];

export const READABLE_SCHEMA_BY_OPERATION = {
  [OPERATION_NAMES.outline]: SCHEMAS.readableOutline,
  [OPERATION_NAMES.read]: SCHEMAS.readableRead,
  [OPERATION_NAMES.find]: SCHEMAS.readableFind,
  [OPERATION_NAMES.info]: SCHEMAS.readableInfo
};

export const OUTPUT_MODE_CONSISTENCY = {
  conformanceDir: "crates/docnav-readable/tests/fixtures/conformance",
  conformanceReadme: "crates/docnav-readable/tests/fixtures/conformance/README.md",
  conformanceFixtures: [
    "01_no_block_outline.json",
    "04_single_block.json",
    "07_chinese.json",
    "10_crlf_payload.json",
    "11_no_trailing_newline.json",
    "12_block_marker_in_body.json",
    "14_readable_error.json",
    "15_error_guidance_array.json",
    "16_undeclared_extension_fields.json",
    "17_order_independent_assertions.json",
    "18_renderer_failure_missing_pointer.json",
    "19_renderer_failure_non_string.json"
  ]
};

export const PROTOCOL_EXAMPLE_FILE = {
  request: (operation: string) => `docs/examples/json/protocol-${operation}-request.json`,
  response: (operation: string) => `docs/examples/json/protocol-${operation}-response.json`,
  responseName: (operation: string) => `protocol-${operation}-response.json`
};

export const READABLE_EXAMPLE_FILE = {
  result: (operation: string) => `docs/examples/json/readable-${operation}.json`
};

// 协议与 readable 示例中反复出现的字段名，集中后避免局部拼写漂移。
export const FIELDS = {
  adapter: "adapter",
  arguments: "arguments",
  code: "code",
  content: "content",
  contentType: "content_type",
  contentTypes: "content_types",
  cost: "cost",
  details: "details",
  display: "display",
  document: "document",
  entries: "entries",
  error: "error",
  extensions: "extensions",
  formats: "formats",
  id: "id",
  limit: "limit",
  manifestVersion: "manifest_version",
  matches: "matches",
  metadata: "metadata",
  ok: "ok",
  operation: "operation",
  page: "page",
  protocolVersion: "protocol_version",
  ref: "ref",
  requestId: "request_id",
  result: "result"
};

// Markdown manifest 示例的语义期望，来源是 v0 首期 adapter 契约。
export const MARKDOWN_MANIFEST_EXPECTED = {
  adapterId: "docnav-markdown",
  contentType: "text/markdown",
  extension: ".md",
  formatId: "markdown"
};
