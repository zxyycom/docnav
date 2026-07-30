本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时契约工件：它删除 JSON-owned selection probe 和 post-probe 特例，同时保留 selected JSON operation 对真实文档的完整 parse 与 validation。

## ADDED Requirements

### Requirement: JSON selected operations 必须验证实际文档

When navigation selects `docnav-json` through an inferred normalized `json` identity or explicit adapter intent, the requested JSON strategy MUST acquire and parse the actual document according to the JSON-owned UTF-8, optional leading BOM, complete-value, trailing-whitespace, maximum-depth, and unique-decoded-member rules before using its private model. Selection success MUST NOT substitute for this parse. A selected JSON document or operation failure MUST return the existing owner-compatible document or JSON adapter diagnostic and MUST NOT trigger format inference again or dispatch another adapter. A document change between inference and selected operation MUST be reported according to the document state observed by the operation; it MUST NOT use the removed probe-stage error id `json-document-changed-after-probe`.

#### Scenario: Automatically selected JSON operation parses current content

- **WHEN** one inference invocation normalizes a document to `json`
- **AND** registry matching selects `docnav-json`
- **THEN** the requested JSON strategy parses the actual document before producing operation facts
- **THEN** JSON ref, depth, duplicate-member, number, and source-region semantics remain JSON-owned

#### Scenario: Explicit JSON selection does not waive parse

- **WHEN** caller explicitly selects `docnav-json`
- **AND** the actual document violates JSON parse or safety rules
- **THEN** navigation skips inference
- **THEN** the selected JSON strategy returns its normal owner-compatible diagnostic
- **THEN** no other adapter is attempted

#### Scenario: Document changes after inference

- **WHEN** navigation infers `json` and selects `docnav-json`
- **AND** the path content changes before the selected operation reads it
- **THEN** the operation validates the document view it actually opens
- **THEN** a read or JSON validation failure uses the normal document or JSON adapter diagnostic
- **THEN** the removed `json-document-changed-after-probe` stage id is not emitted

## MODIFIED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json`、format id `json` 和 content type `application/json` 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于既有 closed standard operation input；core parameter catalog、`StandardInputBinding`、CLI、env、config 和 protocol input inventory MUST 保持注册前的契约。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明 JSON format、`.json` extension 和 `application/json`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** core public input inventory 与注册前相同

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界

JSON adapter 主文档、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖确定性 outline、empty-container 空 entries、root scalar entry、ASCII-safe ref grammar、空/特殊/control key roundtrip、context-sensitive array index、无损 JSON number、原文 occurrence 与 source-region-to-ref 映射、outline/find-to-read roundtrip、parse/ref errors、Unicode pagination、info/full-read、generic `readable-view`、单次 inferred routing、显式 adapter selection、selected parse failure 不 fallback，以及同一个 release binary 中的 linked JSON behavior。

#### Scenario: 验证 JSON 产品与集成语义

- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned parse、navigation、ref、content 和 error behavior
- **THEN** core/release 验证覆盖 inferred/explicit static selection、closed input、no-fallback dispatch 和单一 binary linked behavior
- **THEN** 测试从 package core executable 运行，并证明 public input inventory 与注册前相同

## REMOVED Requirements

### Requirement: JSON probe 必须同时验证格式提示和文档内容

**Reason**: Format recognition moves to one navigation-private inference invocation; JSON selection no longer executes a JSON-owned probe or establishes a probe/reload stage boundary.

**Migration**: The approved inference result maps privately to normalized format id `json`; registry exact-match selects `docnav-json`; selected JSON operations retain complete JSON-owned parse and safety validation. Replace `json-document-changed-after-probe` evidence with normal selected-operation document/JSON diagnostics, then delete JSON probe code/tests and shared probe schema/fixtures after the blocking compatibility gate. If a real owner-backed consumer is found, current apply stops and returns to artifacts/human approval; it does not retain an inspection surface.

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
