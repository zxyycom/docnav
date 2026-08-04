# json-adapter Specification

## Purpose
定义内置 JSON adapter 的 Target JSONC comment-aware 契约：静态注册、格式识别、closed grammar、comment attribution、canonical ref views、导航、读取、原文查找、info、full-read、安全边界与验证。本文的 Target requirements 在 `add-jsonc-comment-aware-navigation` 实现、验收和归档前不代表 Current binary 行为；shared protocol、output shape 与 opaque ref pass-through 保持既有 owner。

## Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json` 和一个 normalized format id `json` descriptor 暴露 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。该 descriptor 的 `extensions[]` basename suffixes MUST 精确等于 `.json`、`.code-workspace` 和 `.jsonc`；`filenames[]` exact basename hints MUST 精确等于 `.prettierrc` 与 `.watchmanconfig`；`content_types[]` MUST 精确等于 `application/json` 与 `application/jsonc`。它 MUST NOT 声明其它 JSON-family pathname hint、adapter identity 或 format identity。

JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于既有 closed standard operation input；注册 JSONC MUST NOT 增加 core parameter、`StandardInputBinding`、CLI、env、config 或 protocol input。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明一个 JSON format、`.json`、`.code-workspace` 与 `.jsonc` suffixes、`.prettierrc` 与 `.watchmanconfig` exact filenames，以及 `application/json` 与 `application/jsonc`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** matched filename、suffix、content type 或 format identity 不进入 strategy input
- **THEN** core public input inventory 不包含 JSON-specific field

#### Scenario: 所有 JSON pathname 使用同一个 grammar

- **WHEN** `.json`、`.jsonc`、`.code-workspace`、exact filename 或 explicit adapter intent 选择 `docnav-json`
- **THEN** selected operation 使用同一个 JSONC-capable grammar
- **THEN** pathname 和 descriptor content type 不选择 strict/JSONC dialect

### Requirement: JSON ref 必须使用 canonical ASCII-safe JSON Pointer grammar

JSON adapter MUST 生成和解析三种非空、ASCII-safe ref view：base ref `json:#<fragment>`、direct-comment ref `json:comments:#<fragment>` 与 tail-comment ref `json:tail-comments:#<fragment>`。三者的 `#<fragment>` MUST 使用同一个 RFC 6901 URI fragment identifier representation并锚定同一个 logical JSON path；`comments:`选择该 navigation binding的 direct-comment bundle，`tail-comments:`选择以该 logical value为tail anchor的tail-comment bundle。View marker MUST NOT形成 logical JSON node、pointer token、format identity或按 source offset持久化的 comment identity。Root base、direct-comment与tail-comment refs分别为 `json:#`、`json:comments:#` 与 `json:tail-comments:#`。

Adapter MUST 先把 object token 中 `~` 和 `/` canonical escape 为 `~0` 和 `~1`，再以 UTF-8 和大写十六进制 percent escape 编码 URI fragment 不允许的 bytes；生成的 ref MUST 不包含原始 NUL 或其它控制字符。空 object key 的 base ref MUST 为 `json:#/`；其 direct-comment与container-tail refs分别 MUST为 `json:comments:#/` 与 `json:tail-comments:#/`。当当前节点是 array 时，token MUST 为 `0` 或不带前导零的十进制 index，且 `-` 不可作为可读节点；相同 token 在 object 上仍按 member name 解释。

Outline MUST为至少含一个direct source comment的root、object member或array element生成direct-comment ref，并为每个非空tail-comment bundle生成tail-comment ref；base ref MUST始终继续被read接受。Canonical comment ref的logical anchor path存在、但所选direct或tail bundle不存在时，read MUST返回`REF_NOT_FOUND`。缺少合法`json:#` / `json:comments:#` / `json:tail-comments:#` prefix、未知view marker、non-root fragment缺少前导`/`、非法或non-canonical percent/`~` escape，以及array token非canonical MUST返回`REF_INVALID`；grammar canonical但当前文档不存在的logical anchor path MUST返回`REF_NOT_FOUND`。

#### Scenario: 特殊 object key roundtrip

- **WHEN** JSON object 包含空 key、key `a/b~c` 和含控制字符或非 ASCII 字符的 key
- **THEN** base refs 分别使用 `json:#/`、`json:#/a~1b~0c` 和 canonical percent-encoded ASCII-safe spelling
- **THEN** 有 direct comments或container-tail bundle的相同 members分别只把 prefix改为 `json:comments:#` 或 `json:tail-comments:#`，pointer token spelling不变
- **THEN** read 接收任一有效 view ref 后定位对应 logical value

