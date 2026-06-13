本 change 的目标是将 OpenSpec capability 命名迁移到长期稳定的主 spec ID；本文是未审核临时 proposal，只存在于 `openspec/changes/normalize-openspec-capability-names/`，不改变现有主规范、主 specs、schema、examples、实现代码或其它 change。

## Why

当前 `openspec/specs/` 中多个 capability ID 混入了 `implementation`、`v0` 或历史 change 语义，导致后续 delta spec 容易把一次性任务名当成长期主 spec 所有权。归档会按 delta spec 目录名合并到主 specs，因此需要先用一个受审计的 change 固定命名规则、迁移映射和执行门禁。

## What Changes

- 新增 OpenSpec 治理能力，定义 change name 与 capability ID 的区别、capability ID 命名标准和迁移前审计要求。
- 建立现有主 spec ID 和 active-only delta capability ID 到目标长期 capability ID 的迁移映射，要求在审计通过后再改动主 specs 或 active changes。
- 要求同步 active changes 的 delta spec 目录和 proposal Capabilities，避免归档时重新生成旧命名。
- 将 capability 命名规则沉淀为 OpenSpec 治理和 skill/人工审计判断，不新增专用脚本、CI gate 或 workspace 验证入口。
- 非目标：本 change 创建阶段不移动、重命名或修改 `openspec/specs/`，不改变 Docnav 产品、协议、CLI、adapter、MCP、schema、examples、实现行为或验证入口。

## Capabilities

### New Capabilities

- `openspec-governance`: 管理本仓库 OpenSpec capability ID 命名、spec 迁移映射、active change 对齐和审计边界。

### Modified Capabilities

- 无。现有主 spec 迁移必须等本 change 的审计任务完成后，按 tasks 中的映射和校验步骤执行。

## Impact

- 影响 OpenSpec artifacts：`openspec/specs/` 的 capability ID 命名、active changes 的 `specs/<capability>/spec.md` 目录和 proposal Capabilities。
- 影响项目规则：后续通过 OpenSpec 治理说明和 skill/人工审计遵循 capability 命名规则，不通过新增脚本或 CI 阻止旧命名。
- 不影响当前主规范文档、schema、examples、Rust/Node 实现、CLI 行为或 MCP 映射。
