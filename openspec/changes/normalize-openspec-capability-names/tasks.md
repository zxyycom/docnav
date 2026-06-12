本 change 的目标是将 OpenSpec capability 命名迁移到长期稳定的主 spec ID；本文是未审核临时 tasks，只存在于 `openspec/changes/normalize-openspec-capability-names/`，不改变现有主规范、主 specs、schema、examples、实现代码或其它 change。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本次 proposal、design、specs 和 tasks，并明确允许开始迁移；未完成本项前，1.x 及后续任务全部处于阻塞状态。
- [ ] 0.2 审计本 change 是否只围绕“OpenSpec capability 命名迁移”这一核心句，确认创建阶段只包含 `openspec/changes/normalize-openspec-capability-names/` 下的未审核临时 artifacts。
- [ ] 0.3 审计创建阶段是否没有修改 `openspec/specs/`、主规范文档、schema、examples、实现代码或其它 active/archive change。
- [ ] 0.4 审计 `openspec-governance` 作为新 capability 是否符合长期能力命名规则，并确认没有把 change name 复用为 capability ID。
- [ ] 0.5 审计 design 中的旧 ID -> 新 ID 候选映射；对每个现有主 spec 明确目标是 rename、merge、split 还是 keep，并记录最终映射。

## 1. 迁移清单确认

- [ ] 1.1 运行 `openspec list --specs --json`，记录所有当前主 spec ID、requirement 数量和目标迁移处理方式。
- [ ] 1.2 运行 `openspec list --json` 和必要的 `openspec show "<change>" --type change --json --deltas-only --no-interactive`，列出所有 active change 使用的 capability ID。
- [ ] 1.3 将最终迁移映射同步回本 change 的 design；如果映射与 proposal/specs 不一致，先更新 artifacts 再继续。
- [ ] 1.4 明确 archive 历史不迁移；校验脚本和人工审计只覆盖 `openspec/specs/` 与 active changes。

## 2. Active Changes 对齐

- [ ] 2.1 按最终映射更新 active changes 的 proposal Capabilities，确保每个 Modified/New Capability 使用目标长期 ID。
- [ ] 2.2 按最终映射移动 active changes 下的 `specs/<old-id>/spec.md` 到 `specs/<target-id>/spec.md`，并确认 delta 内容未丢失。
- [ ] 2.3 对 `openspec list --json` 返回的每个 active change 做归档风险复核，确认不会重新创建旧 capability ID。
- [ ] 2.4 对未同步或决定延后的 active change 记录原因、风险和归档前置条件。

## 3. 主 Specs 迁移

- [ ] 3.1 按审计后的映射迁移 `openspec/specs/` 目录，先创建目标目录和 spec，再确认内容完整后移除旧目录。
- [ ] 3.2 对 merge 场景逐条迁移 requirement 和 scenario，保留原始要求并消除重复或冲突。
- [ ] 3.3 更新迁移后 spec 的标题、overview 和 requirement 分组，使其表达长期 capability 所有权，而不是实现阶段。
- [ ] 3.4 用局部 diff 复核主 specs 迁移只改变 OpenSpec capability 组织和命名，不改变 Docnav runtime contract。

## 4. 验证门禁

- [ ] 4.1 新增或更新验证脚本，扫描 `openspec/specs/` 和 active `openspec/changes/*/specs/` 的 capability ID，拒绝 `implement`、`implementation`、`change`、`task`、日期和临时版本阶段。
- [ ] 4.2 验证 proposal Capabilities 与实际 `specs/<capability>/spec.md` 目录一致，发现缺失或多余 capability 时失败。
- [ ] 4.3 将 capability 命名验证接入仓库现有文档或 workspace 验证入口，确保后续 agent 绕过 OpenSpec instructions 时仍会被检查。
- [ ] 4.4 确认验证范围排除 `openspec/changes/archive/`，避免历史审计材料被迁移规则误报。

## 5. 最终验证

- [ ] 5.1 运行 `openspec validate --specs --json --strict --no-interactive`，确认迁移后主 specs 结构有效。
- [ ] 5.2 对所有 active changes 运行 `openspec validate "<change>" --type change --json --strict --no-interactive` 或等价批量验证，确认 delta spec 路径和 proposal Capabilities 对齐。
- [ ] 5.3 运行 capability 命名验证脚本，确认主 specs 和 active changes 不再使用旧命名。
- [ ] 5.4 运行 `pnpm run validate:docs`；若本轮接入 workspace 验证或触及跨层验证入口，再运行 `pnpm run verify:docnav-workspace`。
- [ ] 5.5 用 `git diff` 复核改动范围，确认本 change 的实现阶段只修改 OpenSpec artifacts、相关验证脚本和必要 package script，不改变 Docnav 产品主规范、schema、examples 或 runtime 实现。
