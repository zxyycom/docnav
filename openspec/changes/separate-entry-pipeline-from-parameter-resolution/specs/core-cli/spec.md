本 delta spec 是 `separate-entry-pipeline-from-parameter-resolution` 的未审核临时文档，目标是为 core CLI 定义标准入口管线与参数来源解析边界；当前 change 只在 `openspec/changes/separate-entry-pipeline-from-parameter-resolution/` 下形成临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Core CLI standard entry pipeline preserves command-family boundaries
`docnav` core CLI MUST classify each invocation into an entry family before loading document configuration or resolving document operation parameters. The standard entry pipeline MUST distinguish document operations, config/init/doctor/version, help, and adapter management. Only document operations and explicitly documented document-context inspection paths MAY invoke entry parameter source resolution. Help and non-document commands MUST NOT be treated as document semantic requests.

#### Scenario: Help exits before document parameter source resolution
- **WHEN** a caller executes `docnav --help`
- **OR** a caller executes `docnav outline --help`
- **THEN** core classifies the invocation as help
- **THEN** core does not load document operation configuration
- **THEN** core does not invoke entry parameter source resolution
- **THEN** core does not select an adapter or construct an invoke request

#### Scenario: Non-document command keeps its owner boundary
- **WHEN** a caller executes `docnav version`
- **OR** a caller executes `docnav init`
- **OR** a caller executes `docnav doctor`
- **THEN** core classifies the invocation as a non-document command
- **THEN** the command uses its own output and error owner
- **THEN** the command does not enter document semantic request construction

#### Scenario: Document command invokes parameter source resolution after classification
- **WHEN** a caller executes `docnav outline docs/guide.md --limit 120`
- **THEN** core classifies the invocation as a document operation
- **THEN** core maps argv into a direct input view
- **THEN** core invokes entry parameter source resolution with direct input, configured sources, and defaults
- **THEN** adapter routing and request construction consume derived typed runtime values rather than raw argv tokens

### Requirement: Core CLI keeps raw argv immutable during parameter resolution
Core CLI parameter source resolution MUST NOT delete, rewrite, reorder, or supplement raw argv tokens. Ignored argv diagnostics, output intent preflight, and document request construction MUST use derived facts or typed runtime values while preserving the original argv as the raw invocation record.

#### Scenario: Config defaults do not mutate argv
- **WHEN** a caller executes `docnav outline docs/guide.md` without `--output`
- **AND** project config supplies `defaults.output`
- **THEN** parameter source resolution may derive an output mode from project config
- **THEN** the raw argv remains unchanged and still lacks `--output`
- **THEN** request construction consumes the derived output value without classifying it as direct input

#### Scenario: Ignored argv diagnostics use raw tokens without resolver mutation
- **WHEN** a caller executes `docnav info docs/guide.md --page nope --output readable-json`
- **THEN** core may report `--page nope` as ignored argv for the selected operation
- **THEN** parameter source resolution does not coerce that raw token into a page value
- **THEN** the info request remains free of page and limit arguments
