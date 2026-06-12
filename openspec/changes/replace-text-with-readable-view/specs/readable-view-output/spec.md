本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 delta spec，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## ADDED Requirements

### Requirement: readable-view 必须成为统一默认阅读输出
`docnav` 和 adapter direct CLI 的 document operation MUST 支持 `readable-view`，并 MUST 在调用方省略 `--output` 时使用该模式。outline、read、find、info、后续组合 operation、成功 warning 和 readable error MUST 使用同一 readable-view 格式版本和通用 renderer。

#### Scenario: 默认 outline 使用 readable-view
- **WHEN** 调用方执行 `docnav outline docs/guide.md` 且未传入 `--output`
- **THEN** stdout 第一行是 `@docnav-readable-view/1`
- **THEN** JSON header 保留 entries、page 和可选 warnings
- **THEN** stdout 不包含 protocol envelope

#### Scenario: 默认 read 使用 readable-view
- **WHEN** 调用方执行 `docnav read docs/guide.md --ref "<ref>"` 且未传入 `--output`
- **THEN** stdout 第一行是 `@docnav-readable-view/1`
- **THEN** JSON header 保留 ref、content 原位置、content_type、cost 和 page
- **THEN** content 字段通过显式 block 引用定位原文 block

#### Scenario: readable error 使用相同 view
- **WHEN** document operation 在 readable-view 模式下返回稳定错误
- **THEN** stdout 使用 `@docnav-readable-view/1`
- **THEN** JSON header 保留 code、error、details、guidance 和可选 warnings
- **THEN** renderer 不追加独立错误文本模板

### Requirement: renderer config 必须是仓库内版本化契约
Readable-view renderer MUST 使用随仓库提交的版本化 renderer config 声明每个 readable view kind 的 block 字段。该 config MUST NOT 从用户配置、项目配置、环境变量、CLI flag 或 adapter manifest 读取。Readable-view MUST NOT 把 JSON header object key 顺序或多个 block section 的输出顺序定义为稳定契约。

#### Scenario: header key 顺序不作为契约
- **WHEN** readable-view renderer 输出 JSON header
- **THEN** header 必须保留 readable payload 的业务字段和值
- **THEN** 调用方、测试和跨语言 conformance 不得要求 header key 按固定顺序输出
- **THEN** renderer 不需要实现 canonical JSON serializer

#### Scenario: 未注册扩展字段不丢失
- **WHEN** readable payload 包含 renderer config 未声明为 block 的扩展字段
- **THEN** renderer 保留该字段的业务值
- **THEN** renderer 不要求该字段在 header 中具有稳定位置
- **THEN** renderer 不把该字段自动变为 block

### Requirement: block 字段必须由 renderer config 显式声明
每个 typed readable result kind MUST 对应一个 renderer config view。只有 config `blocks` 中列出的 JSON Pointer 可以被替换为 block 引用；renderer MUST NOT 根据字符串包含换行、字符串长度、content type 或用户配置自动选择 block 字段。

#### Scenario: read content 由 config 外置
- **WHEN** readable read renderer config 声明 block pointer `/content`
- **THEN** renderer 将 JSON header 的 content 值替换为 `$block` 引用
- **THEN** renderer 把原 content 字符串写入 `/content` block

#### Scenario: 未声明多行字符串保持 JSON 字符串
- **WHEN** readable payload 中某个未被 renderer config 声明的字符串包含换行
- **THEN** 该字段仍作为合法 JSON 字符串保留在 header 中
- **THEN** renderer 不为该字段自动创建 block

#### Scenario: config block 错误显式失败
- **WHEN** config block pointer 不存在、目标值不是字符串、pointer 重复或 block identity 冲突
- **THEN** renderer 返回 `readable_view_render_failed`
- **THEN** renderer 不静默保留原字符串、不跳过字段且不回退为旧 text 输出

### Requirement: readable-view 必须使用可定界的版本化格式
Readable-view MUST 以 `@docnav-readable-view/1` 开始，随后输出一个合法 pretty-printed JSON header。被外置字段的 header 值 MUST 为 `{ "$block": "<json-pointer>", "bytes": <utf8-byte-length> }`。每个 block MUST 使用包含 pointer 和 byte length 的起始行、精确 UTF-8 payload 和匹配的结束行。

#### Scenario: block 引用保留原字段位置
- **WHEN** renderer 外置 `/content`
- **THEN** JSON header 的 content 字段原位置包含 `$block: "/content"`
- **THEN** header 中的 `bytes` 等于 content 字符串 UTF-8 编码后的字节数

#### Scenario: marker 字样不截断正文
- **WHEN** content 自身包含 `[block /content bytes=1]` 或 `[endblock /content]` 文本
- **THEN** renderer 仍按 header 声明的 UTF-8 byte length 输出完整 content
- **THEN** 正文中的 marker 字样不改变 block 边界

