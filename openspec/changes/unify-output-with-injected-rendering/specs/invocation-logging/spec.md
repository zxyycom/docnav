## MODIFIED Requirements

### Requirement: 调用日志不得污染 stdout 或协议输出

Invocation logging MUST write events only to its resolved log sink. Document stdout、protocol request/response、operation results、renderer input/output and linked adapter handler payloads MUST remain unchanged by logging.

#### Scenario: protocol-json stdout 保持纯净

- **WHEN** invocation logging is enabled for a `protocol-json` document operation
- **THEN** stdout contains only the protocol response or failure envelope
- **THEN** log events are written only to the configured sink

#### Scenario: Rendered stdout 保持纯净

- **WHEN** invocation logging is enabled for a rendered document operation
- **THEN** stdout contains only the complete text returned by the selected renderer
- **THEN** logging metadata is absent from renderer input and output

#### Scenario: Adapter handler payload 保持纯净

- **WHEN** core dispatches a linked adapter handler
- **THEN** the handler exchanges only its structured input、result or diagnostic payload
- **THEN** log events remain in the configured sink
