本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 tasks，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## 0. 阻塞级审计门禁

- [ ] 0.1 用户审计确认：用户已审计本次 proposal、design、specs 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。
- [ ] 0.2 审计 proposal、design、specs 和 tasks 是否都围绕“readable-view 替代 document text 输出模式”这一核心句，确认当前 change 只包含 `openspec/changes/replace-text-with-readable-view/` 下的未审核临时 artifacts，且创建阶段没有修改现有主规范、代码、schema、examples 或其它 change。
- [ ] 0.3 审计 `add-fast-outline` 和 `implement-docnav-mcp-bridge` 的当前状态与输出假设，记录需要同步的默认文本、TextContent 和 readable-view integration 项；实现不得让在途 change 重新引入 document text 模式或独立 text formatter。
- [ ] 0.4 在开始实现 MCP 相关代码前，同步 `implement-docnav-mcp-bridge` 的 artifacts，使其依赖本 change 提供的 readable-view contract、renderer config 和 conformance vectors；当前 change 不实现 JavaScript MCP renderer。

## 1. 共享 readable-view 基础

- [ ] 1.1 新增 workspace crate `docnav-readable`，定义格式版本常量、readable view kind、仓库内 renderer config 类型、block field JSON Pointer、renderer error 和 conformance vector 结构；crate 不拥有 protocol envelope、adapter routing 或文档解析。
- [ ] 1.2 建立完整 typed readable payload 到 JSON value 的单一路径，使 readable-json 直接序列化该 payload，readable-view 在同一 JSON value 上应用 renderer config 指定的 block 替换和 framing。
- [ ] 1.3 实现仓库内版本化 renderer config，至少覆盖 outline、read、find、info 和 readable error；read 至少声明 `/content` block，无 block 的 operation 使用空 `blocks`。
- [ ] 1.4 实现 `@docnav-readable-view/1`、pretty JSON header、`{ "$block", "bytes" }` 引用、`[block ...]` / `[endblock ...]` framing 和 UTF-8 byte length 计算；header key 顺序和多 block 输出顺序不作为稳定契约。
- [ ] 1.5 实现 renderer config 校验：pointer 缺失、非字符串目标、重复 pointer 和 identity 冲突必须显式失败；renderer 不按换行、长度、content type 或用户配置自动选择 block。
- [ ] 1.6 实现 renderer 失败边界：写 stdout 前完成内存渲染；render/config 失败使用 `readable_view_render_failed`、内部错误 exit code、stderr 诊断、空 stdout，且不得 fallback 到 readable-json、protocol-json、plain text、旧 text formatter 或递归 readable-view。
- [ ] 1.7 新增 committed conformance vectors 和 Rust tests，覆盖无 block、单 block、多个嵌套 block、空字符串、中文、emoji、组合字符、CRLF、无尾换行、正文包含 block marker、warning、readable error、未注册扩展字段、顺序无关断言和 renderer 失败。
- [ ] 1.8 记录 conformance vectors 的跨实现目标为语义一致，包括格式版本、block pointer、byte length 和 block payload；不要求 header key 顺序、多 block 输出顺序或 Rust 与后续 JavaScript renderer 逐字节一致。

## 2. 核心 docnav 输出迁移

- [ ] 2.1 将核心 document `OutputMode`、clap output enum、preflight output detection、runtime default 和 `defaults.output` 内置默认值切换为 `ReadableView`。
- [ ] 2.2 让核心成功结果、stable error、CLI argv warning 和 adapter candidate warning 先进入完整 readable payload，再分流到 readable-view 或 readable-json。
- [ ] 2.3 为默认和显式 `--output readable-view` 接入共享 renderer，并保持 protocol-json stdout/stderr 边界不变。
- [ ] 2.4 删除 document `OutputMode::Text`、`text_result`、text error/warning writer、text fallback 和核心 document text template 配置。
- [ ] 2.5 保留或重命名非文档纯文本通道为 `PlainText` 或等价明确名称，用于 help、version 和其它非 document operation 诊断；不得通过 document output mode 暴露。
- [ ] 2.6 让 `--output text` 返回明确输入错误，不提供 alias、fallback 或静默迁移。
- [ ] 2.7 实现 legacy `defaults.output: "text"` 普通执行拒绝，错误包含配置路径、字段路径、legacy value 和修复命令。
- [ ] 2.8 为 `docnav config set/unset` 增加 migration-safe raw target config loader，使 `config set defaults.output readable-view`、`config unset defaults.output` 及其 `--user` 变体能修复目标 scope 中的 legacy text；其它配置字段仍严格校验。
- [ ] 2.9 更新核心 Rust tests 和黑盒 smoke，覆盖默认 readable-view、显式三种最终 output mode、read block、warning/error header、renderer 失败、配置优先级、非法 `--output text`、legacy config 普通执行失败、config set/unset 修复路径、help/version PlainText 和 protocol-json 不变。

