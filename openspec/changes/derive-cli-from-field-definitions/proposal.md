本 change 让 canonical field declarations 通过通用 consumer extension metadata 承载 Docnav CLI 信息，并由项目专属 builder 和 projection 统一产出 document CLI 注册、help、candidate extraction 与 selected-field validation。普通执行同时改为 stage-scoped input processing：每个阶段只允许当前 projection 内的配置事实产生结果或诊断，projection 外内容即使被共享实现顺便扫描，也不得影响本次执行。在实现、owner docs 同步和验收完成前，本目录只描述目标变更，不代表当前二进制已经支持。

## Why

Document CLI 的通用参数和 adapter native options 目前在字段声明、Clap 构造、argv 扫描、字符串解码与 navigation 之间重复维护。新增或修改字段需要同步 flag、类型、合法值、默认值、help、owner 和 handler binding，容易让 help、解析与校验逐渐分叉。

`FieldDef` 已拥有语义字段事实，但缺少由 consumer 声明和读取项目专属元信息的通用扩展面。目标是让字段声明成为唯一 authoring source：typed-fields 保持通用，Docnav 在项目层补充 CLI presentation，并让普通 invocation 只处理 routing 与 selected operation 实际需要的字段。

## What Changes

- 为 typed-fields 增加 immutable、type-indexed consumer extension metadata，使其跨 declaration / `FieldDefSet` aggregation 保留并可从 built `FieldDef` typed retrieval，而不解释 payload 语义。
- 新增 framework-neutral 的 `docnav-field-authoring` shared crate，集中定义 Docnav field builder 扩展与 projection；CLI presentation 在字段声明处补充，canonical constraints/defaults 继续由字段本身拥有。Docnav core 把该 view 机械映射为 Clap companion 自己拥有的 projection input，保持 crate 依赖方向单向。
- 按 document operation 从声明生成 registry CLI projection，并用它扩展唯一的 authoritative Clap command tree。Clap 处理 command shape，companion 把已注册输入转换为 typed 或 field-local invalid candidates。
- Navigation 在 adapter selection 后重组 current-operation `FieldDefSet`；只有所选字段进入 resolution、request construction 与 dispatch。
- 普通 document invocation 按阶段读取配置：core-owned logging、routing、outline policy 和 selected operation 各自只校验当前 projection；其它内容不产生本次调用的诊断。`docnav config inspect` 使用 registry-wide projection 检查完整 source。
- 完成 hard cutover，移除旧的并行业务解析路径与运行时 fallback。

## Compatibility

- 保持 commands、flags、defaults、source priority、merge、adapter selection、protocol/readable/ref payload 和 adapter format behavior。
- **BEHAVIOR CHANGE**：普通 document invocation 只报告当前阶段 projection 内的配置错误。Logging 阶段只处理 logging fields，routing 阶段只处理 routing fields，outline policy 只在 outline 阶段处理，selected operation 只处理其 common/adapter fields；projection 只包含这些字段路径和读取它们所必需的结构祖先，不把完整 object 当作本阶段 schema。Projection 外内容不读取、不校验、不报告；底层即使为了复用而顺便生成其它 validation facts，也必须在阶段边界丢弃。完整 source 检查进入 `docnav config inspect`。
- Authoritative Clap parse 仍全局处理 command shape：unknown flag、duplicate single-value input 和 missing value 在 adapter selection 前失败。只有 structural parse 成功后形成的 typed/invalid registry candidates 才按 selected `FieldDefSet` 过滤；未选中 candidate 不产生 field diagnostic。
- Missing explicit config path、unreadable file、invalid JSON 和 top-level non-object 继续按现有 source-level 规则失败。
- `docnav version` command 保持不变。
- **BREAKING**：authoritative command parse、duplicate/invalid output，或 valid output 尚未确定时的 failure 使用 PlainText，不再从 raw argv 推测 output mode。Valid explicit output 控制其后的 failure；config/default output 只在 normal navigation resolution 成功后成为 output context。以 `-` 开头的 string/path option value 使用 `--flag=<value>`。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-architecture`：`docnav-field-authoring` 的项目 authoring 职责与单向依赖边界。
- `typed-fields`：immutable type-indexed consumer extension metadata、project builder 扩展所需的保存与 typed retrieval 边界。
- `core-cli`：authoritative Clap tree、generated document fields、core-owned stage projection 和 parse/output failure mapping。
- `cli-config-resolution`：从 canonical fields 与派生 presentation 生成 Clap projection 和 typed/invalid candidates。
- `navigation-input-resolution`：registry projection、stage-scoped config processing、selected `FieldDefSet` 重组和 selected-only resolution。
- `adapter-contract`：adapter declaration 通过项目 field builder 声明 CLI presentation。
- `output-contract`：只有已成功确定的 document output mode 才参与 failure projection，early parse failure 使用 PlainText。

## Impact

- Docnav 集成层：新增 `crates/shared/field-authoring/`，修改 `crates/docnav/src/cli/**`、`crates/shared/navigation/src/parameters/**`、`crates/shared/adapter-contracts/src/native_option/**`、built-in adapter declarations，以及删除 `crates/shared/cli-args/`。
- 共享 crate：`crates/shared/typed-fields/**` 与 `crates/shared/cli-config-resolution-clap/**` 的 extension metadata、projection、candidate extraction、errors、tests、README 和 example。
- 验证：typed-fields/CLI/navigation/adapter/output owner docs、CLI 与 config inspection tests、companion contract tests、process smoke、case 账本、workspace verifier 和 OpenSpec strict validation。

## Non-Goals

- 不在 typed-fields core 中定义 Clap、Docnav operation、adapter 或 presentation payload；consumer extension 的含义由项目层拥有。
- 不增加 candidate usage accounting，也不从 native option 反向推断 adapter。
- 不改变 config source 文件加载、source priority、merge、default、required、materialization、handler binding 或 resolver public API。
- 不要求按 JSON section 实现物理 lazy parser；config source 可以一次形成 object，但每个阶段只能暴露自己 projection 内的结果和诊断。
- 不迁移 config-only outline selectors；它们继续只在 outline policy 阶段按现有 owner 处理。不改变 protocol/schema shape、adapter format semantics 或 Markdown 性能。
- 不新增 plugin、DI、custom parser 或通用 lifecycle framework。