#### Scenario: 区分非法 ref 与当前文档无匹配

- **WHEN** read 收到 `json:#/items/01` 且 `items` 是 array
- **THEN** adapter 返回 `REF_INVALID`
- **WHEN** read 收到 `json:#/object/01` 且 `object` 是包含 key `01` 的 object
- **THEN** read 返回该 object member
- **WHEN** read 收到 canonical `json:#/items/9` 但 array 没有该 index
- **THEN** adapter 返回 `REF_NOT_FOUND`

#### Scenario: Direct-comment ref 不改变 base identity

- **WHEN** member `retries` 有 direct comments
- **THEN** outline 可以生成 `json:comments:#/retries`
- **THEN** `json:#/retries` 与 `json:comments:#/retries` 定位同一 logical value
- **THEN** 删除 comments 后 base ref 仍可读取，而旧 direct-comment ref 返回 `REF_NOT_FOUND`

#### Scenario: Tail-comment ref以tail-anchor path锚定独立选择

- **WHEN** container `options`在最后一个 child后拥有 independent-tail comments
- **THEN** outline生成 `json:tail-comments:#/options`
- **THEN** `json:#/options`、可能存在的 `json:comments:#/options` 与 tail ref共享 canonical logical path但选择不同 view
- **THEN** 删除或把 tail bundle移到另一 scope后旧 tail ref返回 `REF_NOT_FOUND`，base ref继续可读

#### Scenario: Root 与 array element 使用同一 direct-comment view grammar

- **WHEN** root拥有 root-leading comment，且 root array的 index `0`拥有 element comment
- **THEN** outline分别生成 `json:comments:#` 与 `json:comments:#/0`
- **THEN** 对应 base refs `json:#` 与 `json:#/0` 继续定位相同 logical values
- **THEN** 任一 binding失去全部 direct comments后，只有该 direct-comment ref返回 `REF_NOT_FOUND`

### Requirement: JSON outline 必须提供确定性扁平树导航

JSON outline MUST 对expanded navigation tree进行depth-first preorder遍历。Logical object member与array element各形成一个带完整JSON ref、非空label和`object|array|string|number|boolean|null` value kind的entry；每个非空tail-comment bundle另形成tail anchor的最后一个virtual child entry，其label MUST为`<tail comments>`、kind MUST为`tail_comments`、ref MUST为canonical tail-comment ref。Object member MUST按源码顺序遍历，array element MUST按index升序遍历，因此nested tail entry出现在anchor container全部logical descendants之后，root-tail entry出现在全部root entries之后。Virtual tail entry MUST NOT进入logical tree、JSON node count或JSON Pointer tokens。Object child label MUST使用decoded member name，空key的正常label MUST为两个双引号字符`""`；array child label MUST为`[<index>]`。Root object/array有root direct comments时，outline MUST在所有descendants前新增唯一`<root>` entry；没有root direct comments时MUST保持Current不返回root-container logical entry的行为。

含至少一个direct comment的root、object member或array element entry MUST使用其canonical direct-comment ref；其它logical entries MUST使用base ref。Direct或tail bundle的normalized summary非空时，对应entry MUST在既有optional `summary` field中返回该单行文本；summary MUST NOT 包含 CR 或 LF。所有logical与virtual JSON outline entries MUST保持Current不返回source `location`或JSON-specific `metadata`的行为。Tail entry的raw field set MUST精确等于`{ref, label, kind}`加optional `summary`；`location`、`metadata`、`excerpt`、`rank`与entry-level `cost` MUST省略。没有root direct comments时，root object/array的logical entry set MUST只由descendants组成；但root tail bundle仍 MUST在这些descendants之后生成其virtual entry。既无root direct comments、logical descendants也无root tail bundle的空object/array MUST返回空entries和null page。Root scalar MUST始终先返回label `<root>`且kind对应scalar类型的logical entry，并按是否有root direct comments选择direct-comment或base root ref；其root tail entry如存在MUST随后返回。

Outline MUST 使用现有 limit/page 契约分页并保持 preorder。完整 ref MUST 始终保留；预算不足时 MUST 先在 Unicode scalar boundary 截断或省略 optional summary：保留部分 summary 时必须以 `...` 标记截断，预算不足以保留有意义的 partial summary 时必须省略该 field。随后 adapter 才按 Current label 规则截断，且分页必须持续前进。分页预算截断后没有可见的正常 label 内容可保留时，最小非空 label MUST 为 `.`，且该 fallback MUST NOT 替代空 key 的正常 label `""`。任何 returned ref MUST 能原样传给 read。

