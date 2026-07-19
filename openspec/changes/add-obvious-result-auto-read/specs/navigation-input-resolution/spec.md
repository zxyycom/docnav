本 spec delta 定义 auto-read mode 的 core catalog projection、当前返回 ref 判定和 existing read dispatch。它不向 adapter request 或 closed strategy input 增加 composition field。

## ADDED Requirements

### Requirement: auto-read mode has one canonical CLI and config declaration

The core-authored document parameter catalog MUST declare `docnav.defaults.auto_read` as a `Replace` string enum with CLI locator `--auto-read`, config locator `defaults.auto_read`, built-in default `unique-ref`, no environment locator, and operation bindings limited to `outline` and `find`. Resolution MUST project the selected mode to core/navigation orchestration and MUST NOT serialize it into protocol `OperationArguments`, adapter `Options` or `StandardOperationInput`.

#### Scenario: all omitted sources resolve the built-in mode
- **WHEN** an outline or find invocation has no CLI, project or user candidate for auto-read
- **THEN** canonical resolution materializes `unique-ref` from the built-in default
- **AND** the base adapter request retains its existing operation-specific shape

#### Scenario: explicit CLI overrides config
- **WHEN** CLI, project config and user config all provide valid auto-read values
- **THEN** canonical resolution selects the CLI value
- **AND** provenance records the project and user candidates as overridden

#### Scenario: project config overrides user config
- **WHEN** CLI omits auto-read
- **AND** project and user config both contain valid `defaults.auto_read` values
- **THEN** canonical resolution selects the project value
- **AND** provenance records the user candidate as overridden

#### Scenario: user config overrides the built-in default
- **WHEN** CLI and project config omit auto-read
- **AND** user config contains `defaults.auto_read: "disabled"`
- **THEN** canonical resolution selects `disabled`
- **AND** navigation dispatches only the requested base operation

#### Scenario: valid config is operation scoped
- **WHEN** a loaded config contains a valid `defaults.auto_read` value
- **AND** the requested operation is `read` or `info`
- **THEN** full config validation recognizes the field
- **AND** selected-operation resolution does not project auto-read or alter that operation

#### Scenario: invalid config value is source attributed
- **WHEN** project or user config contains an auto-read value other than `disabled` or `unique-ref`
- **THEN** config validation returns the existing source-attributed invalid enum diagnostic
- **AND** no adapter operation is dispatched

#### Scenario: undeclared environment input has no effect
- **WHEN** the process environment contains a similarly named auto-read variable
- **THEN** the env extractor emits no auto-read candidate
- **AND** resolution continues with CLI, project, user and built-in sources

### Requirement: unique-ref eligibility uses refs in the current returned result

After a successful validated base response, navigation MUST evaluate the refs returned by the current structured outline or find result. It MUST invoke read exactly once when string-exact deduplication produces one non-empty opaque ref, and MUST otherwise return the validated base response unchanged.

#### Scenario: one returned ref invokes read
- **WHEN** a structured outline or find succeeds
- **AND** the current result contains exactly one distinct non-empty ref string
- **THEN** navigation invokes read exactly once with that ref

#### Scenario: repeated find matches share one read target
- **WHEN** a find result contains multiple matches
- **AND** every returned match carries the same non-empty ref string
- **THEN** navigation treats the returned ref as unique
- **AND** invokes read exactly once

#### Scenario: page state does not alter current-result uniqueness
- **WHEN** the current structured result contains exactly one distinct non-empty ref
- **AND** the request page is greater than `1` or the response page is non-null
- **THEN** navigation invokes read exactly once
- **AND** does not interpret that ref as globally unique across other pages

#### Scenario: no returned ref keeps the base response
- **WHEN** the current structured result contains no items
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: multiple returned refs keep the base response
- **WHEN** the current structured result contains more than one distinct ref string
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

#### Scenario: unstructured outline keeps its content response
- **WHEN** outline returns the unstructured content branch
- **THEN** navigation does not invoke read
- **AND** returns the validated base response unchanged

### Requirement: nested read reuses the selected document context

For an eligible unique ref, navigation MUST invoke the existing selected adapter read strategy without recursively invoking the CLI, selecting another adapter or executing an intermediate output plan. The nested read MUST use the same normalized document path, pass the ref unchanged and start at read page `1`; its remaining input MUST follow the existing read contract.

#### Scenario: ref remains opaque across composition
- **WHEN** navigation constructs the nested read input
- **THEN** it passes the candidate ref unchanged
- **AND** only the selected adapter read strategy parses the ref

#### Scenario: existing read inputs remain authoritative
- **WHEN** unique-ref orchestration invokes read
- **THEN** the nested read starts at page `1`
- **AND** uses the already resolved common input that applies to the existing read strategy
- **AND** any nested `ReadResult.page` retains its existing continuation meaning

#### Scenario: validated read success is composed
- **WHEN** the selected adapter read returns a validated success
- **THEN** navigation constructs `auto_read` with reason `unique_ref` and the complete existing `ReadResult`
- **AND** validates the composed response before returning it to output orchestration

#### Scenario: non-successful read keeps the base response
- **WHEN** the selected adapter read does not produce a validated success
- **THEN** navigation returns the validated base response unchanged
- **AND** does not add an auto-read status, reason or error object

#### Scenario: invalid composition keeps the base response
- **WHEN** nested read succeeds
- **AND** the candidate composed response does not pass protocol validation
- **THEN** navigation discards the candidate composition
- **AND** returns the already validated base response unchanged
