本 change 的目标是将 OpenSpec capability 命名迁移到长期稳定的主 spec ID；本文是未审核临时 design，只存在于 `openspec/changes/normalize-openspec-capability-names/`，不改变现有主规范、主 specs、schema、examples、实现代码或其它 change。

## Context

OpenSpec 归档时按 delta spec 目录名写入 `openspec/specs/<capability>/spec.md`。当前主 specs 中已有多个 capability ID 来自历史 change 或实现阶段命名，例如 `docnav-core-cli-routing-output-implementation`、`markdown-adapter-v0-implementation` 和 `v0-contract-documentation`。如果继续使用这些 ID，后续 proposal 和 delta spec 会把一次性实现任务误当成长期规范所有权。

这个 change 只建立迁移方案、规则和门禁。创建阶段不触碰 `openspec/specs/`，不更新主规范文档，不修改 schema/examples，不改变 Rust 或 Node.js 实现。

## Goals / Non-Goals

**Goals:**

- 为本仓库 OpenSpec capability ID 建立长期命名规则。
- 给出现有主 specs 和 active changes 的迁移映射候选。
- 要求审计通过后才能执行主 specs 迁移。
- 要求迁移前同步 active changes，避免归档时重新生成旧 capability ID。
- 增加验证门禁，防止旧命名回流。

**Non-Goals:**

- 不在创建本 change 时重命名、移动或合并 `openspec/specs/`。
- 不改变 Docnav 产品契约、协议、CLI、adapter、MCP、schema、examples 或实现行为。
- 不把历史 archive 中的旧 capability ID 改写为新名称。
- 不把 OpenSpec CLI 的固定目录名 `openspec/specs` 改成其它路径。

## Decisions

### Decision 1: Capability ID is an ownership name, not a change name

Capability ID 表示主 spec 的长期所有权；change name 表示一次性变更。迁移规则优先选择稳定名词短语，例如 `mcp-bridge`、`adapter-management`、`markdown-navigation`。这避免未来多个 change 为同一能力创建多个主 spec。

备选方案是保留现有 `*-implementation` 命名，只通过文档解释含义。该方案成本低，但不能阻止归档继续固化旧语义。

### Decision 2: Use a reviewed mapping before touching main specs

本 change 先记录候选映射，审计后再执行。下表是审计输入，不是已批准的最终迁移结果：

| 当前 ID | 待审计目标 ID | 待审计处理 |
| --- | --- | --- |
| `cli-artifact-layout` | `release-artifacts` | rename |
| `docnav-core-cli-routing-output-implementation` | `core-cli` | rename，确认 readable 输出是否保留在同一 capability |
| `markdown-adapter-v0-implementation` | `markdown-navigation` | merge |
| `markdown-reference-baseline` | `markdown-navigation` | merge 为参考来源和边界要求 |
| `protocol-and-adapter-sdk-implementation` | `adapter-protocol` | rename，确认协议与 SDK 是否需要拆分 |
| `v0-contract-documentation` | `docnav-contracts` | rename，确认文档治理是否需要拆分 |

审计必须确认每个 requirement 的最终归属、是否需要拆分，以及 active changes 是否同步到同一目标 ID。最终映射必须回写到本 design 后才能执行主 specs 迁移。

### Decision 3: Active changes must be aligned before archive

当前 active changes 中仍可能有旧 capability ID，例如 `docnav-mcp-bridge-implementation`、`docnav-adapter-management-implementation` 或 `protocol-and-adapter-sdk-implementation`。迁移任务必须从 `openspec list --json` 和 change delta 中动态发现受影响 change，并在它们归档前同步 proposal Capabilities 和 delta spec 目录。

### Decision 4: Validation is the durable guardrail

Skill 和 `openspec/config.yaml` 只能影响 agent 行为或 OpenSpec instructions 输入，不能阻止绕过流程的文件改动。迁移完成后需要增加仓库验证，扫描主 specs、active delta specs 和 proposal Capabilities，防止旧 ID 或不一致目录进入仓库。

## Risks / Trade-offs

- [映射过早固化] → 在 tasks 中设置阻塞级审计，审计前不得迁移主 specs。
- [active changes 继续归档旧 ID] → 迁移前先列出并同步所有 active changes，归档前增加检查。
- [合并 Markdown specs 丢失 requirement] → 合并时逐条复制 requirement，并用 OpenSpec validate 和 diff 复核。
- [验证脚本误报 archive 历史] → 校验范围只覆盖 `openspec/specs/` 和 `openspec/changes/<active>/specs/`，不扫描 `openspec/changes/archive/`。
- [与产品规范混淆] → 本 change 只治理 OpenSpec artifacts 命名，不改变 Docnav runtime 或 public contract。

## Migration Plan

1. 审计 proposal、design、specs 和 tasks 是否只描述 OpenSpec capability 命名迁移，并确认创建阶段未修改主 specs 或主规范。
2. 审计候选映射，确认每个现有主 spec 的目标 ID、合并方式和是否需要拆分。
3. 同步 active changes 的 proposal Capabilities 和 delta spec 目录，防止后续归档写入旧 ID。
4. 按审计后的映射迁移 `openspec/specs/`，保留 requirement 内容并更新 spec title/overview。
5. 增加验证脚本和 workspace 验证入口，检查 capability ID 命名和 proposal/specs 一致性。
6. 运行 OpenSpec 严格验证和仓库验证，确认迁移只影响 OpenSpec artifacts 和相关验证。

## Open Questions

- `v0-contract-documentation` 是否先整体迁移为 `docnav-contracts`，还是直接拆分为多个 capability。
- `protocol-and-adapter-sdk-implementation` 是否保留协议和 SDK 在同一 capability，还是拆成 `adapter-protocol` 与 `adapter-sdk`。
- `docnav-core-cli-routing-output-implementation` 是否在本次迁移中拆出 `readable-output`，还是先整体迁移后由后续 change 拆分。
