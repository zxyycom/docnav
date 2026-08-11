本 change 的目标是用仓库内 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式。

## 1. 共享 readable-view 基础

- [x] 1.1 新增 workspace crate `docnav-readable`，定义 readable view kind、仓库内 renderer config 类型、block field JSON Pointer、renderer error 和 conformance vector 结构；protocol envelope、adapter routing 和文档解析继续由既有 owner 负责。
- [x] 1.2 建立完整 typed readable payload 到 JSON value 的单一路径，使 readable-json 直接序列化该 payload，readable-view 在同一 JSON value 上应用 renderer config 指定的 block 替换和 framing。
- [x] 1.3 实现仓库内 renderer config，至少覆盖 outline、read、find、info 和 readable error；read 至少声明 `/content` block，readable error 只声明字符串字段 block，无 block 的 operation 使用空 `blocks`。
- [x] 1.4 实现 readable-view pretty JSON header、平台无关 LF byte `0x0A` framing、header/block separator LF、`{ "$block", "bytes" }` 引用、`[block ...]` / `[endblock ...]` marker LF、payload framing LF 和 UTF-8 byte length 计算；conformance 通过字段、pointer 和 byte length 做顺序无关断言。
- [x] 1.5 实现 renderer config 校验：pointer 缺失、非字符串目标、重复 pointer 和 identity 冲突必须显式失败；renderer 只按 config `blocks` 中声明的 JSON Pointer 选择 block。
- [x] 1.6 实现 renderer 失败边界：写 stdout 前完成内存渲染；render/config 失败收束为 `readable_view_render_failed`、内部错误 exit code、stderr 诊断和空 stdout。
- [x] 1.7 新增 committed conformance vectors 和 Rust tests，覆盖无 block、header/block separator、平台无关 LF byte framing、单 block、多个嵌套 block、空字符串、中文、emoji、组合字符、payload CRLF、无尾换行、正文包含 block marker、warning、readable error、readable error guidance 数组保持 header JSON 值、未注册扩展字段、顺序无关断言和 renderer 失败。
- [x] 1.8 记录 conformance vectors 的跨实现目标为语义一致，断言范围聚焦 block pointer、byte length 和 block payload，并把 header key 顺序、多 block 输出顺序和逐字节一致性排除在稳定语义外。

## 2. 核心 docnav 输出收敛

- [x] 2.1 将核心 document `OutputMode`、clap output enum、preflight output detection、runtime default 和已有 `defaults.output` 内置默认值切换为 `ReadableView`。
- [x] 2.2 让核心成功结果、stable error、CLI argv warning 和 adapter candidate warning 先进入完整 readable payload，再分流到 readable-view 或 readable-json。
- [x] 2.3 为默认和显式 `--output readable-view` 接入共享 renderer，并保持 protocol-json stdout/stderr 边界不变。
- [x] 2.4 将核心 document output implementation 收敛到 shared readable payload/renderer path，使 `OutputMode::Text`、`text_result`、text error/warning writer、document text-specific 分支和核心 document text template 配置不再参与 document output。
- [x] 2.5 将非文档纯文本通道命名为 `PlainText` 或等价明确名称，用于 help、version 和其它非 document operation 诊断，并与 document output mode 类型分离。
- [x] 2.6 让 document `--output` 只接受三种当前模式；help、配置枚举和运行时默认值只展示 readable-view、readable-json 和 protocol-json。
- [x] 2.7 更新已有 `defaults.output` key 的合法值校验，使无效 output value 进入配置错误路径，错误包含配置路径、字段路径、收到的值和可接受值。
- [x] 2.8 更新核心 Rust tests 和黑盒 smoke，覆盖默认 readable-view、显式三种最终 output mode、read block、warning/error header、renderer 失败、配置优先级、无效 output value、help/version PlainText 和 protocol-json 不变。

## 3. Adapter SDK 与 Markdown adapter 输出收敛

