# readable-view-output Specification

## Purpose
定义 Docnav document operation 的 readable-view/readable-json 输出契约，包括默认阅读输出、renderer config ownership、block framing、三种 document output mode，以及 readable-view 与 readable-json 从同一 typed readable payload 派生的分层边界。
## Requirements
### Requirement: readable-view 必须成为统一默认阅读输出
`docnav` 和 adapter direct CLI 的 document operation MUST 支持 `readable-view`，并 MUST 在调用方省略 `--output` 时使用该模式。outline、read、find、info、后续组合 operation、成功 warning 和 readable error MUST 使用同一 readable-view 格式和通用 renderer。

#### Scenario: 默认 outline 使用 readable-view
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** stdout 从 pretty JSON header 开始
- **THEN** JSON header 保留 entries、page 和可选 warnings
- **THEN** stdout 不包含 protocol envelope

#### Scenario: 默认 read 使用 readable-view
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref "<ref>"` 且未传入 `--output`
- **THEN** stdout 从 pretty JSON header 开始
- **THEN** JSON header 保留 ref、content 原位置、content_type、cost 和 page
- **THEN** content 字段通过显式 block 引用定位原文 block

#### Scenario: readable error 使用相同 view
- **WHEN** document operation 在 readable-view 模式下返回稳定错误
- **THEN** stdout 使用 readable-view pretty JSON header
- **THEN** JSON header 保留 code、error、details、guidance 和可选 warnings
- **THEN** stdout 只承载该 readable error payload 的 readable-view 表示
- **THEN** guidance 数组保持 header JSON 值，block 替换只应用于 renderer config 声明的字符串字段

### Requirement: renderer config 必须是仓库内契约
Readable-view renderer MUST 使用随仓库提交的 renderer config 声明每个 readable view kind 的 block 字段。block 字段集合 MUST 只来自该仓库内 config；用户配置、项目配置、环境变量、CLI flag 和 adapter manifest 继续承载各自既有职责。Readable-view 稳定契约 MUST 通过字段名和值、block pointer 和 byte length 表达；conformance 对 JSON header object key 顺序和多个 block section 的输出顺序执行顺序无关断言。

#### Scenario: header 语义由字段和值承载
- **WHEN** readable-view renderer 输出 JSON header
- **THEN** header 必须保留 readable payload 的业务字段和值
- **THEN** 调用方、测试和跨语言 conformance 通过字段名和值判断语义
- **THEN** renderer 使用普通 pretty JSON serializer 即可满足该契约

#### Scenario: 未注册扩展字段不丢失
- **WHEN** readable payload 包含 renderer config 未声明为 block 的扩展字段
- **THEN** renderer 保留该字段的业务值
- **THEN** renderer 保持该字段为 header JSON 值
- **THEN** block 替换仍只应用于 renderer config 声明的字段

### Requirement: block 字段必须由 renderer config 显式声明
每个 typed readable result kind MUST 对应一个 renderer config view。renderer MUST 只把 config `blocks` 中列出的 JSON Pointer 替换为 block 引用；未声明字段保持 header JSON 值，即使该字段是长字符串、多行字符串或带有 content type 提示。

#### Scenario: read content 由 config 外置
- **WHEN** readable read renderer config 声明 block pointer `/content`
- **THEN** renderer 将 JSON header 的 content 值替换为 `$block` 引用
- **THEN** renderer 把原 content 字符串写入 `/content` block

#### Scenario: 未声明多行字符串保持 JSON 字符串
- **WHEN** readable payload 中某个未被 renderer config 声明的字符串包含换行
- **THEN** 该字段仍作为合法 JSON 字符串保留在 header 中
- **THEN** block section 只覆盖 renderer config 声明的字段

#### Scenario: config block 错误显式失败
- **WHEN** config block pointer 不存在、目标值不是字符串、pointer 重复或 block identity 冲突
- **THEN** renderer 返回 `readable_view_render_failed`
- **THEN** renderer 使用稳定失败路径，stdout 为空且 stderr 包含诊断

### Requirement: readable-view 必须使用可定界格式
Readable-view MUST 输出一个合法 pretty-printed JSON header。Readable-view framing MUST 在所有平台使用 LF byte `0x0A`。header MUST 以 LF 结束；存在 block section 时，header 结束 LF 后 MUST 再输出一个空 separator LF。被外置字段的 header 值 MUST 为 `{ "$block": "<json-pointer>", "bytes": <utf8-byte-length> }`。每个 block MUST 使用包含 pointer 和 byte length 的起始行、精确 UTF-8 payload、匹配的结束行和 marker 行 LF。

#### Scenario: block 引用保留原字段位置
- **WHEN** renderer 外置 `/content`
- **THEN** JSON header 的 content 字段原位置包含 `$block: "/content"`
- **THEN** header 中的 `bytes` 等于 content 字符串 UTF-8 编码后的字节数

#### Scenario: header 和 block 之间使用固定分隔
- **WHEN** readable-view 输出包含至少一个 block section
- **THEN** pretty JSON header 以 LF byte `0x0A` 结束
- **THEN** header 和第一个 block 起始行之间存在一个空 separator 行
- **THEN** `[block ...]` 和 `[endblock ...]` marker 行以 LF byte `0x0A` 结束

#### Scenario: 无 block 输出只包含 header
- **WHEN** renderer config 对当前 readable kind 使用空 `blocks`
- **THEN** stdout 只包含 pretty JSON header 及其结尾 LF byte `0x0A`
- **THEN** stdout 在该 header 结尾 LF 后结束

#### Scenario: marker 字样不截断正文
- **WHEN** content 自身包含 `[block /content bytes=1]` 或 `[endblock /content]` 文本
- **THEN** renderer 仍按 header 声明的 UTF-8 byte length 输出完整 content
- **THEN** 正文中的 marker 字样不改变 block 边界

#### Scenario: 多字节和无尾换行内容可审计
- **WHEN** content 包含中文、emoji、组合字符或没有尾部换行
- **THEN** byte length 按实际 UTF-8 bytes 计算
- **THEN** renderer 在需要时使用不属于 payload 的 framing LF byte `0x0A` 分隔结束 marker
- **THEN** block payload 还原后与 readable 字段字符串完全一致

#### Scenario: payload CRLF 不影响 framing LF
- **WHEN** content 字符串包含 CRLF 行尾
- **THEN** CRLF 作为 payload bytes 保留并计入 header byte length
- **THEN** readable-view marker 和 separator 仍使用 LF byte `0x0A`

#### Scenario: 多个 block 通过 pointer 和 byte length 定位
- **WHEN** renderer config 声明多个 block pointer
- **THEN** JSON header 在每个字段原位置保留对应 `$block` 引用
- **THEN** 每个 block section 通过 pointer 和 byte length 定位自身
- **THEN** 调用方和测试通过 pointer 和 byte length 匹配 block section
- **THEN** 每个 block 的 pointer 和 byte length 与 header 一致

### Requirement: readable-view 和 readable-json 必须同源
实现 MUST 先构造一个包含 operation readable 字段、稳定 readable error 和可选 warnings 的完整 typed readable payload。`readable-json` MUST 直接序列化该 payload；`readable-view` MUST 只在其 JSON value 上应用 renderer config 指定的 block 替换和 framing。两种输出 MUST 保持相同业务字段和值。

#### Scenario: warning 在两种阅读输出中一致
- **WHEN** 同一成功结果包含 CLI argv warning 或 adapter candidate warning
- **THEN** readable-json 顶层包含完整 warnings 数组
- **THEN** readable-view JSON header 包含语义相同的 warnings 数组
- **THEN** readable-view 的 warning 只由 header 中的 `warnings` 数组承载

#### Scenario: read 字段除 block 表示外一致
- **WHEN** 同一 read 结果分别渲染为 readable-json 和 readable-view
- **THEN** ref、content_type、cost、page 和 warnings 值一致
- **THEN** readable-view 的 `/content` block payload 等于 readable-json 的 content 字符串值

### Requirement: renderer 失败必须有稳定边界
Readable-view renderer MUST 在写 stdout 前完成内存渲染。config 校验、pointer 查找、block 替换或 JSON header 序列化失败时，CLI MUST 输出空 stdout，MUST 在 stderr 输出稳定诊断，MUST 使用 `readable_view_render_failed` 作为错误 id，并 MUST 返回内部错误 exit code。

#### Scenario: config 失败不产生部分 stdout
- **WHEN** readable-view renderer 因 config pointer 指向非字符串字段而失败
- **THEN** stdout 为空
- **THEN** stderr 包含 `readable_view_render_failed`
- **THEN** 命令使用内部错误 exit code 退出
- **THEN** stdout 保持为空

#### Scenario: stdout I/O 错误按既有 I/O 失败处理
- **WHEN** readable-view 已完成内存渲染但写 stdout 失败
- **THEN** CLI 按项目既有 I/O 错误路径退出
- **THEN** 该 I/O 失败使用项目既有 I/O 错误路径表达

### Requirement: renderer config 和 conformance vectors 必须支持跨语言消费
Readable-view renderer config 和 conformance vectors MUST 可供 MCP bridge 等非 Rust 实现消费。Conformance 验证 MUST 聚焦语义字段：block pointer、byte length、block payload 还原、header 字段语义、warning 数组一致和空 block 合法。Conformance 断言 MUST NOT 依赖 JSON header object key 顺序、block section 输出顺序或与 Rust 实现逐字节一致。

#### Scenario: 跨语言 renderer 通过 conformance vectors 验证
- **WHEN** JavaScript renderer 实现根据本 spec 的 renderer config 渲染 readable-view
- **THEN** 跨语言 conformance 验证 block pointer 存在且正确
- **THEN** 跨语言 conformance 验证 `bytes` 等于该字段字符串 UTF-8 编码后的字节数
- **THEN** 跨语言 conformance 验证 block payload 与 readable-json 对应字段一致
- **THEN** 跨语言 conformance 按字段名和值判断 header 语义，不按 key 顺序
- **THEN** 跨语言 conformance 不要求与 Rust renderer 逐字节一致

#### Scenario: 多 block 按 pointer 独立可定位
- **WHEN** renderer config 声明多个 block pointer 且跨语言 conformance 验证
- **THEN** 每个 block 的 pointer 和 byte length 与 header 对应 `$block` 引用一致
- **THEN** conformance 按 pointer 匹配 block，不依赖 block section 输出顺序

### Requirement: document 输出模式必须固定为三种
Document operation 的当前输出模式 MUST 只包含 `readable-view`、`readable-json` 和 `protocol-json`。实现、配置、help、当前主规范、README、skills、examples 和测试 MUST 将 document output mode 声明为这三种。其它 output value 按通用输入错误或配置错误报告。Help、version 和其它非文档纯文本输出 MAY 使用语义明确的 `PlainText` 通道。

#### Scenario: CLI output 值按三种模式校验
- **WHEN** 调用方执行 document operation 并传入无效 `--output` 值
- **THEN** CLI 返回输入错误
- **THEN** help 指向 `readable-view`、`readable-json` 和 `protocol-json`
- **THEN** CLI 在 adapter routing 和 document operation 执行前返回

#### Scenario: 普通执行校验配置 output 值
- **WHEN** 项目或用户配置包含无效 `defaults.output` 值
- **AND** 调用方执行 document operation
- **THEN** 配置校验返回明确错误
- **THEN** 错误包含配置路径、字段路径、收到的值和可接受值
- **THEN** 运行时默认输出仍由 readable-view 提供

#### Scenario: 交付审计确认三种模式
- **WHEN** change 完成交付审计
- **THEN** 核心和 adapter SDK document output implementation 使用 shared readable payload/renderer path
- **THEN** 当前非归档代码和文档只声明 readable-view、readable-json 和 protocol-json 作为 document output mode
- **THEN** 非文档纯文本输出使用 `PlainText` 或等价明确命名，并与 document output mode 类型分离

### Requirement: Document output 编排必须位于 readable rendering 之上

`docnav-output` MUST 拥有 `readable-view`、`readable-json` 和 `protocol-json` 的 document operation output orchestration。`docnav-output` MUST 通过 document-only facade 接收调用方已构造的 operation、request id、output mode、document outcome 和 warnings。`docnav-readable` MUST 继续作为下层 owner，负责 readable payload/value helper、`ReadableViewKind`、renderer config、readable-view block rendering 和 conformance vectors。`docnav-json-io` MUST 作为更下层 helper 负责低层 JSON 写出。`docnav-output` MUST 调用 `docnav-readable` 完成 readable rendering，readable-view block framing 仍由 `docnav-readable` 拥有。

#### Scenario: readable-json 和 readable-view 共享同一个 readable payload

- **WHEN** document operation result 分别渲染为 `readable-json` 和 `readable-view`
- **THEN** `docnav-output` 构造或接收一个包含 operation fields、readable error fields 和 optional warnings 的完整 readable payload
- **THEN** `readable-json` 将该 readable payload 序列化为 JSON
- **THEN** `readable-view` 将同一个 readable value 和 `ReadableViewKind` 传给 `docnav-readable`

#### Scenario: protocol-json 保持 protocol-shaped

- **WHEN** document operation result 渲染为 `protocol-json`
- **THEN** `docnav-output` 向 stdout 写出 protocol response envelope
- **THEN** stdout 对该模式只包含一个 JSON value
- **THEN** 低层 JSON serialization 和 newline writing 可以通过 `docnav-json-io` 完成
- **THEN** warning metadata 不注入 protocol envelope
- **THEN** 需要表达 compatible CLI warnings 时，将其渲染为 stderr diagnostics

#### Scenario: readable renderer contract 仍在 docnav-readable

- **WHEN** readable-view rendering 执行 block pointer replacement、byte length calculation、marker framing 或 conformance vector validation
- **THEN** 该行为继续通过 `docnav-readable` 实现和测试
- **THEN** `docnav-output` 只选择 readable kind、在渲染前注入 warnings，并处理 output channel writing

#### Scenario: 非文档输出保持 owner-specific

- **WHEN** `docnav` 或 adapter 输出 help、version、manifest 或 probe
- **THEN** 该输出不成为 document output mode
- **THEN** 该输出不需要 readable-view framing
- **THEN** 该输出不通过 `docnav-output` 编排
- **THEN** 只有在不改变既有 schema、plain text 或 stderr boundary 时，才可以复用 `docnav-json-io` 或 diagnostics helper
