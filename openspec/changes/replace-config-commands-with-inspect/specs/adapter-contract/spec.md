本 spec delta 定义 `adapter-contract` 的新增要求：adapter native option declaration 必须作为参数汇总事实源被消费，同时保持 adapter-id namespace 和 selected-adapter 边界。

本 spec delta 只拥有 adapter declaration 与 handler 边界：adapter 继续声明 native option facts，registry/navigation projection 将这些 facts 映射到 adapter-id config path，handler 仍只接收 typed input。CLI inspect 输出、source loading 和 selected-operation dispatch 细节不在本 delta 中定义。

## ADDED Requirements

### Requirement: Native option declarations provide config validation facts

Adapter native option declarations MUST provide the typed-field facts needed by the parameter aggregation layer to validate config source values for that adapter option, including owner identity, option key, operation applicability, value kind, constraints, defaults when declared, source processing metadata, and operation binding metadata. The registry/navigation aggregation layer combines those declaration facts with the existing adapter registry id to produce adapter-id namespaced persistent config processing paths.

#### Scenario: Adapter option is used for config validation

- **WHEN** a registered adapter declares native option key `max_heading_level` for an operation
- **THEN** parameter aggregation can combine that declaration with the adapter registry id and project the persistent config path to `options.<adapter-id>.max_heading_level`
- **THEN** config source validation can use that declaration to validate a config source value for that adapter-id path
- **THEN** the adapter handler receives only the typed option value after navigation resolution succeeds

### Requirement: Shared config projections keep adapter option keys distinct

Parameter aggregation MUST project adapter-owned persistent config paths with the existing adapter registry id segment and keep `options.<adapter-a>.<key>` distinct from `options.<adapter-b>.<key>`. Equal option keys from different adapters remain separate adapter-owned fields. This migration changes persistent config-source paths only; adapter handler handoff remains operation-specific typed input. When a single adapter exposes incompatible declarations for the same adapter-id config path, shared layers MUST report adapter-local declaration conflict through the consuming CLI/config boundary or require the adapter to expose distinct config keys.

#### Scenario: Same key in different adapters remains distinct

- **WHEN** two adapters declare option key `mode` with different value kind or constraints
- **AND** config inspection or source validation evaluates `options.docnav-markdown.mode`
- **THEN** parameter aggregation uses only the declaration for adapter id `docnav-markdown`
- **THEN** declarations from other adapter ids do not affect that validation

#### Scenario: Adapter-local conflict is rejected

- **WHEN** one adapter declares the same config path `options.docnav-markdown.mode` with incompatible metadata for different operations
- **THEN** parameter aggregation does not choose one declaration implicitly
- **THEN** the consuming config or navigation boundary reports an adapter-local declaration conflict before producing source validation results or dispatching the adapter

### Requirement: Adapter handlers remain downstream of typed validation

Adapter handlers MUST continue to receive operation-specific typed input after config source validation, selected adapter option extraction, and request construction. Adapter handlers MUST NOT be responsible for basic type/range/nullability validation of raw config values that can be expressed in their native option declarations.

#### Scenario: Invalid config option blocks dispatch

- **WHEN** a config source provides an invalid value for a selected adapter native option
- **THEN** navigation or the consuming input boundary reports the validation diagnostic before adapter dispatch
- **THEN** the adapter handler is not invoked with the invalid raw config value