#### Scenario: 遍历混合 JSONC 树

- **WHEN** root object 含有 object、array、scalar descendants 和部分带direct comments的members
- **THEN** outline 按 object member 源码顺序和 array index 顺序执行 depth-first preorder
- **THEN** container 与 scalar entry 的 kind 反映 logical JSON value kind
- **THEN** 有 direct comments的 root/member/index target使用 direct-comment ref和非空 normalized summary，其他 logical entry使用 base ref
- **THEN** 每个tail-comment bundle在其tail-anchor subtree末尾使用tail ref、`<tail comments>` label、`tail_comments` kind与可用的normalized summary
- **THEN** 每个 entry ref 都能原样传给 read

#### Scenario: Root binding 与 tail slot 都无 comments 的空容器没有可展开节点

- **WHEN** JSON/JSONC文档的logical root是`{}`或`[]`，root binding没有leading、same-line trailing或empty-container direct comments，且complete root后没有document-tail bundle
- **THEN** outline 返回空 entries 和 null page

#### Scenario: Root comment 新增或升级 root entry

- **WHEN** root object或 array拥有 root-leading、same-line trailing或 empty-container comments
- **THEN** outline在 descendants前新增 `<root>` entry、对应 root kind、`json:comments:#` 和非空 normalized summary
- **WHEN** root是 scalar且拥有相同 root direct comments
- **THEN**既有唯一 `<root>` entry使用 `json:comments:#`；没有 root direct comments时仍使用 `json:#`

#### Scenario: Array element comment 使用 index ref

- **WHEN** root或nested array的index `0`拥有direct comments
- **THEN**该 `[0]` entry使用对应 canonical `json:comments:#/0` 或 nested index path
- **THEN**其他无 comments的 array elements继续使用 base refs

#### Scenario: Tail-comment entry位于tail-anchor subtree末尾

- **WHEN** nested container `/options`与document root各自拥有 independent-tail bundle
- **THEN** `/options`的 `<tail comments>` entry在其全部 descendants之后、下一个 sibling之前使用 `json:tail-comments:#/options`
- **THEN** root `<tail comments>` entry在全部其它 outline entries之后使用 `json:tail-comments:#`
- **THEN** root只有 tail bundle而没有 root direct comments时不额外生成 `<root>` logical entry

#### Scenario: Comment summary 服从 entry budget

- **WHEN** direct或tail entry的 normalized comment summary与完整 ref、label合计超过当前 outline limit
- **THEN** pagination保留完整 comment ref和该 entry的正常非空 label
- **THEN** summary 在 Unicode scalar boundary 被显式截断或省略
- **THEN** 后续 entry 的顺序和 page forward progress 不变

### Requirement: JSON read 必须返回 ref 所选择的规范化 view

JSON read MUST解析JSON-owned ref，并为base、direct-comment与tail-comment refs生成三个确定性view。Base ref MUST把指定logical JSON value序列化为valid strict JSON：object member保持源码顺序，container layout使用两空格缩进，每个number使用原始strict-JSON token，string escape、普通scalar spelling和尾随换行使用workspace-pinned serializer的pinned result；comments与trailing commas MUST不进入base content，content type MUST为`application/json`。

Direct-comment ref MUST先按source order输出只归属于selected navigation binding的完整raw comment tokens，再输出该logical value的同一strict-JSON serialization。Tail-comment ref MUST先输出只归属于selected tail slot的完整raw tokens，再输出tail-anchor logical value的同一serialization。每个raw token MUST包含原始`//`或`/* ... */` delimiters但不包含line-comment terminator；adapter MUST在每个token后插入一个LF byte`0x0A`，使最后一个comment与value分隔。两种comment content MUST是完整valid JSONC document，MUST NOT包含其它direct/tail、ancestor、descendant或sibling comments，也MUST NOT恢复source trailing comma；content type MUST为`application/jsonc`。这些view是本 change 的 comment-aware projections，不限制private selection consumer未来使用完整selected-first context。

三种 view 的 ReadResult MUST 保留输入 ref，对分页前的完整 content 计算 cost，并按现有 Unicode-safe text pagination 返回 content 和下一页。Generic `readable-view` MUST 从同一个 raw result 形成既有 read header 与 `/content` block，不重读文档、不解析 ref 或重建 comments。

#### Scenario: Base read 规范化 JSONC value

- **WHEN** base read 收到 `json:#/retries`，且 source member 有 comments 或 trailing comma
- **THEN** content 是 `retries` value 的确定性 strict JSON serialization
- **THEN** comments 与 trailing commas 不在 content 中
- **THEN** content type 为 `application/json`

