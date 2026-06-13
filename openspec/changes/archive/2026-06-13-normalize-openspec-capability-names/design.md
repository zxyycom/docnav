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
- 将命名规则沉淀为 OpenSpec 治理和 skill/人工审计判断，不新增长期自动化阻断。

**Non-Goals:**

- 不在创建本 change 时重命名、移动或合并 `openspec/specs/`。
- 不改变 Docnav 产品契约、协议、CLI、adapter、MCP、schema、examples 或实现行为。
- 不把历史 archive 中的旧 capability ID 改写为新名称。
- 不把 OpenSpec CLI 的固定目录名 `openspec/specs` 改成其它路径。

## Decisions

### Decision 1: Capability ID is an ownership name, not a change name

Capability ID 表示主 spec 的长期所有权；change name 表示一次性变更。迁移规则优先选择稳定名词短语，例如 `mcp-bridge`、`adapter-management`、`markdown-navigation`。这避免未来多个 change 为同一能力创建多个主 spec。

备选方案是保留现有 `*-implementation` 命名，只通过文档解释含义。该方案成本低，但不能阻止归档继续固化旧语义。

### Decision 2: Use the audited mapping before touching specs

本 change 已将候选映射收敛为首轮执行映射。首轮迁移只规范 capability ID 所有权命名，不做 requirement 语义拆分；需要拆分时必须由后续 change 重新提出。

| 当前 ID | 目标 ID | 迁移处理 |
| --- | --- | --- |
| `cli-artifact-layout` | `release-artifacts` | main spec rename |
| `docnav-core-cli-routing-output-implementation` | `core-cli` | main spec + active delta rename；既有 core CLI、routing、config、output mapping requirement 先保留在同一 capability |
| `markdown-adapter-v0-implementation` | `markdown-navigation` | main spec + active delta merge |
| `markdown-reference-baseline` | `markdown-navigation` | main spec merge 为参考来源、行为基线和边界 requirement |
| `protocol-and-adapter-sdk-implementation` | `adapter-protocol` | main spec + active delta rename；协议类型、adapter SDK invoke 生命周期、schema/example validation 首轮保持同一 capability |
| `v0-contract-documentation` | `docnav-contracts` | main spec + active delta rename；v0 文档契约、阅读路径和跨层责任边界首轮保持同一 capability |
| `docnav-adapter-management-implementation` | `adapter-management` | active delta only rename |
| `docnav-mcp-bridge-implementation` | `mcp-bridge` | active delta only rename |

已符合命名规则的 active capability ID 保持不变：`fast-outline`、`readable-view-output`、`openspec-governance`。

### Decision 3: Active changes must be aligned before archive

迁移任务必须同步所有 active changes 的 proposal Capabilities 和 delta spec 目录，避免后续归档重新创建旧 ID。审计时确认的 active change 对齐目标如下：

- `implement-docnav-adapter-management`: `docnav-adapter-management-implementation` -> `adapter-management`
- `implement-docnav-mcp-bridge`: `docnav-mcp-bridge-implementation` -> `mcp-bridge`
- `plan-runtime-schema-validation-removal`: `protocol-and-adapter-sdk-implementation` -> `adapter-protocol`
- `refine-adapter-owned-ref-contract`: `markdown-adapter-v0-implementation` -> `markdown-navigation`；`v0-contract-documentation` -> `docnav-contracts`
- `replace-text-with-readable-view`: `docnav-core-cli-routing-output-implementation` -> `core-cli`；`protocol-and-adapter-sdk-implementation` -> `adapter-protocol`；`markdown-adapter-v0-implementation` -> `markdown-navigation`；`readable-view-output` 保持不变
- `add-fast-outline`: `fast-outline` 保持不变
- `normalize-openspec-capability-names`: `openspec-governance` 保持不变

### Decision 4: Naming guidance is not a validation gate

本次问题来自历史 skill 没有区分 change name 与 capability ID，而不是 runtime、schema 或 CI 缺口。命名规则应沉淀为 OpenSpec 治理和 skill/人工审计判断；本 change 不新增 capability 命名脚本、CI gate、package script 或 workspace 验证入口。

## Risks / Trade-offs

- [映射过早固化] → 在 tasks 中设置阻塞级审计，审计前不得迁移主 specs。
- [active changes 立即归档旧 ID] → 本轮迁移先列出并同步当前 active changes，避免迁移过程中把旧 ID 写回主 specs。
- [合并 Markdown specs 丢失 requirement] → 合并时逐条复制 requirement，并用 OpenSpec validate 和 diff 复核。
- [过度治理命名坏例] → 不把历史坏命名沉淀为脚本或 CI gate；后续命名问题通过 skill/人工审计修正。
- [与产品规范混淆] → 本 change 只治理 OpenSpec artifacts 命名，不改变 Docnav runtime 或 public contract。

## Migration Plan

1. 审计 proposal、design、specs 和 tasks 是否只描述 OpenSpec capability 命名迁移，并确认创建阶段未修改主 specs 或主规范。
2. 使用已审计映射，确认每个现有主 spec 和 active-only delta capability 的目标 ID、合并方式和首轮不拆分边界。
3. 同步 active changes 的 proposal Capabilities 和 delta spec 目录，避免本轮迁移期间归档写入旧 ID。
4. 按审计后的映射迁移 `openspec/specs/`，保留 requirement 内容并更新 spec title/overview。
5. 不新增 capability 命名脚本、CI gate 或 workspace 验证入口；迁移正确性通过 OpenSpec 严格验证和局部 diff 复核。
6. 确认迁移只影响 OpenSpec artifacts，不改变 Docnav runtime、public contract 或既有验证入口。

## Deferred Splits

首轮迁移没有阻塞性开放问题。以下拆分不在本 change 中执行，只有出现独立需求或归档冲突时才由后续 change 处理：

- `docnav-contracts` 可在后续拆分为更细的文档治理或契约 capability。
- `adapter-protocol` 可在后续拆分出 `adapter-sdk`。
- `core-cli` 可在后续把 readable output 的通用契约迁移到独立 capability；当前 `readable-view-output` 作为 active new capability 保持不变。
