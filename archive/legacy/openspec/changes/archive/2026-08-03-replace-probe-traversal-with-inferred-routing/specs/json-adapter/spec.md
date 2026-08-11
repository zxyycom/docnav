本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `json-adapter` 尚未应用的 Target：删除 JSON-owned selection probe 和 post-probe 特例，同时保留 selected JSON operation 对真实文档的完整 parse 与 validation；它不表示 Current 主规范或实现已经迁移。

## ADDED Requirements

### Requirement: JSON selected operations 必须验证实际文档

When navigation selects `docnav-json` through manifest pathname routing or explicit adapter intent, the requested JSON strategy MUST acquire and parse the actual document according to the JSON-owned UTF-8, optional leading BOM, complete-value, trailing-whitespace, maximum-depth, and unique-decoded-member rules before using its private model. A pathname hint is only an adapter-selection hint; selection success MUST NOT assert that content is valid JSON or substitute for this parse. A selected JSON document or operation failure MUST return the owner-compatible document or JSON adapter diagnostic and MUST NOT trigger pathname routing again or dispatch another adapter. A document change between selection and the operation read MUST be reported according to the document state observed by the operation; it MUST NOT use the removed probe-stage error id `json-document-changed-after-probe`.

Invalid UTF-8 MUST continue to use `DOCUMENT_ENCODING_UNSUPPORTED`. Invalid JSON syntax, trailing non-whitespace input, duplicate decoded member names, and maximum-depth overflow MUST use `DOCUMENT_CONTENT_INVALID` with exact reasons `JSON_SYNTAX_INVALID`, `JSON_TRAILING_INPUT`, `JSON_DUPLICATE_MEMBER`, and `JSON_MAXIMUM_DEPTH_EXCEEDED`, respectively. Canonical details MUST contain only the normalized `path` and stable `reason`; parser-library types/messages, unstable offsets, duplicate names, and dependency traces MUST remain private.

Until a separate JSONC grammar change is applied, the selected adapter's grammar remains the current strict JSON grammar. A `.code-workspace` file containing JSONC-only syntax or a `.prettierrc` containing YAML therefore MAY match a JSON pathname hint and then fail with the normal JSON-owned parse diagnostic. That failure is an expected consequence of hint-based selection, not evidence that routing should retry another adapter.

#### Scenario: Automatically selected JSON operation parses current content

- **WHEN** one manifest pathname lookup maps a document to format `json`
- **AND** registry lookup selects `docnav-json`
- **THEN** the requested JSON strategy parses the actual document before producing operation facts
- **THEN** JSON ref, depth, duplicate-member, number, and source-region semantics remain JSON-owned

#### Scenario: Explicit JSON selection does not waive parse

- **WHEN** caller explicitly selects `docnav-json`
- **AND** the actual document violates JSON parse or safety rules
- **THEN** navigation skips automatic pathname routing
- **THEN** the selected JSON strategy returns its owner-compatible document or `DOCUMENT_CONTENT_INVALID` diagnostic
- **THEN** no other adapter is attempted

#### Scenario: Pathname alias does not prove JSON validity

- **WHEN** `.prettierrc` or `.code-workspace` matches a JSON manifest pathname hint
- **AND** its content is outside the selected JSON adapter's then-current grammar
- **THEN** the selected operation returns its normal JSON-owned parse diagnostic
- **THEN** navigation does not retry format routing or another adapter

#### Scenario: Document changes after pathname selection

- **WHEN** navigation routes a pathname to `docnav-json`
- **AND** the path content changes before the selected operation reads it
- **THEN** the operation validates the document view it actually opens
- **THEN** a read or JSON validation failure uses the applicable document or `DOCUMENT_CONTENT_INVALID` diagnostic
- **THEN** the removed `json-document-changed-after-probe` stage id is not emitted

## MODIFIED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json`、format id `json` 和 content type `application/json` 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于既有 closed standard operation input；core parameter catalog、`StandardInputBinding`、CLI、env、config 和 protocol input inventory MUST 保持注册前的契约。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明 JSON format、`.json` 与 `.code-workspace` `extensions[]` basename suffixes、`.prettierrc` 与 `.watchmanconfig` exact filenames，以及 `application/json`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** matched filename、suffix 或 format identity 不进入 strategy input
- **THEN** core public input inventory 与注册前相同

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界