#### Scenario: Direct-comment read 带出归属注释

- **WHEN** direct-comment read 收到 `json:comments:#/retries`
- **AND** `retries` 依 source order 归属 `// Maximum attempts` 与 `/* before retrying */`
- **THEN** content 依次包含两个完整 comment tokens、各自后的 LF，以及 normalized logical value
- **THEN** content 是完整 JSONC document，content type 为 `application/jsonc`
- **THEN** nested members 自己的 comments 不被合并进该 view

#### Scenario: Root 与 array element direct-comment read

- **WHEN** read收到带 root direct comments的 `json:comments:#`
- **THEN** content包含 root direct-comment tokens与 normalized完整 root value
- **WHEN** read收到带 element comments的 `json:comments:#/items/0`
- **THEN** content只包含 index `0` comment bundle与该 element的 normalized value

#### Scenario: Tail read带出tail-comment bundle

- **WHEN** read收到 `json:tail-comments:#/options`
- **THEN** content只包含 `/options` tail anchor的independent-tail tokens与normalized `/options` value
- **THEN**同一路径的 direct comments、children comments与ancestor comments不进入该 projection
- **THEN** content是完整 JSONC document，content type为 `application/jsonc`

#### Scenario: 分页读取 comment Unicode content

- **WHEN** direct或tail comments 与 value 的完整 content 超过 limit 且包含多字节 Unicode 字符
- **THEN** read 在字符边界分页
- **THEN** cost 描述分页前的完整 comment content
- **THEN** page 指向下一页直至内容结束

### Requirement: JSON find 必须返回可继续读取的节点

JSON find MUST 接受长度至少为一的 query，并 MUST 拒绝长度为零的 query。它 MUST 对去除一个可选开头 UTF-8 BOM 后的原文执行大小写敏感、从左到右、非重叠的 literal search，包括 comments、trailing commas 与原始 string/number spelling；canonical refs 和 read serialization 不扩充 find 语料。

Adapter-private source regions MUST让每个命中确定性归属到可读取selection：root region覆盖完整BOM-stripped source；object child region从member name token开始并覆盖其value；array child region覆盖其value。完全位于direct-comment span内的occurrence MUST返回该binding的direct-comment ref；完全位于tail-comment span内的occurrence MUST返回tail-anchor ref。其它occurrence，包括ordinary whitespace、trailing comma、logical token、string内comment markers，以及跨越comment boundary或多个regions的occurrence，MUST使用Current deepest-covering source-region rule并返回base ref；跨越多个child regions时MUST映射到同时覆盖它们的最近container。

Find MUST 为每个 source occurrence 返回一个 match，并按 source offset 排序；多个 occurrences 映射到同一 ref 时 MUST 保留多个 matches。每个 match MUST 带完整 ref、`kind: "match"`、从原文派生的非空 bounded excerpt label 和 source line location。Entry pagination MAY 截断 display facts，并在完整 ref 已耗尽预算时只保留最小非空 label；分页 MUST 保持 occurrence order 并持续前进。返回 ref MUST 能原样传给 read。Generic `readable-view` MUST 只使用这些 raw entry facts。

#### Scenario: Member name 命中映射到 base value

- **WHEN** 原文 object member name `a/b` 命中 query `a/b`
- **THEN** find 返回该 member value 的 base ref `json:#/a~1b`
- **THEN** member 即使另有direct comments，普通name/value occurrence也不切换view

#### Scenario: Direct comment命中进入direct-comment read

- **WHEN** query occurrence 完全位于归属于 member `retries` 的一个 comment span 内
- **THEN** find 返回 `json:comments:#/retries`
- **THEN** read该ref返回direct comments与logical value的JSONC view
- **WHEN** occurrence位于root或array index `0`的direct-comment span内
- **THEN** find分别返回 `json:comments:#` 或该 index的 canonical direct-comment ref

#### Scenario: Tail comment命中进入tail read，跨边界命中保持positional mapping

- **WHEN** occurrence完全位于 non-empty container或document的 independent-tail span
- **THEN** find返回对应canonical `json:tail-comments:#<anchor-fragment>` ref
- **THEN** read该ref返回tail bundle与tail-anchor value
- **WHEN** occurrence跨越 comment与 token/whitespace boundary
- **THEN** comment-view override不适用，find使用覆盖完整 occurrence的 Current logical source region并返回base ref

#### Scenario: 搜索保留原文 spelling

- **WHEN** string value 的原文 token 包含 `\u0061`，或 comment 包含 source-only spelling
- **THEN** query 命中对应 source occurrence
- **THEN** read serializer 与 comment projection 不改变 find 语料

