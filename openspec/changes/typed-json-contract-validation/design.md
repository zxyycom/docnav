本 design 只记录 typed JSON contract validation 的高层设计取向；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Context

用户明确希望 manifest 等 JSON 校验复用标准参数方案背后的 typed path/value 原理。更准确的边界是：复用 typed-field engine，不复用标准参数的 CLI/config/source merge 语义。

## Goals / Non-Goals

**Goals:**

- 用 typed field metadata 支撑 manifest、probe、protocol request/response 的 runtime typed decoder。
- 保留 JSON Schema 作为 public contract material、example validation 和 CI drift check。
- 建立 schema keyword 到 typed field 或 semantic validation 的 parity audit。
- 为未来移除 runtime generic schema validator 提供受控路径。

**Non-Goals:**

- 不把 manifest/probe/protocol response 变成标准参数。
- 不生成完整 JSON Schema 文件。
- 不改变当前稳定 error category、field path、stdout/stderr placement 或 protocol envelope。
- 不在审计前移除 `jsonschema` runtime dependency。

## Decisions

1. 非标准参数 JSON 复用 typed-field engine，不复用 standard parameter resolver。
   - Rationale: 字段 path/value 校验是同一原理；来源合并、默认值和 passthrough 不是同一语义。

2. JSON Schema 保留为契约材料。
   - Rationale: schema 对 examples、fixtures、CI 和第三方实现仍有价值。

3. runtime dependency removal 只能在 parity audit 后执行。
   - Rationale: 必须证明 unknown fields、required fields、types、version constants、operation/result pairs 和 manifest/probe payloads 没有退化。

## Risks / Trade-offs

- [Risk] typed decoder 与 schema 文件漂移 → Mitigation: parity tests 或 schema metadata consistency check 是实现前置。
- [Risk] error mapping 改变 → Mitigation: 审计要求先列出现有 stable error mapping 并保留。
- [Risk] 把标准参数语义套到 manifest/probe → Mitigation: spec 明确禁止继承 CLI/config/source merge。

## Open Questions

- 首轮覆盖范围是 manifest + probe，还是同时覆盖 protocol request/response。
- runtime `jsonschema` 是按 surface 分阶段移除，还是等所有 parity tests 完成后一次移除。
