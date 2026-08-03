This temporary delta specifies the selected one-strategy JSONC behavior, bounded adjacent-family boundary, and remaining parser-evidence gate for the existing `json-adapter` capability; task 0 must approve one exact implementation before apply.

## MODIFIED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json` 和一个 normalized format id `json` descriptor 暴露 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。该 descriptor 的 `extensions[]` basename suffixes MUST 精确等于 `.json`、`.code-workspace` 和 `.jsonc`；`filenames[]` exact basename hints MUST 精确等于 `.prettierrc` 与 `.watchmanconfig`；`content_types[]` MUST 精确等于 `application/json` 与 `application/jsonc`；它 MUST NOT 声明其它 JSON-family pathname hint、adapter identity 或 format identity。JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于 shared closed standard operation input；注册 JSON MUST NOT 增加 core parameter、`StandardInputBinding`、CLI、env、config 或 protocol input。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明一个 JSON format、`.json`、`.code-workspace` 与 `.jsonc` `extensions[]` basename suffixes、`.prettierrc` 与 `.watchmanconfig` exact filenames，以及 `application/json` 与 `application/jsonc`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用 shared closed public input

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** matched filename、suffix、content type 或 format identity 不进入 strategy input
- **THEN** core public input inventory 不包含 JSON-specific field

#### Scenario: Manifest excludes adjacent JSON-family pathnames

- **WHEN** manifest metadata for `docnav-json` is inspected
- **THEN** the `json` descriptor contains exactly `.json`、`.code-workspace`、`.jsonc` suffixes and `.prettierrc`、`.watchmanconfig` exact filenames
- **THEN** strict-profile, `.code-snippets`, multi-document, and binary JSON-family pathnames are absent
- **THEN** the two declared content types do not select a grammar or dialect

### Requirement: JSON read 必须返回指定节点的规范化 JSON

JSON read MUST 解析 JSON-owned ref，并将指定 logical JSON value 序列化为 deterministic valid strict JSON：JSONC comments 和 accepted trailing commas MUST 不出现在 structured payload；object member MUST 保持源码顺序；container layout MUST 使用两空格缩进。每个 number MUST 使用原始 strict-JSON token；string escape、普通 scalar spelling 和尾随换行 MUST 使用 workspace-pinned parser/serializer 定义的 pinned result，而不能继承未审计的 dependency default。Read result MUST 保留输入 ref，使用 `application/json` content type，对分页前的完整 structured-read text 计算 cost，并按 Unicode-safe text pagination 返回 content 和下一页。Generic `readable-view` MUST 从这些 raw result facts 形成 read header 和 content block。

#### Scenario: 读取嵌套 object

- **WHEN** read 收到 outline 返回的 nested object ref
- **THEN** content 是该 object 的确定性 pretty-printed strict JSON
- **THEN** content type 为 `application/json`
- **THEN** cost 描述分页前的完整选中值

#### Scenario: Structured read normalizes JSONC

- **WHEN** selected JSON source contains approved comments or trailing commas
- **AND** read targets any logical value from that document
- **THEN** content preserves the same ordered logical value as deterministic valid strict JSON
- **THEN** comments and trailing commas are absent from content
- **THEN** content type remains `application/json` and the input `json:#` ref is preserved

#### Scenario: 分页读取 Unicode string

- **WHEN** 指定 JSON value 的规范化输出超过 limit 且包含多字节 Unicode 字符
- **THEN** read 在字符边界分页
- **THEN** page 指向下一页直至内容结束

### Requirement: JSON find 必须返回可继续读取的节点

JSON find MUST 接受长度至少为一的 query，并 MUST 拒绝长度为零的 query。它 MUST 对去除一个可选开头 UTF-8 BOM 后的原文执行大小写敏感、从左到右、非重叠的 literal search，包括 accepted comments、trailing commas 和原始 string/number spelling；canonical pointer 和 structured-read serialization 只作为其它 operation 的结果，不扩充 find 语料。