#### Scenario: 同一节点的多个源码命中分别返回

- **WHEN** 同一logical value或comment bundle中存在两个non-overlapping query occurrences
- **THEN** find 按 source offset 返回两个 matches
- **THEN** 两个 matches MAY 携带相同 ref，且该 ref 均可原样传给 read

### Requirement: JSON info 和 full-read 必须暴露稳定事实

JSON info MUST 返回 source-derived content type、UTF-8、包含可选 BOM 的原文件 byte size、adapter id `docnav-json` 和 format id `json`。已成功解析的 source 实际包含 comment 或 accepted trailing comma 时，source-derived content type MUST 为 `application/jsonc`；否则 MUST 为 `application/json`。String token 中的 comment-like text MUST NOT 触发 `application/jsonc`。JSON-specific metadata key set MUST 精确等于 `{root_kind, node_count, max_depth}`；`root_kind` MUST 使用 `object|array|string|number|boolean|null`，`node_count` MUST 包含 root，root depth MUST 为 `0`。

JSON adapter MUST 声明 unstructured full-read content 与 cost capability；full-read MUST 返回去除一个可选 UTF-8 BOM 后的原始 source text，不删除 comments、trailing commas、whitespace、escapes 或 number spelling，并 MUST 使用与 info 相同的 source-derived content type和针对实际返回 text 的 lines/bytes/tokens cost measurements。

#### Scenario: 查看 strict JSON 摘要

- **WHEN** info 针对不含 JSONC-only syntax 的有效 strict JSON 文档执行
- **THEN** result 标识 `docnav-json`、format `json` 和 `application/json`
- **THEN** metadata 给出 root kind、包含 root 的 node count 和以 root 为 `0` 的 max depth

#### Scenario: JSONC info 与 full-read 保留 source facts

- **WHEN** 有效 source 含 accepted comment 或 trailing comma
- **THEN** info 与 unstructured full-read 使用 format `json` 和 content type `application/jsonc`
- **THEN** full-read 返回 BOM-stripped source，且 JSONC syntax 保持原样
- **THEN** cost 描述实际返回的 source text

#### Scenario: String 内 marker 不是 JSONC syntax

- **WHEN** otherwise strict JSON string 含 source characters `//` 或 `/* literal */`
- **THEN** info 与 full-read 使用 `application/json`
- **THEN** characters 继续作为 string source data

### Requirement: JSON adapter 必须用 owner 证据覆盖产品边界

JSON adapter 主文档、adapter tests、Case ledger、coverage mapping、core CLI smoke 和 release package smoke MUST 覆盖 strict/JSONC grammar、deterministic attribution、direct/tail outline projection、base/direct-comment/tail refs、三种 read views、comment find-to-read、info/full-read、manifest facts、source offsets、raw number、decoded duplicates、depth、error mapping、Unicode pagination、cost、generic `readable-view`、automatic/explicit selection、selected failure no-fallback 和同一个 release binary 中的 linked JSON behavior。

Evidence MUST把parser behavior、private source model与observable contract分层证明：strict/no-comment JSON的existing refs、entries、read、find、content types和diagnostics不回归；JSONC cases覆盖root/member/index placement、empty-container-self attribution、independent-tail anchor boundary与virtual entry、comment kind、trailing comma、malformed input、stale direct/tail ref和raw/readable parity；broader JSON5 syntax与multiple roots保持rejected。Large/deep/comment-heavy input MUST证明comment indexes、summary construction、attribution lookup、find和drop behavior的work/memory有界，不为每个entry或occurrence全量扫描comment set。

#### Scenario: 验证 JSON 产品与集成语义

- **WHEN** JSON adapter 的 owner 与 release 验证运行
- **THEN** 验证覆盖 JSON-owned parse、attribution、navigation、ref/view、content 和 error behavior
- **THEN** core/release 验证覆盖 exact manifest facts、pathname/explicit static selection、closed input、opaque ref pass-through、no-fallback dispatch 和单一 binary linked behavior
- **THEN** schema-valid protocol 与 generic readable output 使用同一 adapter facts，且 public input/output shape 未增加 JSON-specific field

#### Scenario: Parser default 不扩大产品 grammar

- **WHEN** selected implementation 的 library default 接受本 spec 契约外的 JSON5、missing-comma 或 multi-root syntax
- **THEN** adapter corpus 证明该 behavior 已被关闭或在 adapter boundary 拒绝
- **THEN** dependency token、AST、error message 或 attachment heuristic 不进入 public contract

