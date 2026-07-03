本 delta spec 记录 text cost calculator helper 与 current `cost.measurements[]` protocol shape 的映射边界。该边界不重新定义 raw protocol 或 readable output 字段。

## ADDED Requirements

### Requirement: Text cost calculator outputs protocol-compatible measurements
Shared text cost calculator helpers MUST return cost measurements that can be represented through the current `cost.measurements[]` protocol shape without adding format-specific protocol fields or readable-only fields. For each helper function call, the returned measurement MUST include the function-defined `unit` and a helper-computed non-negative integer `value`.

#### Scenario: Plain text cost maps to protocol cost
- **WHEN** a Docnav component already has selected plain text and calls a shared text cost helper function
- **THEN** the helper returns a measurement with protocol-compatible `unit` and non-negative integer `value`
- **THEN** callers can embed one or more such measurements in `cost.measurements[]` without changing response envelope or operation result shape
- **THEN** readable cost summaries remain derived by the output layer from protocol measurements

#### Scenario: Scope remains caller-owned protocol context
- **WHEN** a shared text cost helper function returns a measurement
- **THEN** the helper result has no helper-selected scope
- **WHEN** a caller embeds that measurement in a protocol result that has scoped cost semantics
- **THEN** the caller attaches the operation-appropriate scope without changing helper input or helper function selection
