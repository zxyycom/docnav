本 tasks 清单定义实现 core release 内置 adapter-layer workspace crates change 的执行顺序、相关 change 处置和验证范围。

## 1. 相关在途 change 处置

- [x] 1.1 改写 `implement-docnav-adapter-management`，将其收敛为删除动态注册/制品管理命令，并保留 core release 内置 adapter inspection；完成前不得实现运行时 adapter 制品管理。
- [x] 1.2 改写 `enable-local-core-adapter-service-mode`，将其重写为 core service 性能、启动成本和缓存策略问题；完成前不得把 local service mode 作为 adapter implementation source。
- [x] 1.3 更新 `separate-entry-pipeline-from-parameter-resolution`，确认入口分类和 source resolution 以 core release static adapter registry 为 adapter implementation source boundary。

## 2. 主规范和验证材料同步

- [x] 2.1 更新 `docs/architecture.md`，定义 adapter 制品职责和运行边界为 core release 内置 adapter-layer workspace crates + static registry + adapter-owned code/contract boundary。
- [x] 2.2 更新 `docs/cli.md`，定义 `docnav adapter list` 为内置 adapter inspection，定义 `doctor` 对 static registry 和 adapter layer 的检查，并删除 `install/register/update/remove` 等 dynamic adapter management commands。
- [x] 2.3 更新 `docs/adapter-contract.md`，保留 adapter-owned parsing/ref/navigation/native option validation 边界，说明 source-level static native option registry、generic merge 和 adapter option handoff，说明 adapter direct CLI、`invoke` 或非默认 adapter 本地调试入口不再是默认 surface，并按 `docnav-navigation` + `docnav-adapter-contracts` 和 operation-handler granularity 结论定义 adapter interface。
- [x] 2.4 更新 `docs/testing.md` 和必要的 coverage/case 账本，写清 built-in adapter source boundary、dynamic management command removal、historical registration material removed、protocol/readable output stability 和 ref opacity 的证明目标。
- [x] 2.5 检查 `docs/schemas/`、`docs/examples/` 和 fixture 是否包含 adapter 制品管理或历史 registration 示例；删除或更新与新契约冲突的验证材料。

## 3. Core release 内置 adapter-layer workspace crates 实现

- [x] 3.1 将 adapter layer 实现为独立 workspace crates，并作为 core release 的直接组成部分编译；默认 adapter set 不通过 feature gate 裁剪。
- [x] 3.2 实现 core static adapter registry，统一注册 adapter id、version、format metadata、native option registry、operation handler bindings 和 adapter layer implementation handle。
- [x] 3.3 将已实现 Markdown adapter 作为 core release 内置 adapter-layer workspace crate 接入 static registry，保持 Markdown parser、ref grammar、navigation strategy、pagination 和 native option 语义仍由 Markdown adapter owner 维护。
- [x] 3.4 将默认 document operation path 的 adapter implementation source 收敛到 static registry 和 adapter layer API；CLI 输入和有效项目/用户配置只能通过 `--adapter` 或 `defaults.adapter` 声明 registry 中已有 adapter id。
- [x] 3.5 移除默认路径中对独立 adapter package、external executable、command path 和 historical artifact record 的读取、校验和 fallback。
- [x] 3.6 将现有 SDK 中仍有价值的内部类型和工具迁移到 `docnav-navigation` 或 `docnav-adapter-contracts`；默认不新增 `docnav-adapter-support`，除非实现证明重复工具会污染 contract boundary。
- [x] 3.7 保持 successful document output 的 `protocol-json`、`readable-json` 和 `readable-view` 语义不变，并继续把 adapter-generated ref 原样传递。

## 4. CLI 命令面和历史材料清理

- [x] 4.1 将 `docnav adapter list` 实现为 static registry metadata inspection，只展示 core release 内置 adapter layer metadata。
- [x] 4.2 从默认 CLI surface 删除 `adapter install/register/update/remove` 等 dynamic adapter management commands，并更新 help、错误、docs 和 tests。
- [x] 4.3 更新 `docnav doctor`，检查项目/用户配置、static registry 和 core release 内置 adapter layer 可用性。
- [x] 4.4 删除历史 adapter registration 配置创建、读取、校验、schema/example/fixture 引用和测试断言。
- [x] 4.5 更新 diagnostic/readable failure guidance，确保 adapter selection failure 不再把外部 adapter artifact 或 historical registration 作为默认修复路径。

## 5. 内部 navigation / adapter interface 收敛

- [x] 5.1 创建或调整 `docnav-navigation`，作为内部 operation orchestration layer，集中调配 `outline/read/find/info`。
- [x] 5.2 创建或调整 `docnav-adapter-contracts`，承载 adapter layer interface definitions 和共享 contract types；adapter crates 不依赖完整 operation orchestration。
- [x] 5.3 采用 operation-handler adapter interface：adapter handle 暴露 static descriptor metadata、probe check、source-level native option registry entries 和 `outline/read/find/info` operation handlers；记录 primitive split 会把格式内部结构和 orchestration glue 扩散到 adapter/core 边界且当前没有产品收益。
- [x] 5.4 不保留 adapter direct CLI、`invoke` 或非默认 adapter 本地调试入口；相关证明只通过黑盒 CLI 测试、白盒 adapter/core 测试和 core 调用路径完成。

## 6. 测试和验证

- [x] 6.1 添加或更新 core CLI 测试，覆盖 default built-in adapter source success、declared missing adapter id failure、external adapter artifact 不参与 fallback。
- [x] 6.2 添加 adapter source 边界测试，覆盖 historical adapter registration material 不再参与 document operation implementation source。
- [x] 6.3 添加命令面测试，覆盖 `adapter list` 内置 adapter metadata inspection，以及 `adapter install/register/update/remove` 不再是有效默认 CLI commands。
- [x] 6.4 添加 protocol/readable 回归测试，证明 default built-in adapter layer dispatch 后 document success/failure output shape 与对应 schema/example 仍一致。
- [x] 6.5 添加 `docnav-navigation`、`docnav-adapter-contracts`、static descriptor/source-level native option registry、generic merge、adapter-side option validation 和 operation-handler adapter interface 相关测试。
- [x] 6.6 运行范围匹配的格式化、单元、集成和 schema/example 验证；跨 docs、schema、CLI 和 Rust 行为完成后优先运行 `bun run verify:docnav-workspace`。