Adapter-private logical source regions MUST 让每个命中确定性归属到可读取 JSON value：root region 覆盖完整 BOM-stripped source；object child region 从 member name token 开始并覆盖其 value；array child region 覆盖其 value。JSONC implementation MUST 另外保留 bounded、original-offset recorded comment spans，且这些 spans MUST NOT 成为第二棵 full logical tree。一个 block-comment span MUST 从 opening `/*` 延伸到第一个 following `*/`；span 内另一个 `/*` MUST 保持同一 comment span 的 text，不能开始第二个 span。String token 中的 `//` 或 `/*` MUST NOT 进入 recorded comment spans。

完全位于单一 recorded JSONC comment span 内的命中 MUST 映射到包含它的最深 object/array，若没有 child container 则映射到 root；这条 comment-only rule MUST 覆盖与它重叠的 member/value region。其它所有命中 MUST 使用本 requirement 上文定义的 logical source-region ownership：选择完整覆盖 occurrence 的最深 logical source region，跨越多个 child region 时映射到同时覆盖它们的最近 container。其它命中包括 logical token、string 内 comment markers、普通 strict whitespace、accepted trailing comma、container punctuation、root value 外围 source，以及跨越 comment 边界、comment/token boundary、多个 comment spans 或其它多个 region 的 occurrence。

Find MUST 为每个 source occurrence 返回一个 match，并按 source offset 排序；多个 occurrence 映射到同一 ref 时 MUST 保留多个 match。每个 match MUST 带完整 ref、`kind: "match"`、从原文派生的非空 bounded excerpt label 和 source line location。Entry pagination MAY 截断 display facts，并在完整 ref 已耗尽预算时只保留最小非空 label；分页 MUST 保持 occurrence order 并持续前进。返回 ref 必须能原样传给 read。Generic `readable-view` MUST 使用这些 raw entry facts。

#### Scenario: Member name 命中映射到 member value

- **WHEN** 原文 object member name `a/b` 命中 query `a/b`
- **THEN** find 返回该 member value 的 canonical ref `json:#/a~1b`
- **THEN** read 该 ref 返回对应 JSON value

#### Scenario: 搜索保留原文 spelling

- **WHEN** string value 的原文 token 包含 `\u0061`
- **THEN** query `\u0061` 命中该 source occurrence
- **THEN** structured read 的 pinned serializer spelling 不改变 find 语料

#### Scenario: Comment occurrence overrides an overlapping member region

- **WHEN** a query occurrence is wholly inside a JSONC comment between an object member name and its scalar value
- **THEN** one recorded comment span classifies the complete occurrence even though a member region also covers that source range
- **THEN** find maps it to the deepest enclosing object rather than the scalar member
- **THEN** read of the returned ref yields normalized logical JSON rather than comment text

#### Scenario: Cross-comment-boundary occurrence uses logical source-region mapping

- **WHEN** one source occurrence is not wholly inside one recorded comment span because it crosses a comment/token, comment/whitespace, or multiple-comment-span boundary
- **THEN** the comment-only override does not apply
- **THEN** find uses the deepest logical source region that covers the occurrence's complete range
- **THEN** an occurrence crossing multiple child regions maps to their nearest containing object or array

#### Scenario: 同一节点的多个源码命中分别返回

- **WHEN** 同一 scalar 或 container region 中存在两个 non-overlapping query occurrence
- **THEN** find 按 source offset 返回两个 match
- **THEN** 两个 match MAY 携带相同 ref，且该 ref 均可原样传给 read

### Requirement: JSON info 和 full-read 必须暴露稳定事实

JSON info MUST 返回 source-derived content type、UTF-8、包含可选 BOM 的原文件 byte size、adapter id `docnav-json` 和 format id `json`。一个已成功解析的 source 实际包含 comment 或 accepted trailing comma 时，source-derived content type MUST 为 `application/jsonc`；否则 MUST 为 `application/json`。Comment markers inside strings MUST NOT trigger `application/jsonc`。JSON-specific metadata key set MUST 精确等于 `{root_kind, node_count, max_depth}`；`root_kind` MUST 使用 `object|array|string|number|boolean|null`，`node_count` MUST 包含 root，root depth MUST 为 `0`。