#### Scenario: Unique comment ref 复用既有 auto-read

- **WHEN** current outline/find page 的非空 refs 去重后恰好是一个 canonical direct-comment 或 tail-comment ref
- **AND** core 的 existing `unique-ref` auto-read mode 启用
- **THEN** navigation 把该 opaque ref 原样传给同一 selected adapter 的 read strategy
- **THEN** successful `auto_read.read` 保留该 comment view 的 ref、`application/jsonc` content、cost 和 page
- **THEN** protocol-json 与 generic readable-view 从同一 composed response 派生，readable nested content 使用既有 `/auto_read/read/content` block

### Requirement: JSON selected operations 必须验证实际文档

When navigation selects `docnav-json` through manifest pathname routing or explicit adapter intent, every requested JSON strategy MUST acquire and parse the actual document once with the same JSONC-capable grammar before using its private model. The adapter MUST remove at most one leading UTF-8 BOM, decode UTF-8, validate one complete root value with only trailing grammar trivia, calculate root depth as `0` with maximum depth `127`, preserve strict JSON raw number tokens and source order, and reject duplicate decoded member names within each object. A pathname hint or declared content type is only format metadata; it MUST NOT assert content validity, select a dialect, substitute for parsing, or enter the closed operation input.

Outside strings, comments MUST be accepted only where strict JSON permits whitespace. Grammar trivia MUST contain only strict-JSON SP/HTAB/LF/CR、`//` line comments or `/* ... */` block comments. A line comment MUST end immediately before LF、CRLF、lone CR or at EOF；a block comment MUST close at the first following `*/` and MUST NOT nest. Object/array grammar MAY contain at most one trailing comma only after a member/element in a non-empty container；`{,}`、`[,]`、missing/doubled commas remain invalid。Strict JSON `value`、string 与 number grammar MUST remain unchanged；single quotes、unquoted names、hexadecimal or leading-plus numbers、`NaN`、infinity、JSON5 extensions and multiple roots MUST be rejected。

Comments, trailing commas and trivia MUST NOT become logical JSON nodes、logical child entries、pointer tokens或JSON value kinds。A non-empty tail-comment bundle MUST形成一个`tail_comments` virtual navigation entry，但该entry只选择tail-anchor path的tail view，不进入logical tree、node count或pointer grammar。One primary ordered source-aware logical tree MUST serve traversal、ref resolution、read、info and source occurrence mapping；bounded ordered comment spans、navigation-binding direct bundles与tail-anchor bundles MAY supplement it, but parser/CST types MUST remain private and a second full logical tree MUST NOT be created。

A selected JSON document or operation failure MUST return the owner-compatible document or JSON adapter diagnostic and MUST NOT trigger pathname routing again、retry another parser mode or dispatch another adapter。A document change between selection and the operation read MUST be reported according to the document view observed by that operation；the removed error id `json-document-changed-after-probe` MUST NOT be emitted。

Invalid UTF-8 MUST use `DOCUMENT_ENCODING_UNSUPPORTED`。Malformed JSON/JSONC syntax、unterminated comments and rejected leniency MUST use `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`；non-trivia input or a second root after one complete root MUST use `DOCUMENT_CONTENT_INVALID / JSON_TRAILING_INPUT`；duplicate decoded member names and maximum-depth overflow MUST use `DOCUMENT_CONTENT_INVALID` with reasons `JSON_DUPLICATE_MEMBER` and `JSON_MAXIMUM_DEPTH_EXCEEDED`。Canonical details MUST contain only normalized `path` and stable `reason`；parser-library types/messages、unstable offsets、duplicate names、dependency traces and recovery state MUST remain private。

#### Scenario: Automatically selected JSONC operation parses current content

- **WHEN** manifest pathname lookup maps a document to format `json` and registry selects `docnav-json`
- **THEN** requested strategy parses the actual document once with the one JSONC-capable grammar before producing operation facts
- **THEN** logical JSON、ref、depth、duplicate、raw number、source region and comment evidence remain JSON-owned

#### Scenario: JSONC syntax in `.json` 使用同一个 grammar

- **WHEN** automatic routing selects `docnav-json` for a `.json` pathname
- **AND** source uses only strict JSON plus comments or trailing comma allowed by this requirement
- **THEN** selected strategy accepts it exactly as it would under `.jsonc`、`.code-workspace` or explicit selection
- **THEN** routing does not pass a dialect into the adapter

#### Scenario: Broader syntax remains unsupported

