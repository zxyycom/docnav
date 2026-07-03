本 delta spec 记录跨组件共享 text cost calculator helper 的长期边界。该 helper 面向任意已经拥有纯文本的 Docnav 组件，不限定为 adapter 使用。

## ADDED Requirements

### Requirement: Shared text cost calculator exposes simple text-to-cost functions
Docnav shared helpers MUST provide text cost calculator functions with a uniform interface: each function accepts plain text input and returns a protocol-compatible cost measurement for the function-defined cost type. The helper MUST NOT require adapter identity, format identity, path, ref, operation, parser state, output mode, unit parameters, scope, tokenizer policy, encoding, model preset, or strategy objects to calculate cost.

#### Scenario: Any component can calculate cost from selected text
- **WHEN** a Docnav component has already selected or produced plain text
- **AND** the component chooses which text cost helper function to call
- **THEN** it can call that function with the text as the only required input
- **THEN** the helper returns a deterministic cost measurement with the function-defined `unit` and calculated `value`
- **THEN** the caller remains responsible for whether and where that measurement is exposed in protocol, readable output, tests, diagnostics, or internal tooling

#### Scenario: First helper functions and units are fixed
- **WHEN** a caller uses the initial shared text cost calculator API
- **THEN** `line_cost(text: &str) -> Measurement` returns a measurement with unit `lines`
- **THEN** `byte_cost(text: &str) -> Measurement` returns a measurement with unit `bytes`
- **THEN** `token_cost(text: &str) -> Measurement` returns a measurement with unit `tokens`
- **THEN** each function accepts the selected plain text as its only required input
- **THEN** the helper does not attach scope to the returned measurement

#### Scenario: Adapter chooses functions and calls helper directly
- **WHEN** an adapter needs cost measurements for output it owns
- **THEN** the adapter chooses which helper functions to call and calls those functions directly
- **THEN** core, output formatting, and the helper do not choose adapter cost measurements on the adapter's behalf

#### Scenario: Token cost uses the same text-only interface
- **WHEN** a caller needs token cost for already selected plain text
- **THEN** it calls the token cost helper function with that text as the only required input
- **THEN** the token helper uses `tiktoken-rs` `o200k_base` ordinary plain-text tokenization to return a deterministic cost measurement with a token unit and calculated token value
- **THEN** text that looks like a special token is counted as plain output text rather than as a tokenizer control token

#### Scenario: Helper does not own text selection
- **WHEN** an adapter, output tool, or validation script needs cost for a document fragment
- **THEN** that caller chooses or produces the fragment text before invoking the helper
- **THEN** the helper does not parse the source document, resolve refs, apply native options, truncate content, or decide pagination budget semantics