JSON adapter MUST 声明 unstructured full-read content 与 cost capability；full-read MUST 返回去除一个可选 UTF-8 BOM 后的原始 source text，不删除 comments、accepted trailing commas、whitespace、escapes 或 number spelling，并 MUST 使用与 info 相同的 source-derived content type 和针对实际返回 text 的 lines/bytes/tokens cost measurements。Pathname 或 profile-shaped content MUST NOT 让 generic JSON info、read 或 full-read 声称 GeoJSON、JSON-LD、notebook、HAR、webmanifest、SARIF、I-JSON、JCS 或其它 profile media validity；generic JSON operations MUST NOT 解析 profile links/contexts、执行 remote resolution、schema validation 或 canonicalization。

#### Scenario: 查看 strict JSON 摘要

- **WHEN** info 针对不含 JSONC-only syntax 的有效 strict JSON 文档执行
- **THEN** result 标识 `docnav-json`、format `json` 和 `application/json`
- **THEN** metadata 给出 root kind、包含 root 的 node count 和以 root 为 `0` 的 max depth

#### Scenario: JSONC info and full-read expose source facts

- **WHEN** a valid selected source contains an approved comment or trailing comma
- **THEN** info and unstructured full-read use format id `json` and content type `application/jsonc`
- **THEN** full-read returns the BOM-stripped source with that JSONC syntax unchanged
- **THEN** cost describes the actual returned source text

#### Scenario: 小 strict JSON 进入 unstructured full-read

- **WHEN** navigation policy 对不含 JSONC-only syntax 的 JSON 文档选择 unstructured full-read
- **THEN** adapter 返回去除可选 BOM 后的原始 JSON text 和 `application/json`
- **THEN** result 使用 unstructured full-read content/cost shape

#### Scenario: JSONC markers inside a string do not change source type

- **WHEN** an otherwise strict JSON string contains source characters `//` or `/* literal */`
- **THEN** info and full-read use `application/json`
- **THEN** the characters remain string source data

#### Scenario: Generic strict-profile input does not claim profile validity

- **WHEN** caller explicitly selects `docnav-json` for a valid strict GeoJSON-, JSON-LD-, or notebook-shaped source on any pathname
- **THEN** the source is navigated as generic JSON and reports `application/json`
- **THEN** Docnav does not substitute a profile media type, validate that profile, or resolve remote resources

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界

JSON adapter 主文档、adapter tests、case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖确定性 outline、empty-container 空 entries、root scalar entry、ASCII-safe ref grammar、空/特殊/control key roundtrip、context-sensitive array index、无损 JSON number、原文 occurrence 与 source-region-to-ref 映射、outline/find-to-read roundtrip、exact `DOCUMENT_CONTENT_INVALID`/encoding/ref errors、Unicode pagination、info/full-read、generic `readable-view`、manifest pathname routing、显式 adapter selection、selected parse failure 不 fallback，以及同一个 release binary 中的 linked JSON behavior。

Evidence MUST distinguish strict-document semantics from JSONC syntax and MUST cover the one grammar through automatic and explicit selection. The JSONC corpus MUST cover line/block/EOF comments at every strict-JSON whitespace boundary; comment markers in strings; LF/CRLF/CR including CR-only line comments; Unicode spelling and original byte offsets; unterminated comments; first-`*/` closure with valid `{"a": /* outer /* marker */ 1}` producing member value `1`; invalid `{"a": /* outer /* nested */ outer */ 1}` producing syntax failure from tokens left after the first closer; the trailing-comma rule for objects and arrays; rejected `{,}`/`[,]`, missing/doubled/extra commas, and every JSON5/loose syntax; optional/multiple BOM behavior; invalid UTF-8; multiple roots/trailing input; decoded duplicates; strict/raw numbers including `1e9999`; depths `127` and `128`; empty containers; root scalars; bounded original-offset recorded comment spans; wholly-single-comment versus ordinary-whitespace/trailing-comma/comment-whitespace/comment-token/cross-comment-boundary/multiple-comment-span find mapping; outline/find-to-read roundtrips; normalized structured read; source full-read; syntax-derived content types; pagination; cost; generic readable output; hostile large comments/lines/logical nesting; and selected failures without fallback. Strict JSON regression evidence MUST preserve ref, traversal, source order, raw number, source-region, pagination, cost, info/full-read, diagnostic, and raw/readable behavior required by the JSON adapter specification.