- **WHEN** selected source uses single quotes、unquoted names、missing/doubled commas、hexadecimal/unary-plus numbers、`NaN`、infinity or multiple roots
- **THEN** adapter returns the applicable JSON-owned content diagnostic
- **THEN** no alternate parser mode or adapter is attempted

#### Scenario: Explicit JSON selection does not waive parse

- **WHEN** caller explicitly selects `docnav-json`
- **AND** actual document violates grammar、UTF-8、duplicate or depth rules
- **THEN** navigation skips automatic pathname routing
- **THEN** selected JSON strategy returns its owner-compatible document diagnostic
- **THEN** no other adapter is attempted

#### Scenario: Pathname alias does not prove JSON validity

- **WHEN** `.prettierrc`、`.code-workspace` or `.jsonc` matches a JSON pathname hint
- **AND** content is outside the selected JSONC grammar
- **THEN** selected operation returns its normal JSON-owned parse diagnostic
- **THEN** navigation does not retry format routing or another adapter

#### Scenario: Document changes after pathname selection

- **WHEN** navigation routes a pathname to `docnav-json`
- **AND** path content changes before the selected operation reads it
- **THEN** operation validates the document view it actually opens
- **THEN** read or validation failure uses the applicable document or content diagnostic
- **THEN** error id `json-document-changed-after-probe` is not emitted


### Requirement: JSONC comments 必须按syntax placement归属于direct binding或tail slot

After parsing the closed grammar, `docnav-json` MUST assign every source comment exactly one attribution result：the logical root、one object member、one array element或一个`Tail(tail_anchor)` slot。Root、member与element分别以root selector、decoded key与canonical index作为direct navigation binding；array index在attribution与ref continuation中承担和object key相同的binding职责。Nested object/array tail的anchor MUST为该container logical value的canonical path，complete root后的document-tail anchor MUST为root path，包括root scalar。每个canonical tail-anchor path MUST至多对应一个tail slot；root container闭合符前与complete root后的tail comments MUST合并到root slot并保持source order。Attribution MUST使用本requirement拥有的source tokens、regions与lexical line boundaries；parser-provided previous/next attachment MUST NOT替代这些规则。A lexical line ends at LF、CRLF、lone CR或EOF；该attribution line model不改变Current LF-counted `find.location` behavior。

Adapter MUST按以下顺序处理每个 comment：

1. Complete root value之前的root-leading grammar trivia归root direct binding；root value之后与其complete token或container closing token从同一lexical line开始的comment也归root direct binding。Root之后从独立后续lexical line开始的document-tail comment归`Tail(root)`，与root是object、array或scalar无关。
2. 其它 comments先选择 deepest enclosing object或 array syntax context；nested value内的 comment不能归 ancestor binding。
3. 在object中，member-name token之后、member-value token之前的header trivia，以及complete member value之后、separator comma之前的suffix trivia归当前member。Opening token或previous separator comma之后、next member name之前的comment归next member；但存在previous member且comment与previous value或comma从同一lexical line开始时归previous member。Last member或optional trailing comma之后、closing token之前的comment在与last value/comma同一行时归last member；从独立后续lexical line开始时归`Tail(this object)`。
4. 在array中，complete element value之后、separator comma之前的suffix trivia归当前index。Opening token或previous separator comma之后、next element value之前的comment归next index；但存在previous element且comment与previous value或comma从同一lexical line开始时归previous index。Last element或optional trailing comma之后、closing token之前的comment在与last value/comma同一行时归last index；从独立后续lexical line开始时归`Tail(this array)`。
5. Empty object/array内的 container-only comments归该 container value自身的 direct navigation binding：nested container使用其 parent object key或 array index，root empty container使用 root selector。没有 child时不因container内部comment创建tail slot；complete empty root之后的document-tail仍按rule 1进入root tail slot。

每个navigation binding与每个canonical tail anchor MUST各自至多拥有一个optional comment bundle。`None` MUST只表示该slot没有comments；`Some` MUST包含至少一个source-ordered comment index，并保留每个token的exact BOM-stripped half-open UTF-8 byte span。Bundle内的comment spans不要求连续；root internal/document tail合并时仍按各token原始offset排序。多条comments归同一binding或tail slot时共享一个bundle与一个ref；同一comment MUST NOT同时进入direct与tail bundle或两个anchors。A raw line-comment token MUST include `//` and its body but exclude its terminator；a raw block token MUST include `/*` through the first `*/`。For outline summary only, adapter MUST remove delimiters、collapse every Unicode whitespace run in each body to one ASCII space、trim each body、discard empty bodies and join the remaining bodies with `; `。The derived summary MUST NOT contain CR or LF。An empty derived summary MUST NOT erase the comment bundle或其comment view；it only omits `Entry.summary`。Bundle MUST NOT cache a second complete normalized copy of the comment text。

