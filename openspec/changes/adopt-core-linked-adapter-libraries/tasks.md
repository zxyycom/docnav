本 tasks 清单定义实现 core release 内置 adapter-layer workspace crates change 的执行顺序、阻塞审计点、剩余设计问题决策门槛和验证范围。

## 1. 阻塞级 artifact 审计

- [ ] 1.1 审计 `proposal.md`、`design.md`、`specs/core-cli/spec.md`、`specs/docnav-contracts/spec.md` 和本 `tasks.md` 是否都围绕“默认 document operation implementation source 来自 core release 内置 adapter-layer workspace crates；core 使用统一 static adapter registry；adapter layer 继续拥有格式语义”这一核心句。
- [ ] 1.2 审计 capability ID 是否只使用已有 `core-cli` 和 `docnav-contracts`，且没有把 change 名称误建为新 capability。
- [ ] 1.3 审计当前阶段是否只更新 `openspec/changes/adopt-core-linked-adapter-libraries/` 下的 change artifacts，且未提前修改现有主规范、docs、schema、examples、代码或其它 change。
- [ ] 1.4 审计 `design.md` 是否已经把 workspace crate、static registry、默认全部内置、动态命令删除、direct adapter CLI 默认删除、local service mode 后置和历史 registration 材料删除写入 Decisions。
- [ ] 1.5 在进入 4.x 实现前收敛 `design.md` Remaining Design Questions 中的 adapter support/orchestration 和最小 adapter layer interface，并把结论同步到 specs 和 tasks。

## 2. 相关在途 change 处置

- [ ] 2.1 审计 `implement-docnav-adapter-management`，将其改写为删除动态注册/制品管理命令，并保留 core release 内置 adapter inspection；完成前不得实现运行时 adapter 制品管理。
- [ ] 2.2 审计 `enable-local-core-adapter-service-mode`，将其重写为 core service 性能、启动成本和缓存策略问题；完成前不得把 local service mode 作为 adapter implementation source。
- [ ] 2.3 审计 `separate-entry-pipeline-from-parameter-resolution`，确认入口分类和 source resolution 以 core release static adapter registry 为 adapter source。

## 3. 主规范和验证材料同步

- [ ] 3.1 更新 `docs/architecture.md`，定义 adapter 选择、制品职责和运行边界为 core release 内置 adapter-layer workspace crates + static registry + adapter-owned code/contract boundary。
- [ ] 3.2 更新 `docs/cli.md`，定义 `docnav adapter list` 为内置 adapter inspection，定义 `doctor` 对 static registry 和 adapter layer 的检查，并删除 `install/register/update/remove` 等 dynamic adapter management commands。
- [ ] 3.3 更新 `docs/adapter-contract.md`，保留 adapter-owned parsing/ref/navigation/native option 边界，说明 adapter direct CLI 或 `invoke` 不再是默认 surface，并按 Remaining Design Questions 结论定义 adapter interface。
- [ ] 3.4 更新 `docs/testing.md` 和必要的 coverage/case 账本，写清 static registry selection、dynamic management command removal、historical registration material removed、protocol/readable output stability 和 ref opacity 的证明目标。
- [ ] 3.5 审查 `docs/schemas/`、`docs/examples/` 和 fixture 是否包含 adapter 制品管理或历史 registration 示例；删除或更新与新契约冲突的验证材料。

## 4. Core release 内置 adapter-layer workspace crates 实现

- [ ] 4.1 将 adapter layer 实现为独立 workspace crates，并作为 core release 的直接组成部分编译；默认 adapter set 不通过 feature gate 裁剪。
- [ ] 4.2 实现 core static adapter registry，统一注册 adapter id、version、format metadata、extensions、content types、capabilities 和 adapter layer implementation。
- [ ] 4.3 将已实现 Markdown adapter 作为 core release 内置 adapter-layer workspace crate 接入 static registry，保持 Markdown parser、ref grammar、navigation strategy、pagination 和 native option 语义仍由 Markdown adapter owner 维护。
- [ ] 4.4 改写 core adapter selection，使 declared adapter、format/content-type hint、extension inference 和 fallback traversal 只使用 static registry candidates。
- [ ] 4.5 将默认 document operation path 的 adapter implementation source 收敛到 static registry 和 adapter layer API。
- [ ] 4.6 按已收敛的 support/orchestration 设计调整现有 SDK，使其不再承担外部 runtime adapter SDK 职责。
- [ ] 4.7 保持 successful document output 的 `protocol-json`、`readable-json` 和 `readable-view` 语义不变，并继续把 adapter-generated ref 原样传递。

## 5. CLI 命令面和历史材料清理

- [ ] 5.1 将 `docnav adapter list` 实现为 static registry metadata inspection，只展示 core release 内置 adapter layer metadata。
- [ ] 5.2 从默认 CLI surface 删除 `adapter install/register/update/remove` 等 dynamic adapter management commands，并更新 help、错误、docs 和 tests。
- [ ] 5.3 更新 `docnav doctor`，检查项目/用户配置、static registry 和 core release 内置 adapter layer 可用性。
- [ ] 5.4 删除历史 adapter registration 配置创建、读取、校验、schema/example/fixture 引用和测试断言。
- [ ] 5.5 更新 diagnostic details 和 readable/protocol failure projection，确保 adapter selection failure 不再引用外部 adapter artifact 或 historical registration 作为可修复路径。

## 6. 测试和验证

- [ ] 6.1 添加或更新 core CLI 测试，覆盖 declared static-registry adapter success、declared missing adapter diagnostic、automatic static-registry adapter discovery、all candidates failed `FORMAT_UNKNOWN`。
- [ ] 6.2 添加 adapter source 边界测试，覆盖 historical adapter registration material 不再参与 document operation implementation source。
- [ ] 6.3 添加命令面测试，覆盖 `adapter list` 内置 adapter metadata inspection，以及 `adapter install/register/update/remove` 不再是有效默认 CLI commands。
- [ ] 6.4 添加 protocol/readable 回归测试，证明 default built-in adapter layer dispatch 后 document success/failure output shape 与对应 schema/example 仍一致。
- [ ] 6.5 添加 SDK/support/orchestration 相关测试，证明 adapter layer 通过指定 interface 和 static registry 接入 core。
- [ ] 6.6 运行范围匹配的格式化、单元、集成和 schema/example 验证；跨 docs、schema、CLI 和 Rust 行为完成后优先运行 `bun run verify:docnav-workspace`。