#### Scenario: 多字节和无尾换行内容可审计
- **WHEN** content 包含中文、emoji、组合字符或没有尾部换行
- **THEN** byte length 按实际 UTF-8 bytes 计算
- **THEN** renderer 在需要时使用不属于 payload 的 framing LF 分隔结束 marker
- **THEN** block payload 还原后与 readable 字段字符串完全一致

#### Scenario: 多个 block 不依赖输出顺序
- **WHEN** renderer config 声明多个 block pointer
- **THEN** JSON header 在每个字段原位置保留对应 `$block` 引用
- **THEN** 每个 block section 通过 pointer 和 byte length 定位自身
- **THEN** 调用方和测试不得依赖多个 block section 的相对输出顺序
- **THEN** 每个 block 的 pointer 和 byte length 与 header 一致

### Requirement: readable-view 和 readable-json 必须同源
实现 MUST 先构造一个包含 operation readable 字段、稳定 readable error 和可选 warnings 的完整 typed readable payload。`readable-json` MUST 直接序列化该 payload；`readable-view` MUST 只在其 JSON value 上应用 renderer config 指定的 block 替换和 framing。两种输出 MUST 保持相同业务字段和值。

#### Scenario: warning 在两种阅读输出中一致
- **WHEN** 同一成功结果包含 CLI argv warning 或 adapter candidate warning
- **THEN** readable-json 顶层包含完整 warnings 数组
- **THEN** readable-view JSON header 包含语义相同的 warnings 数组
- **THEN** readable-view 不在 block 后重复拼接 warning 文本

#### Scenario: read 字段除 block 表示外一致
- **WHEN** 同一 read 结果分别渲染为 readable-json 和 readable-view
- **THEN** ref、content_type、cost、page 和 warnings 值一致
- **THEN** readable-view 的 `/content` block payload 等于 readable-json 的 content 字符串值

### Requirement: renderer 失败必须有稳定边界
Readable-view renderer MUST 在写 stdout 前完成内存渲染。config 校验、pointer 查找、block 替换或 JSON header 序列化失败时，CLI MUST NOT 写入部分 readable-view stdout，MUST 在 stderr 输出稳定诊断，MUST 使用 `readable_view_render_failed` 作为错误 id，并 MUST 返回内部错误 exit code。该失败路径 MUST NOT fallback 到 readable-json、protocol-json、plain text、旧 text formatter 或递归 readable-view 错误渲染。

#### Scenario: config 失败不产生部分 stdout
- **WHEN** readable-view renderer 因 config pointer 指向非字符串字段而失败
- **THEN** stdout 为空
- **THEN** stderr 包含 `readable_view_render_failed`
- **THEN** 命令使用内部错误 exit code 退出
- **THEN** stdout 不包含 readable-json、protocol-json、plain text 或旧 text 输出

#### Scenario: stdout I/O 错误按既有 I/O 失败处理
- **WHEN** readable-view 已完成内存渲染但写 stdout 失败
- **THEN** CLI 按项目既有 I/O 错误路径退出
- **THEN** 实现不要求构造一个完整 readable-view 错误 payload 来表示该 I/O 失败

### Requirement: text 输出模式必须从当前 document 契约中删除
Document operation 的当前输出模式 MUST 只包含 `readable-view`、`readable-json` 和 `protocol-json`。实现、配置、help、当前主规范、README、skills、examples 和测试 MUST NOT 把 document `text` 作为受支持模式、alias、fallback 或 deprecated compatibility value。Help、version 和其它非文档纯文本输出 MAY 使用语义明确的 `PlainText` 通道。

#### Scenario: CLI text 值被拒绝
- **WHEN** 调用方执行 document operation 并传入 `--output text`
- **THEN** CLI 返回输入错误
- **THEN** help 指向 `readable-view`、`readable-json` 和 `protocol-json`
- **THEN** CLI 不执行 document operation 后再回退输出

#### Scenario: 普通执行拒绝配置 text 值
- **WHEN** 项目或用户配置包含 `defaults.output: "text"`
- **AND** 调用方执行 document operation
- **THEN** 配置校验返回明确错误
- **THEN** 错误包含配置路径、字段路径、legacy value 和修复命令
- **THEN** 实现不把该值静默映射为 readable-view

#### Scenario: config 命令可以修复 legacy text 值
- **WHEN** 目标项目或用户配置包含 `defaults.output: "text"`
- **AND** 调用方执行 `docnav config set defaults.output readable-view` 或 `docnav config unset defaults.output` 修复同一目标 scope
- **THEN** config 命令成功更新目标配置
- **THEN** 命令不因正在修复的 legacy output value 在加载阶段失败
- **THEN** 其它配置字段仍按正常 schema 校验

#### Scenario: text 实现符号不存在
- **WHEN** change 完成交付审计
- **THEN** 核心和 adapter SDK 不再包含 document text output enum variant、text formatter trait、text-specific writer 或 text template 配置
- **THEN** 当前非归档代码和文档不再要求 document operation `--output text`
- **THEN** 非文档纯文本输出使用 `PlainText` 或等价明确命名，不复用 document output mode 名称
