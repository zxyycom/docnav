本 change 让 canonical field declarations 通过通用 consumer extension metadata 承载 Docnav CLI 信息，并由项目专属 builder 和 projection 统一产出 document CLI 注册、help、candidate extraction 与 selected-field validation。在实现、owner docs 同步和验收完成前，本目录只描述目标变更，不代表当前二进制已经支持。

## Why

Document CLI 的通用参数和 adapter native options 目前在字段声明、Clap 构造、argv 扫描、字符串解码与 navigation 之间重复维护。新增或修改字段需要同步 flag、类型、合法值、默认值、help、owner 和 handler binding，容易让 help、解析与校验逐渐分叉。

`FieldDef` 已拥有语义字段事实，但缺少由 consumer 声明和读取项目专属元信息的通用扩展面。目标是让字段声明成为唯一 authoring source：typed-fields 保持通用，Docnav 在项目层补充 CLI presentation，并让普通 invocation 只处理 routing 与 selected operation 实际需要的字段。

## What Changes

- 为 typed-fields 增加 opaque consumer extension metadata，支持随 declaration / `FieldDefSet` 保存、显式替换与 typed retrieval，而不解释 payload 语义。
- 新增 framework-neutral 的 `docnav-field-authoring` shared crate，集中定义 Docnav field builder 扩展与 projection；CLI presentation 在字段声明处补充，canonical constraints/defaults 继续由字段本身拥有。
- 按 document operation 从声明生成 registry CLI projection，并用它扩展唯一的 authoritative Clap command tree。Clap 处理 command shape，companion 把已注册输入转换为 typed 或 field-local invalid candidates。
- Navigation 在 adapter selection 后重组 current-operation `FieldDefSet`；只有所选字段进入 resolution、request construction 与 dispatch。
- 普通 document invocation 只读取 routing 与 selected-operation config fields；`docnav config inspect` 使用 registry-wide projection 检查完整 source。
- 完成 hard cutover，移除旧的并行业务解析路径与运行时 fallback。

## Compatibility

- 保持 commands、flags、defaults、source priority、merge、adapter selection、protocol/readable/ref payload 和 adapter format behavior。
- **BEHAVIOR CHANGE**：普通 document invocation 只报告 selected fields 的错误。未进入 selected field set 的 registry CLI candidates，以及配置中未被 selected projection 读取的内容，不影响本次执行；完整检查进入 `docnav config inspect`。
- Missing explicit config path、unreadable file、invalid JSON 和 top-level non-object 继续按现有 source-level 规则失败。
- `docnav version` command 保持不变。
- **BREAKING**：authoritative command parse、duplicate/invalid output，或 valid output 尚未确定时的 failure 使用 PlainText，不再从 raw argv 推测 output mode。以 `-` 开头的 string/path option value 使用 `--flag=<value>`。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `docnav-architecture`：`docnav-field-authoring` 的项目 authoring 职责与单向依赖边界。
- `typed-fields`：opaque consumer extension metadata、project builder 扩展所需的保存与 typed retrieval 边界。
- `core-cli`：authoritative Clap tree、generated document fields 和 parse/output failure mapping。
- `cli-config-resolution`：从 canonical fields 与派生 presentation 生成 Clap projection 和 typed/invalid candidates。
- `navigation-input-resolution`：registry projection、selected `FieldDefSet` 重组和 selected-only resolution。
- `adapter-contract`：adapter declaration 通过项目 field builder 声明 CLI presentation。
- `output-contract`：只有已成功确定的 document output mode 才参与 failure projection，early parse failure 使用 PlainText。

## Impact

- 主仓库：新增 `crates/shared/field-authoring/`，修改 `crates/docnav/src/cli/**`、`crates/shared/navigation/src/parameters/**`、`crates/shared/adapter-contracts/src/native_option/**`、built-in adapter declarations，以及删除 `crates/shared/cli-args/`。
- 子仓库：`subrepos/cli-config-resolution/crates/typed-fields/**` 与 `crates/cli-config-resolution-clap/**` 的 extension metadata、projection、candidate extraction、errors、tests、README 和 example。
- 验证：typed-fields/CLI/navigation/adapter/output owner docs、CLI 与 config inspection tests、companion contract tests、process smoke、case 账本、workspace verifier 和 OpenSpec strict validation。

## Non-Goals

- 不在 typed-fields core 中定义 Clap、Docnav operation、adapter 或 presentation payload；consumer extension 的含义由项目层拥有。
- 不增加 candidate usage accounting，也不从 native option 反向推断 adapter。
- 不改变 config source 文件加载、source priority、merge、default、required、materialization、handler binding 或 resolver public API。
- 不迁移 config-only outline selectors，不改变 protocol/schema shape、adapter format semantics 或 Markdown 性能。
- 不新增 plugin、DI、custom parser 或通用 lifecycle framework。
