本 spec delta 定义 core CLI 复用共享 diagnostics、argv、JSON IO 和 output helper 的要求，同时保持 `docnav` 的 surface policy owner 身份。

## ADDED Requirements

### Requirement: Core CLI 必须复用共享 helper 且保留 core policy owner

`docnav` core CLI MUST 在共享 helper 存在后复用 diagnostics、direct CLI argv compatibility、JSON IO 和 document output orchestration helper。Core CLI MUST 继续拥有 adapter routing、configuration、project root handling、adapter process startup、registry command resolution、non-document command behavior 和 concrete core exit code enum。

#### Scenario: Core document argv compatibility 使用共享 scanner

- **WHEN** core document CLI 解析 unknown flags、extra positional values 或当前 operation 不使用的 known flags
- **THEN** 它使用共享 direct CLI argv compatibility scanner 做 token classification
- **THEN** 当前 operation 实际使用参数的 typed parsing 仍由 core 拥有
- **THEN** loose argv rule 不应用于 adapter `invoke` stdin JSON

#### Scenario: Core warning construction 使用共享 diagnostics

- **WHEN** core CLI 输出 ignored argv 或 adapter candidate warnings
- **THEN** warning item 使用 `docnav-diagnostics` 提供的稳定 warning envelope
- **THEN** CLI argv warning 保持 `id: "cli_argv_ignored"`
- **THEN** adapter candidate warning 保持 `id: "adapter_candidate_failure"`
- **THEN** protocol-json stdout 不包含 warning fields

#### Scenario: Core non-document JSON output 保持 core-owned

- **WHEN** core CLI 输出非 document operation 的 machine-readable JSON
- **THEN** 它可以复用 `docnav-json-io` 执行低层 JSON value serialization 和 newline writing
- **THEN** help、version、manifest、probe 或其它非 document output mode 不通过 `docnav-output` 编排
- **THEN** schema、plain text、stderr 和 exit behavior 仍由 core owning surface 决定

#### Scenario: Core document output 使用共享输出编排

- **WHEN** core CLI 得到 document operation success 或 stable error outcome
- **THEN** 它将 outcome、operation、request id、output mode 和 collected warnings 传给 `docnav-output` 的 document-only facade
- **THEN** `readable-json` 和 `readable-view` 从同一个 readable payload 派生
- **THEN** `protocol-json` 向 stdout 写出一个 protocol response envelope
- **THEN** diagnostics 和 protocol-json warning text 只写入 stderr

#### Scenario: Core exit code enum 仍由 core 拥有

- **WHEN** core CLI 将 `StableErrorCode` 映射为 process exit code
- **THEN** 它可以使用共享 classification helper
- **THEN** concrete core exit code enum 和最终 process exit decision 仍由 `docnav` core 拥有