#### Scenario: Root-leading and same-line trailing trivia belongs to root

- **WHEN** source在完整 root value之前包含 comments，或 comment与 complete root token从同一 lexical line开始
- **THEN**这些 comments唯一归 logical root，不因 root kind改变
- **THEN** outline对 root object/array新增或对 root scalar升级 `json:comments:#` entry
- **WHEN** comment在 root之后从独立后续行开始
- **THEN**它归`Tail(root)`并在全部root entries之后生成`json:tail-comments:#` virtual entry
- **THEN**它不改变 root logical entry使用的 base或direct-comment ref

#### Scenario: Leading comment belongs to the following member

- **WHEN** source is `{ // retries<LF> "retries": 3 }`
- **THEN** the comment belongs uniquely to member `retries`
- **THEN** outline generates `json:comments:#/retries` with summary `retries`

#### Scenario: Same-line trailing comment belongs to the previous member

- **WHEN** source is `{ "retries": 3, // attempts<LF> "timeout": 5 }`
- **THEN** comment `// attempts` belongs to member `retries`
- **THEN** a following comment that starts on a later lexical line before `"timeout"` belongs to member `timeout`

#### Scenario: Array comment belongs to an index binding

- **WHEN** source is `[ // first<LF> 1, 2 ]`
- **THEN** comment `// first` belongs uniquely to index `0` and outline uses `json:comments:#/0`
- **WHEN** comment starts on the same line after element `1` or its comma
- **THEN** it belongs to index `0` rather than the following index

#### Scenario: Header or suffix comment belongs to the current target

- **WHEN** a comment occurs after object member name and before its value token、or after a complete member/element value and before its separator comma
- **THEN** it belongs to that current member/index regardless of whether it starts on a later lexical line
- **THEN** a comment lexically inside a nested value is evaluated in the nested context instead

#### Scenario: Empty-container comments belong to the container value

- **WHEN** comments occur inside an empty object/array selected by an object key、array index或 root selector
- **THEN** comments归该 empty container binding并生成对应 direct-comment ref、summary与 read view
- **THEN** nested empty container可通过其 parent key/index path读取，root empty container使用 `json:comments:#`

#### Scenario: Independent tail comments获得tail-comment ref

- **WHEN** comment在 non-empty container最后一个 member/element之后从独立后续行开始，或在 complete root之后从独立后续行开始
- **THEN** comment唯一进入所在container或root tail anchor的ordered tail bundle
- **THEN** outline在该tail-anchor subtree末尾生成一个`tail_comments` virtual entry、canonical `json:tail-comments:#<anchor-fragment>` ref与可用的summary
- **THEN** read该ref返回tail tokens与normalized tail-anchor value，find在完整tail span内命中时返回同一ref

#### Scenario: Root内部与文档tail共享唯一root ref

- **WHEN** non-empty root object/array在closing token前与complete root后都拥有independent-tail comments
- **THEN**两段tokens按source order进入一个root tail bundle
- **THEN** outline只生成一个 `json:tail-comments:#` virtual entry，read一次返回该bundle与normalized root value

#### Scenario: Nested value uses the deepest attribution context

- **WHEN** an outer target value contains a nested object/array whose child target has a leading comment
- **THEN** the comment belongs to the nested member/index
- **THEN** no ancestor target inherits or duplicates that comment

#### Scenario: Multiple comments share one slot bundle and ref

- **WHEN** one navigation binding或one tail slot owns multiple comments
- **THEN** all tokens retain source order in one bundle and use one direct-comment或tail ref
- **THEN** no individual comment token creates an independent key、index、offset identity或ref
- **WHEN** the same bundle contains `// first` and a multiline `/* second<LF> line */`
- **THEN** its untruncated outline summary is exactly `first; second line` and contains no line break

#### Scenario: Direct与tail views可在同一container共存

- **WHEN** container `/options`自身拥有direct comments，且其最后一个child后另有independent-tail bundle
- **THEN** logical `/options` entry使用 `json:comments:#/options`
- **THEN**其subtree末尾的virtual entry使用 `json:tail-comments:#/options`
- **THEN**两条refs分别读取各自bundle而不互相合并

#### Scenario: Empty comment body still creates the comment view

- **WHEN** a direct target或tail slot owns only `//` or `/* */` comments whose derived bodies are empty
- **THEN** outline uses the applicable direct-comment或tail ref but omits `summary`
- **THEN** comment read still includes the raw comment token before the selected logical value