Representative adjacent-family compatibility evidence MUST prove that strict JSON profiles remain generic JSON, JSONC configuration variants use the same grammar regardless of pathname, JSON5 and multi-document sources remain rejected, deterministic read is not advertised as I-JSON/JCS validation or canonicalization, and CBOR/BSON do not introduce a shared model. Compatibility samples are parser/model evidence only: they MUST NOT be treated as manifest-routing evidence and MUST NOT cause pathname hints, profile media types, remote resolution, a public family/dialect mode, another ref model, or a binary parser to exist.

#### Scenario: 验证 JSON 产品与集成语义

- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned strict/JSONC parse、navigation、ref、content 和 error behavior
- **THEN** core/release 验证覆盖 exact manifest hints and content types、pathname/explicit static selection、closed input、no-fallback dispatch 和单一 binary linked behavior
- **THEN** 测试从 package core executable 运行，并证明 public input inventory 不包含 JSON-specific field

#### Scenario: Strict and JSONC matrices run together

- **WHEN** JSON adapter verification runs
- **THEN** strict JSON positive and negative cases prove the strict-document semantics required by the JSON adapter specification
- **THEN** JSONC cases prove accepted syntax, source-span/ref mapping, identity/output/diagnostic behavior through the same grammar
- **THEN** no parser-library default supplies an untested public behavior

#### Scenario: Adjacent families remain bounded

- **WHEN** representative JSON-family compatibility evidence runs
- **THEN** generic strict profiles and JSONC configuration shapes exercise the selected parser/model without profile semantics
- **THEN** JSON5, multiple-root, canonicalization, remote-resolution, and binary boundaries remain outside the delivered implementation
- **THEN** no additional pathname hint or public dialect/ref/model is inferred from the corpus

### Requirement: JSON selected operations 必须验证实际文档

When navigation selects `docnav-json` through manifest pathname routing or explicit adapter intent, every requested JSON strategy MUST acquire and parse the actual document once with one JSONC-capable grammar before using its private model. The adapter MUST remove at most one leading UTF-8 BOM, decode UTF-8, validate one complete root value with only trailing grammar whitespace, calculate root depth as `0` with maximum depth `127`, preserve strict JSON raw number tokens and object/array source order, and reject duplicate decoded member names within each object. A pathname hint or declared content type is only format metadata; it MUST NOT assert content validity, select a dialect, substitute for parsing, or enter the closed operation input.

Outside strings, the one grammar MUST accept `//` line comments and `/* ... */` block comments wherever strict JSON permits whitespace. A block comment MUST begin at `/*` and close at the first following `*/`; another `/*` before that closer MUST remain comment text and MUST NOT open nested state. It MUST accept EOF line comments, LF/CRLF/CR line endings, and one trailing comma after the final object member or array element. Comment markers inside strings MUST remain string data. Unterminated block comments, `#` comments, missing or multiple commas, single-quoted strings, unquoted property names, hexadecimal or leading-plus numbers, leading/trailing decimal points, `NaN`, infinity, multiple roots, and every other JSON5/JavaScript extension MUST be rejected. A source that relies on nested block-comment structure MUST be evaluated using first-closer semantics; tokens left after that closer MUST be parsed normally and cause `JSON_SYNTAX_INVALID` when they are not valid JSON/JSONC source. The adapter MUST NOT use pathname, parser success/failure, confidence, caller input, or a retry to choose a stricter or looser grammar.

Comments, accepted trailing commas, and other syntax trivia MUST NOT become logical nodes, outline entries, ref tokens, member values, or new node kinds. One primary ordered source-aware logical tree MUST serve traversal, ref resolution, structured read, info, and source occurrence mapping. Bounded original-offset recorded comment spans or an offset-preserving parse view MAY supplement that tree for grammar validation and the comment-only find override, but MUST NOT become a second full logical tree or change the JSON find requirement's ownership for ordinary strict whitespace, accepted trailing commas, or cross-boundary occurrences. Canonical `json:#` refs, traversal order, node count, empty-container/root-scalar behavior, pagination, cost, and logical results MUST be identical for strict JSON and JSONC sources with the same logical value.

