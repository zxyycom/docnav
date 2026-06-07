## MODIFIED Requirements

### Requirement: Markdown adapter 必须有负向 CLI 矩阵测试
`docnav-markdown` MUST 提供由 Node.js runner 执行的黑盒 CLI 矩阵测试，覆盖非法命令行输入、兼容性 warning 输入和非法 invoke 输入。矩阵 MUST 覆盖缺 path、缺 `--ref`、缺 `--query`、unknown flag、多余 positional、当前 operation 不使用的已知 flag、`page` 或 `limit_chars` 为 0、`page` 或 `limit_chars` 非数字、`max_heading_level` 越界、missing file、invalid ref、non-UTF-8 document、malformed invoke JSON。每个用例 MUST 按所属输出层断言 stdout、stderr 和 process exit code。

#### Scenario: 参数校验失败保持 CLI 诊断
- **WHEN** 负向矩阵执行缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit 或非法 max heading level
- **THEN** 进程非零退出
- **THEN** stderr 包含简洁诊断
- **THEN** stdout 不包含 protocol payload 或 readable result payload

#### Scenario: 兼容参数 warning 后继续
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的已知 flag
- **THEN** warning 包含具体 `ignored_tokens`、kind 和 reason
- **THEN** 若其它参数有效，进程成功退出
- **THEN** stdout 包含所选输出模式的正常结果
- **THEN** text 输出在正常结果后拼接 warning 文本
- **THEN** readable-json 输出包含 `warnings` 数组
- **THEN** protocol-json、manifest 和 probe stdout 不包含 `warnings` 字段，warning 写入 stderr

#### Scenario: unknown flag 的后续普通 token 有明确归属
- **WHEN** CLI 矩阵执行 `docnav-markdown outline <path> --future extra`
- **THEN** `--future` 被归属为 unknown flag warning 的 ignored token
- **THEN** `extra` 继续按普通 token 处理
- **THEN** 因 outline 已接收 `<path>` positional，`extra` 被归属为多余 positional warning 的独立 ignored token

#### Scenario: unknown flag 不吞后续已知 flag
- **WHEN** CLI 矩阵执行 `docnav-markdown outline <path> --future --output protocol-json`
- **THEN** `--future` 被 warning 后忽略
- **THEN** `--output protocol-json` 仍生效
- **THEN** stdout 是通过 protocol response schema 的 protocol JSON 且不包含 `warnings`
- **THEN** stderr 包含 `--future` warning

#### Scenario: 已知 flag 的值紧跟解析
- **WHEN** 负向矩阵执行 `docnav-markdown read <path> --ref --future-value`
- **THEN** `--future-value` 作为 `--ref` 的值传入
- **THEN** 命令按该 ref 执行业务逻辑并返回对应 operation 结果或稳定 ref 错误
- **THEN** stderr 不包含缺少 `--ref` 值的诊断

#### Scenario: readable operation 错误保留 code 和 details
- **WHEN** 负向矩阵以 `--output readable-json` 执行 missing file、invalid ref 或 non-UTF-8 document 用例
- **THEN** stdout 包含 readable error JSON，并保留稳定 `code`、`error`、`details` 和 `guidance`
- **THEN** stdout 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`
- **THEN** stderr 不包含替代 readable payload

#### Scenario: protocol-json operation 错误保留 envelope
- **WHEN** 负向矩阵以 `--output protocol-json` 执行 invalid ref 或 non-UTF-8 document 用例
- **THEN** stdout 包含 failure protocol envelope
- **THEN** envelope 保留 request operation 和稳定 error details
- **THEN** stderr 只包含诊断，且不重复 protocol JSON

#### Scenario: malformed invoke JSON 返回结构化协议失败
- **WHEN** 负向矩阵向 `docnav-markdown invoke` 写入 malformed JSON
- **THEN** stdout 包含 `operation: null` 且 error code 为 `INVALID_REQUEST` 的 protocol failure envelope
- **THEN** 进程非零退出

#### Scenario: invoke 参数 schema 错误返回结构化协议失败
- **WHEN** 负向矩阵向 `docnav-markdown invoke` 写入 JSON 语法合法但缺少必需字段或参数类型错误的请求
- **THEN** stdout 包含 `INVALID_REQUEST` protocol failure envelope
- **THEN** failure envelope 的 operation 在可解析时保留对应 operation，否则为 null
- **THEN** 进程非零退出
