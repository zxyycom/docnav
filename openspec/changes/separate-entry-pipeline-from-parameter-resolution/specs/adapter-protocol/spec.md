本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是为 adapter SDK 和 invoke protocol 定义标准入口管线与不可变原始输入边界；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter SDK standard entry pipeline preserves direct command boundaries
`docnav-adapter-sdk` MUST classify direct CLI invocations into document operations, manifest, probe, invoke, and help before document operation configuration loading or entry parameter source resolution. Manifest, probe, and help MUST keep their existing schema or plain-text boundaries and MUST NOT enter document output mode.

#### Scenario: Manifest does not read document operation config
- **WHEN** a caller executes `docnav-markdown manifest --output protocol-json`
- **THEN** SDK classifies the invocation as manifest
- **THEN** SDK does not read adapter document operation project or user config
- **THEN** SDK does not invoke entry parameter source resolution for document operation parameters
- **THEN** stdout contains only the manifest schema payload

#### Scenario: Probe does not enter document output mode
- **WHEN** a caller executes `docnav-markdown probe docs/guide.md --output protocol-json`
- **THEN** SDK classifies the invocation as probe
- **THEN** SDK emits the probe schema payload
- **THEN** SDK does not render readable-view or readable-json document output

#### Scenario: Direct document operation invokes parameter source resolution
- **WHEN** a caller executes `docnav-markdown outline docs/guide.md --limit 120`
- **THEN** SDK classifies the invocation as a document operation
- **THEN** SDK maps argv into a direct input view, config locators, and explicit adapter native option source descriptors
- **THEN** SDK invokes entry parameter source resolution with direct input, config sources, owner-declared native option sources, and defaults
- **THEN** request construction consumes derived typed runtime values

### Requirement: Adapter invoke keeps stdin request immutable
Adapter `invoke` MUST treat decoded stdin JSON as immutable raw protocol input. Entry parameter source resolution MAY derive safe operation values from request envelope facts, request `arguments`, registered config sources, and defaults, but MUST NOT write config/default values back into the raw stdin JSON or reclassify them as direct input.

#### Scenario: Invoke config fills derived operation value without mutating request
- **WHEN** adapter `invoke` receives an outline request whose `arguments` omit `limit`
- **AND** project adapter config supplies `defaults.pagination.limit`
- **THEN** SDK may derive the operation limit from project config
- **THEN** the raw decoded stdin request still omits `arguments.limit`
- **THEN** the derived limit source info is project config, not direct input

#### Scenario: Invoke direct input remains strict
- **WHEN** adapter `invoke` receives malformed JSON bytes
- **THEN** SDK reports a transport decode failure before entry parameter source resolution
- **WHEN** adapter `invoke` receives decoded JSON with an invalid registered argument type
- **THEN** entry parameter source resolution produces a validation diagnostic
- **THEN** SDK does not ignore the invalid field using direct CLI-specific rules
