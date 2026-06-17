import { assert } from "./fs-utils.mjs";
import { listMainMarkdownDocs, readText } from "./document-files.mjs";

const MCP_ARTIFACT = "(?:^|[^A-Za-z0-9_-])`?docnav-mcp`?(?![A-Za-z0-9_-])";
const MCP_ARTIFACT_PATTERN = new RegExp(MCP_ARTIFACT, "iu");
const MCP_DELIVERED_FACT_PATTERNS = [
  {
    pattern: new RegExp(
      `${MCP_ARTIFACT}[^\\n]*(是|作为)[^\\n]*(bridge|tools?|tool call|MCP 接入|TextContent|structuredContent|stdio|端到端|E2E)`,
      "iu",
    ),
    label: "current MCP bridge/tool artifact",
  },
  {
    pattern: new RegExp(
      `${MCP_ARTIFACT}[^\\n]*(负责|通过|将|把|直接|提供|暴露|实现|支持|返回|输出|转为|转换|调用|映射(?:为|到)|覆盖|验证|验收|必须)[^\\n]*(bridge|tools?|tool call|核心 \x60?docnav\x60? CLI|TextContent|structuredContent|stdio|端到端|E2E|MCP 接入|tool 映射)`,
      "iu",
    ),
    label: "current MCP bridge/tool behavior",
  },
  {
    pattern: new RegExp(
      `${MCP_ARTIFACT}[^\\n]*(bridge|tools?|tool call|核心 \x60?docnav\x60? CLI|TextContent|structuredContent|stdio|端到端|E2E|MCP 接入|tool 映射)[^\\n]*(负责|通过|将|把|直接|提供|暴露|实现|支持|返回|输出|转为|转换|调用|映射(?:为|到)|覆盖|验证|验收|必须)`,
      "iu",
    ),
    label: "current MCP bridge/tool behavior",
  },
];
const MCP_EXPLICIT_HANDOFF_CONTEXT_PATTERN =
  /(目标|handoff|交接|承接|in-progress|implement-docnav-mcp-bridge|不表示|不交付|当前主文档只定义|当前测试策略只记录)/iu;
const MCP_OWNERSHIP_CONTEXT_PATTERN = /(ownership|owner|不拥有)/iu;
const MCP_DELIVERED_FACT_WITH_HANDOFF_CONTEXT_SAMPLES = [
  "docnav-mcp 通过 Node.js / JavaScript bridge 直接调用核心 docnav CLI，不拥有文档解析职责。",
  "docnav-mcp 将 MCP tool call 直接映射为核心 `docnav` CLI 调用，不拥有 adapter 路由职责。",
  "docnav-mcp 把 readable 结果转为 TextContent 和 structuredContent，不拥有 adapter 管理职责。",
];
const MCP_HANDOFF_CONTEXT_ONLY_SAMPLE =
  "docnav-mcp 不拥有文档解析、adapter 路由或 adapter 管理职责。";
const MCP_EXPLICIT_HANDOFF_CONTEXT_SAMPLE =
  "`docnav-mcp` 是 Node.js / JavaScript MCP bridge 的目标制品，当前由 in-progress 的 `implement-docnav-mcp-bridge` change 承接实现。";

export function collectMcpBridgeHandoffDocViolations(
  line,
  location = "sample",
) {
  if (!MCP_ARTIFACT_PATTERN.test(line)) {
    return [];
  }

  const labels = new Set();
  for (const { pattern, label } of MCP_DELIVERED_FACT_PATTERNS) {
    if (pattern.test(line)) {
      labels.add(label);
    }
  }

  if (labels.size === 0 || MCP_EXPLICIT_HANDOFF_CONTEXT_PATTERN.test(line)) {
    return [];
  }

  return [...labels].map((label) => `${location} describes ${label}`);
}

function validateMcpBridgeHandoffGuardSamples() {
  for (const line of MCP_DELIVERED_FACT_WITH_HANDOFF_CONTEXT_SAMPLES) {
    assert(
      MCP_OWNERSHIP_CONTEXT_PATTERN.test(line),
      "MCP handoff negative sample must include context wording",
    );
    assert(
      collectMcpBridgeHandoffDocViolations(line).length > 0,
      `MCP handoff semantic guard must reject delivered-fact wording before ownership-only context wording: ${line}`,
    );
  }

  assert(
    collectMcpBridgeHandoffDocViolations(MCP_HANDOFF_CONTEXT_ONLY_SAMPLE)
      .length === 0,
    "MCP handoff semantic guard must allow ownership wording when no delivered-fact wording exists",
  );
  assert(
    collectMcpBridgeHandoffDocViolations(MCP_EXPLICIT_HANDOFF_CONTEXT_SAMPLE)
      .length === 0,
    "MCP handoff semantic guard must allow explicit target handoff wording",
  );
}

export function validateMcpBridgeHandoffDocs() {
  const violations = [];
  const docs = listMainMarkdownDocs();
  validateMcpBridgeHandoffGuardSamples();

  for (const relPath of docs) {
    const lines = readText(relPath).split(/\r?\n/u);
    lines.forEach((line, index) => {
      violations.push(
        ...collectMcpBridgeHandoffDocViolations(
          line,
          `${relPath}:${index + 1}`,
        ),
      );
    });
  }

  assert(
    violations.length === 0,
    `main docs must describe docnav-mcp bridge/tools/E2E as target handoff, not current delivery:\n${violations.join("\n")}`,
  );
  console.log(`MCP bridge handoff docs ok: ${docs.length} file(s)`);
}