JSON adapter 主文档、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖确定性 outline、empty-container 空 entries、root scalar entry、ASCII-safe ref grammar、空/特殊/control key roundtrip、context-sensitive array index、无损 JSON number、原文 occurrence 与 source-region-to-ref 映射、outline/find-to-read roundtrip、exact `DOCUMENT_CONTENT_INVALID`/encoding/ref errors、Unicode pagination、info/full-read、generic `readable-view`、manifest pathname routing、显式 adapter selection、selected parse failure 不 fallback，以及同一个 release binary 中的 linked JSON behavior。

#### Scenario: 验证 JSON 产品与集成语义

- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned parse、navigation、ref、content 和 error behavior
- **THEN** core/release 验证覆盖 pathname/explicit static selection、closed input、no-fallback dispatch 和单一 binary linked behavior
- **THEN** 测试从 package core executable 运行，并证明 public input inventory 与注册前相同

## REMOVED Requirements

### Requirement: JSON probe 必须同时验证格式提示和文档内容

**Reason**: Automatic selection moves to one navigation-private, manifest-derived pathname lookup; JSON selection no longer executes a JSON-owned probe or establishes a probe/reload stage boundary.

**Migration**: Manifest hints map `.json` and `.code-workspace` basename suffixes plus exact `.prettierrc` and `.watchmanconfig` basenames privately to format id `json`; registry exact-match selects `docnav-json`; selected JSON operations retain complete JSON-owned parse and safety validation without receiving the matched routing format. Replace `json-document-changed-after-probe` with `DOCUMENT_CONTENT_INVALID` and the four exact JSON reasons, preserve existing path/encoding diagnostics, then delete JSON probe code/tests and shared probe schema/fixtures after the blocking removal inventory is complete. Every discovered consumer is deleted, migrated, or recorded as an explicit breaking impact; no compatibility or inspection surface is retained. JSONC grammar support remains owned by the separate `support-jsonc-in-json-adapter` change.

JSON probe MUST 先匹配大小写不敏感的 `.json` extension。Extension mismatch MUST 返回单个 content-conflict reason，并在文件读取前结束 probe。

Extension match 后，probe MUST 去除一个可选开头 UTF-8 BOM，执行 UTF-8 decode 和完整 JSON parse。Root depth MUST 为 `0`，最大支持 `max_depth` MUST 为 `127`。文档包含一个完整 JSON value、其后仅有 whitespace、`max_depth <= 127`，且每个 object 的 decoded member name 唯一时，probe MUST 返回 `supported: true`、format `json`、confidence `1.0`，并依次给出 extension-match 和 content-match reason。

非 UTF-8、parse failure、trailing non-whitespace、`max_depth > 127` 或重复 decoded member MUST 返回 `supported: false`、confidence `0.0` 和对应 read/content-conflict reason，selection MUST 在 strategy dispatch 前结束。若同一路径在成功 probe 后、operation reload 前变为上述任一无效状态，adapter MUST 返回 `INTERNAL_ERROR` 和 error id `json-document-changed-after-probe`；文件消失或编码变化继续使用既有 document diagnostics。

#### Scenario: 自动选择有效 JSON

- **WHEN** 未声明 adapter 且文档名为 `settings.JSON`
- **AND** 文档是 UTF-8、包含一个完整 JSON value，且每个 object 的 decoded member name 唯一
- **THEN** automatic discovery 选择 `docnav-json`
- **THEN** probe 返回 `supported: true` 和 format `json`

#### Scenario: 有歧义的 JSON 输入标记为 unsupported

- **WHEN** `.json` 文档包含 parse failure、trailing non-whitespace、`max_depth > 127` 或同一 object 中的重复 member name
- **THEN** JSON probe 返回 `supported: false`
- **THEN** selection 在 JSON operation strategy dispatch 前结束

#### Scenario: 选择后文档发生竞争修改

- **WHEN** JSON probe 成功后，同一路径在 selected operation reload 前变为 syntactically invalid、超过 depth 上限或出现重复 member
- **THEN** operation 返回 `INTERNAL_ERROR`
- **THEN** error id 为 `json-document-changed-after-probe`