## 3. Adapter SDK 与 Markdown adapter 迁移

- [ ] 3.1 让 `docnav-adapter-sdk` 依赖共享 readable crate，并让 direct CLI 成功结果、readable error 和 warning 使用与核心 CLI 相同的 readable payload/rendering 路径。
- [ ] 3.2 将 `DirectOutputMode` 默认值和 clap 枚举切换为 `ReadableView`，并保持 readable-json、protocol-json、manifest、probe 和 help 通道边界不变。
- [ ] 3.3 删除 document `DirectOutputMode::Text`、`DirectTextFormatter`、text result/error/warning writer 和所有 SDK document text-specific formatter 接口。
- [ ] 3.4 删除 `docnav-markdown` 私有 document text formatter 及相关常量，只保留 Markdown parsing、navigation、ref、pagination 和 format-native options 责任。
- [ ] 3.5 更新 Markdown CLI smoke 和负向矩阵，覆盖 readable-view header、`/content` block 还原、UTF-8 byte length、marker 内容、warning/error、renderer 失败、顺序无关断言、非法 `--output text`、manifest/probe/help 独立通道和三种最终 document output mode help。
- [ ] 3.6 更新其它 adapter SDK Rust tests，证明 direct CLI 和 invoke 仍共享 canonical document operation input，且 protocol/readable-json schema 行为不变。

## 4. MCP 与在途 change 对齐

- [ ] 4.1 同步 `implement-docnav-mcp-bridge` 的 proposal、design、spec 和 tasks：保留 TextContent transport 类型，删除“精简文本 formatter”语义，改为依赖本 change 的 readable-view contract、renderer config 和 conformance vectors。
- [ ] 4.2 在当前 change 中发布 MCP 可消费的 renderer config/conformance vector 说明；JavaScript renderer、TextContent 输出和 bridge wiring 任务保留在 `implement-docnav-mcp-bridge` change。
- [ ] 4.3 同步 `add-fast-outline` 的 proposal、design、spec 和 tasks：默认输出使用 readable-view，read mode 的 content pointer 由最终 typed readable shape 的 renderer config 声明，outline mode 使用空或对应显式 config。
- [ ] 4.4 检查其它 active change，确保未完成 artifacts 不再把 document operation `text` 作为支持模式或实现任务。

## 5. 文档、配置和验证材料

- [ ] 5.1 更新 `docs/cli.md`、`docs/architecture.md`、`docs/adapter-contract.md`、`docs/protocol.md` 和 `docs/testing.md`，定义最终三种 document output mode、readable-view 格式、仓库内 renderer config、静态 block 字段、warning/error、legacy config 修复路径和 MCP handoff。
- [ ] 5.2 更新 `docs/CODING_STYLE.md`，要求 readable-view/readable-json 从同一 readable payload 派生，要求 renderer config 只声明 block 字段且不承诺 header key 顺序，并禁止 operation 或 adapter 私有 document text formatter。
- [ ] 5.3 更新 README、docs navigation/术语、schema/example 索引、项目 skills 和命令示例，将默认阅读命令迁移到 readable-view 或在需要结构化消费时显式使用 readable-json。
- [ ] 5.4 新增 readable-view golden fixtures 和格式说明，覆盖 outline、read、find、info、error、warning、多 block framing、顺序无关断言、未注册扩展字段和 renderer 失败；readable JSON schema 与 protocol schema 保持不变。
- [ ] 5.5 删除或迁移 document text 模板配置、text golden fixtures、text smoke case 和当前非归档文档中的 `--output text` 示例。
- [ ] 5.6 更新 CLI/output mode 一致性验证，确认 help、配置枚举、Rust constants、主规范、golden fixtures 和测试矩阵只把 readable-view、readable-json 和 protocol-json 声明为 document output mode，同时允许非文档 `PlainText` 通道存在。

## 6. 验证与交付审计

- [ ] 6.1 运行 `cargo fmt --all --check` 和受影响 crates 的 Rust unit/integration tests。
- [ ] 6.2 运行核心 CLI、Markdown adapter 和相关 Node.js smoke/conformance tests，确认 stdout、stderr、exit code、block byte framing、renderer failure、legacy config repair 和顺序无关断言。
- [ ] 6.3 运行 schema、example、文档和 OpenSpec 严格校验，确认 readable-json/protocol-json 既有校验继续通过。
- [ ] 6.4 用限定路径搜索确认当前代码和主文档不存在 document `OutputMode::Text`、`DirectOutputMode::Text`、`DirectTextFormatter`、document operation `--output text` 或 document text template 配置；归档历史和本 change 的迁移说明不计为残留，非文档 `PlainText` 不计为残留。
- [ ] 6.5 运行 `pnpm run verify:docnav-workspace`，并用局部 diff 确认实现只修改 readable output、相关 docs/tests、MCP handoff 和在途 change 对齐范围。