- [x] 3.1 让 `docnav-adapter-sdk` 依赖共享 readable crate，并让 direct CLI 成功结果、readable error 和 warning 使用与核心 CLI 相同的 readable payload/rendering 路径。
- [x] 3.2 将 `DirectOutputMode` 默认值和 clap 枚举切换为 `ReadableView`，并保持 readable-json、protocol-json、manifest、probe 和 help 通道边界不变。
- [x] 3.3 将 SDK direct document output surface 收敛到 shared readable payload/renderer path，移除 `DirectOutputMode::Text`、`DirectTextFormatter`、text result/error/warning writer 和 SDK document text-specific formatter 接口。
- [x] 3.4 将 `docnav-markdown` CLI 输出接入 SDK shared renderer path，使 Markdown adapter 代码只保留 parsing、navigation、ref、pagination 和 format-native options 责任。
- [x] 3.5 更新 Markdown CLI smoke 和负向矩阵，覆盖 readable-view header、`/content` block 还原、UTF-8 byte length、marker 内容、warning/error、renderer 失败、顺序无关断言、无效 output value、manifest/probe/help 独立通道和三种最终 document output mode help。
- [x] 3.6 更新其它 adapter SDK Rust tests，证明 direct CLI 和 invoke 仍共享 canonical document operation input，且 protocol/readable-json schema 行为不变。

## 4. MCP 与在途 change 对齐

- [x] 4.1 同步 `implement-docnav-mcp-bridge` 的 proposal、design、spec 和 tasks：TextContent 作为 MCP transport 承载保留，其渲染任务消费本 change 的 readable-view contract、renderer config 和 conformance vectors。
- [x] 4.2 在当前 change 中发布 MCP 可消费的 renderer config/conformance vector 说明；JavaScript renderer、TextContent 输出和 bridge wiring 任务保留在 `implement-docnav-mcp-bridge` change。
- [x] 4.3 同步 `explore-operation-composition` 的 proposal、design、spec 和 tasks：该探索 change 保留归属、筛选标准和后续决策问题；后续 implementation change 按最终 typed readable shape 声明 content pointer 和 renderer config。
- [x] 4.4 检查其它 active change，将未完成 artifacts 中的 document output mode、default output 和 formatter-related 任务对齐为 readable-view、readable-json、protocol-json 三种模式与共享 renderer model。

## 5. 文档、配置和验证材料

- [x] 5.1 更新 `docs/cli.md`、`docs/architecture.md`、`docs/adapter-contract.md`、`docs/protocol.md` 和 `docs/testing.md`，定义最终三种 document output mode、readable-view 格式、仓库内 renderer config、静态 block 字段、warning/error、配置校验和 MCP handoff。
- [x] 5.2 更新 `docs/CODING_STYLE.md`，要求 readable-view/readable-json 从同一 readable payload 派生，要求 renderer config 只声明 block 字段、稳定语义以字段和 block 引用表达，并要求 operation 或 adapter document output 通过共享 readable payload/renderer 表达。
- [x] 5.3 更新 README、docs navigation/术语、schema/example 索引、项目 skills 和命令示例：默认阅读命令使用 readable-view，需要结构化消费时显式使用 readable-json。
- [x] 5.4 新增 readable-view golden fixtures 和格式说明，覆盖 outline、read、find、info、error、warning、平台无关 LF byte framing、header/block separator、多 block framing、顺序无关断言、未注册扩展字段和 renderer 失败；readable JSON schema 与 protocol schema 保持不变。
- [x] 5.5 将 document text 模板配置、text golden fixtures、text smoke case 和当前非归档文档示例改为 readable-view/readable-json 验收材料；无效 output value 只作为输入错误用例出现。
- [x] 5.6 更新 CLI/output mode 一致性验证，确认 help、配置枚举、Rust constants、主规范、golden fixtures 和测试矩阵只把 readable-view、readable-json 和 protocol-json 声明为 document output mode，同时允许非文档 `PlainText` 通道存在。

## 6. 验证与交付审计

- [x] 6.1 运行 `cargo fmt --all --check` 和受影响 crates 的 Rust unit/integration tests。
- [x] 6.2 运行核心 CLI、Markdown adapter 和相关 Node.js smoke/conformance tests，确认 stdout、stderr、exit code、block byte framing、renderer failure、配置校验和顺序无关断言。
- [x] 6.3 运行 schema、example、文档和 OpenSpec 严格校验，确认 readable-json/protocol-json 既有校验继续通过。
- [x] 6.4 用限定路径搜索审计当前非归档 surface：代码、README、主文档、adapter 文档、examples/schema 索引、项目 skills、smoke scripts、tests 和 active changes 均与三种 document output mode 及共享 renderer model 对齐；document text 相关符号和示例只保留在本 change 的收敛说明或归档审计中，非文档 `PlainText` 不计为 document output mode。
- [x] 6.5 运行 `pnpm run verify:docnav-workspace`，并用局部 diff 确认实现只修改 readable output、相关 docs/tests、MCP handoff 和在途 change 对齐范围。