A selected JSON document or operation failure MUST return the owner-compatible document or JSON adapter diagnostic and MUST NOT trigger pathname routing again, retry another parser mode, or dispatch another adapter. A document change between selection and the operation read MUST be reported according to the document state observed by the operation; the adapter MUST NOT emit error id `json-document-changed-after-probe`.

Invalid UTF-8 MUST continue to use `DOCUMENT_ENCODING_UNSUPPORTED`. Invalid JSON/JSONC syntax MUST use `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`; trailing non-whitespace input or a second root, duplicate decoded member names, and maximum-depth overflow MUST use `DOCUMENT_CONTENT_INVALID` with exact reasons `JSON_TRAILING_INPUT`, `JSON_DUPLICATE_MEMBER`, and `JSON_MAXIMUM_DEPTH_EXCEEDED`, respectively. Canonical content-invalid details MUST contain only the normalized `path` and stable `reason`; parser-library types/messages, recovery traces, unstable offsets, duplicate names, dependency details, and confidence MUST remain private.

#### Scenario: Automatically selected JSON operation parses the selected source

- **WHEN** one manifest pathname lookup maps a document to format `json`
- **AND** registry lookup selects `docnav-json`
- **THEN** the requested JSON strategy parses the actual document once with the one JSONC-capable grammar before producing operation facts
- **THEN** JSON ref, depth, duplicate-member, number, source-region, and recorded-comment-span semantics remain JSON-owned

#### Scenario: JSONC syntax in a `.json` file is accepted

- **WHEN** automatic routing selects `docnav-json` for a `.json` pathname
- **AND** the document uses only strict JSON plus approved comments or trailing commas
- **THEN** the selected strategy accepts it with the same grammar used for `.jsonc`, `.code-workspace`, exact JSON filenames, and explicit selection
- **THEN** navigation does not parse content or pass a dialect into the adapter

#### Scenario: Broader JSON5 syntax remains unsupported

- **WHEN** a selected document uses an unapproved extension such as a single-quoted string, unquoted member name, missing/multiple comma, hexadecimal number, unary plus, `NaN`, infinity, or multiple root values
- **THEN** `docnav-json` rejects the document with the applicable JSON-owned content diagnostic
- **THEN** no stricter/looser parser retry or adapter fallback occurs

#### Scenario: Inner block-comment opener is comment text

- **WHEN** selected source is `{"a": /* outer /* marker */ 1}`
- **THEN** the first `*/` closes the only block comment
- **THEN** the inner `/*` does not open nested state
- **THEN** the document is accepted with member `a` equal to number `1`

#### Scenario: Nesting-dependent block-comment source fails after the first closer

- **WHEN** selected source is `{"a": /* outer /* nested */ outer */ 1}`
- **THEN** the first `*/` closes the only block comment
- **THEN** `outer */` remains ordinary source after the comment
- **THEN** the document fails with `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`

#### Scenario: Explicit JSON selection does not waive parse

- **WHEN** caller explicitly selects `docnav-json` on any pathname
- **AND** the actual document violates the one JSONC grammar or safety rules
- **THEN** navigation skips automatic pathname routing
- **THEN** the selected JSON strategy returns its owner-compatible document or `DOCUMENT_CONTENT_INVALID` diagnostic
- **THEN** no other parser mode or adapter is attempted

#### Scenario: Pathname alias does not prove JSON validity

- **WHEN** `.prettierrc`, `.code-workspace`, or `.jsonc` matches a JSON manifest pathname hint
- **AND** its content is outside the selected JSON adapter grammar
- **THEN** the selected operation returns its normal JSON-owned parse diagnostic
- **THEN** navigation does not retry format routing or another adapter

#### Scenario: Document changes after pathname selection

- **WHEN** navigation routes a pathname to `docnav-json`
- **AND** the path content changes before the selected operation reads it
- **THEN** the operation validates the document view it actually opens
- **THEN** a read or JSON validation failure uses the applicable document or `DOCUMENT_CONTENT_INVALID` diagnostic
- **THEN** error id `json-document-changed-after-probe` is not emitted
