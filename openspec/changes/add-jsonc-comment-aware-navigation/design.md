本 design 说明如何用一个 source-aware JSON model 交付完整 JSONC comment-aware navigation；observable target 由 delta spec 拥有，本文只拥有实现选择、审计证据、理由和风险。

## Decision Status

用户已确认本 change 必须把 JSONC syntax、direct-comment navigation、tail-comment ref 和保留 ancestor context 的 private selection chain 作为一个结果交付。Delta spec 记录完整 observable target；Decision 1–11 记录支撑该 target 的 private model 与 process boundaries。

2026-08-04 的实施前审计已闭合 task 0.1–0.7：tail view 使用 `json:tail-comments:#<fragment>` 及固定 virtual-entry package，root internal tail 与 document tail 合并为一个 root bundle；parser/model 选择 Decision 9 的 offset-preserving scanner 与 Current `serde_json` 复用方案，不新增 dependency。[`Implementation Audit`](#implementation-audit) 是该门禁的证据 owner；下一个执行入口是 task 1.1。

## Context

Current `docnav-json` 由 `serde_json::Deserializer` 驱动一个 adapter-private ordered tree，同时用 `BuildState` cursor 在原文中重建 member/value regions、raw number tokens、duplicate/depth facts 和 BOM-stripped source。Outline、ref resolution、structured read、find、info 与 full-read 都消费这一个 model。Comments 和 trailing commas 会同时影响 parser acceptance 与 cursor/source-region alignment，因此不能只替换一个 parse call 后再单独补 comment semantics。

Shared protocol 已有 optional `Entry.summary`，`ReadResult` 已有 opaque `ref`、`content`、`content_type`、`cost` 与 `page`。Shared ref contract 又明确让 adapter 生成和解析 ref、让 core 原样传递。因此完整目标可以留在 `json-adapter` capability 内完成，不需要扩展 protocol shape 或 renderer API。

## Goals / Non-Goals

**Goals:**

- 让 parser/model 选择由最终 attribution、ref/read 和 source-fidelity 义务驱动，而不是先优化 syntax-only 接入。
- 在一个 primary logical tree 上增加 bounded source-comment evidence、navigation-binding direct-comment bundles 与 tail-anchor bundles。
- 把ref syntax parsing与document resolution分开，并让read消费selected-first borrowed selection chain，而不是重新解析path或复制ancestor values。
- 让 strict/no-comment JSON 的 base navigation 继续使用现有 observable contract。
- 让每个 raw 和 readable result 都能从 adapter-owned facts 唯一推导并按既有 pagination/cost 规则验证。

**Non-Goals:**

- 不创建通用 comment AST、JSON-family abstraction、public dialect enum 或第二套 logical document model。
- 不让 parser attachment、formatter convention 或 schema knowledge决定 attribution。
- 不为每条 comment token创建独立 ref、logical JSON node或 pointer token，也不增加 shared structured annotation fields 或 JSON-specific readable renderer。

## Decisions

### Decision 1: Syntax 与 comment navigation 构成一个 vertical slice

本 change 只有一个交付结果：selected JSONC document 能在保留 logical JSON compatibility 的同时，让 direct comments与independent tail comments完成 `outline -> ref -> read`。Grammar/source model 是该结果的基础任务，不形成独立发布阶段。替代方案是先发布 syntax-only support；它会产生“document 可读但关键 comments 仍从正常导航丢失”的中间 contract，并可能让后续 attribution 重选 parser，因此不采用。

### Decision 2: 一套 grammar 服务所有 selected documents

`.json`、`.jsonc`、`.code-workspace`、exact filename 与 explicit selection 使用同一闭合 JSONC grammar；strict JSON 是子集。Adapter 不接收 pathname、content type 或 dialect input，也不在失败后 retry strict/lenient mode。这样 routing 只选择 adapter，grammar lifecycle 留在 `docnav-json`。

### Decision 3: Attribution 由 syntax placement 与 navigable binding 决定

Delta spec的attribution requirement是完整observable placement owner；本文只定义private结果形状与选择理由。Parser提供ordered tokens、comment spans与root/member/element/container regions，但dependency的previous/next attachment不得直接成为产品归属。Attribution pass必须让每个source comment唯一进入以下一个结果：

- `Direct(Root | ObjectMember(decoded_key) | ArrayElement(canonical_index))`
- `Tail(tail_anchor)`

下表只提供读取索引；具体lexical-line顺序、header/suffix slots和scenarios以delta requirement为准。

| Placement class | Attribution result |
| --- | --- |
| Complete root前的leading comment或complete root后的same-line comment | `Direct(Root)` |
| 独立leading comment位于下一个object member或array element前 | 下一个member/index的`Direct(...)` |
| Comment与前一个complete value或separator comma从同一lexical line开始 | 前一个member/index的`Direct(...)` |
| Empty object/array内部的container-only comments | 该container value的direct binding，包括root empty container |
| Non-empty container最后一个child后的独立后续行comments | `Tail(container_path)` |
| Complete root后的独立后续行comments | `Tail(root_path)` |

Array index与object key承担同等navigation-binding职责；root、array index与真实空字符串object key必须保持不同binding variants。Direct与tail bundles不得重复持有同一token。当root container同时存在最后一个child后的internal tail与complete root后的document tail时，两组comments合并为一个source-ordered、可非连续的root tail bundle，共享`json:tail-comments:#`、一个summary与一次read。

### Decision 4: Ref view不改变logical identity

Base ref保持 `json:#<fragment>`；direct-comment view使用 `json:comments:#<fragment>`；tail view使用 `json:tail-comments:#<fragment>`。三种view共享同一canonical JSON Pointer path：base和direct-comment view选择该navigation binding，tail view选择以该logical value为tail anchor的tail slot。View marker不形成logical node、pointer token、format identity或按offset持久化的comment identity；core/protocol/output继续把ref当opaque strings。

| `/options` selection | Ref |
| --- | --- |
| Logical value | `json:#/options` |
| `/options` binding 的 direct comments | `json:comments:#/options` |
| `/options` container 的 independent tail | `json:tail-comments:#/options` |

只有navigation binding存在direct bundle时direct-comment view才存在，只有tail anchor存在tail bundle时tail view才存在；同一container可以同时拥有两种comment view且refs不冲突。删除或把comment移出对应slot后旧comment ref返回`REF_NOT_FOUND`，而base ref继续按logical path工作。Base ref不会根据当前source自动切换content grammar，因此strict/no-comment JSON compatibility保持稳定。

### Decision 5: Outline 用既有 summary 暴露选择信息

有direct comments的root/member/index logical entry使用direct-comment ref，并在按需派生的normalized summary非空时填充`Entry.summary`。Root object/array只有拥有root direct comments时才在descendants前新增`<root>` entry；无root direct comments时保持Current不返回root logical entry的行为。

每个非空tail-comment bundle生成一个virtual navigation entry，其固定facts是label `<tail comments>`、kind `tail_comments`、canonical tail ref与optional normalized `Entry.summary`；其它optional entry fields省略。Shared `Entry.kind`是adapter提供的非空字符串，因此该new kind不改变protocol schema shape。在expanded navigation tree中，tail entry是tail anchor的最后一个child：nested tail entry位于该container全部logical descendants之后，root tail entry位于全部root entries之后。Root container不因只有tail bundle而新增`<root>` logical entry；tail entry不进入logical JSON tree、node count或JSON Pointer tokens。

JSON entry pagination把 summary纳入 Unicode character budget；预算不足时先缩减或省略 summary，再沿用 Current label fallback，ref始终完整。

Summary 只是选择提示，不是 raw comment 的存储形式。Adapter 按需从 bundle spans 派生单行文本：去掉 comment delimiters，把每个 Unicode whitespace run 折叠为一个 ASCII space，trim 每个 body，丢弃空 body，再用 `; ` 连接多个 body。这保证 `Entry.summary` 不含 CR/LF，不会把 generic readable outline 的一个 entry 拆成多行；exact raw tokens 仍由 comment read 交付。

替代方案只在 ref 中放 marker；它虽然可继续 read，但 outline 消费者无法在不理解私有 spelling 的情况下看到注释信息。新增 metadata/shared field 又增加不必要的 protocol ownership，因此复用 summary。

### Decision 6: Comment-bearing read 是完整 JSONC projection

Direct-comment read按source order输出selected binding的exact raw comment tokens，每个token后加LF，再输出与base read相同的normalized strict-JSON value。Tail read以相同规则输出selected tail-comment bundle，再输出tail-anchor value的normalized strict-JSON serialization。两者都是完整`application/jsonc` document，而不是孤立member fragment或comment-only片段；它们不复制其它direct/tail、ancestor、descendant或sibling comments，也不恢复trailing comma。

这使现有 `ReadResult` 足以表达结果并保持 value 可被 JSONC parser 读取。直接返回 member source slice 会带入 comma/container context，未必构成完整 document；同时返回 normalized JSON 与 structured annotations 则要求扩展 shared protocol，二者都不采用。Source fidelity 仍由 full-read 提供，direct-comment/tail read明确是 projection。

这一决定只固定本 change 的 target raw `ReadResult`，不把“comment + value”固化成 private selection model 的唯一渲染能力。Key、ancestor values与各级 comments仍由 Decision 11 的 selection chain保留；未来经过独立批准的 JSON-specific/custom renderer可以从同一 semantic context选择 member fragment、breadcrumb或其它表示，而无需 ref resolver预先裁剪信息。本 change 不新增该 renderer或 renderer API。

### Decision 7: Find 只在完整 comment occurrence 上切换 view

Literal search继续以BOM-stripped original source为语料。Occurrence完全位于direct-comment span内时返回该binding的direct-comment ref，完全位于tail-comment span内时返回tail-anchor ref；其它token与cross-boundary occurrence沿用Current deepest-covering region并返回base ref。这样所有独立comment occurrence都能进入相应read，又不会把普通logical-value query隐式切换为comment view。

### Decision 8: 一个 primary tree 加有界 comment indexes

最终model保留现有ordered `JsonNode` / `JsonMember` / array-element semantics，并为source comments保存ordered non-overlapping half-open byte spans、raw token kind、binding-local direct bundles与anchor-local tail bundles。Attribution在parse/model construction后至多执行一次线性pass；outline/read/find使用binding/anchor-local slices或按offset排序的lookup，不为每个entry/occurrence扫描完整comment set。Drop与maximum-depth paths不得递归处理未受控的第二棵AST。

每个navigation binding至多拥有一个optional direct-comment bundle：`None`唯一表示没有direct comment；`Some`必须包含至少一个source-ordered comment index。多条comments共享一个bundle和一个direct-comment ref；`//`、`/**/`或只含空白body的真实comment不会与absent混淆。Raw read 和 normalized summary 都从comment indexes对应的source spans按需派生，bundle不缓存第二份完整comment文本。

每个canonical tail anchor至多拥有一个optional tail-comment bundle；`None`与`Some`语义和direct bundle相同。一个bundle是source-ordered comment-index sequence，不要求对应一个连续source span；因此root internal tail与document tail可以合并，同时保留每个token的exact span。Empty-container comments归container自身direct binding，不因没有child进入tail slot。同一comment不得同时出现在direct与tail bundle，也不得属于两个anchors。多条tail tokens共享一个path-anchored tail ref；不为每条token或source offset生成identity。

选定方案不产生 dependency AST/CST；若未来重开 parser 选择，任何 dependency types 仍必须在 load boundary 投影后释放，不能进入 operation、protocol 或 ref types。

### Decision 9: Offset-preserving scanner 复用 Current `serde_json` model

实现在 `crates/adapters/json` 内增加一个 private `jsonc` lexical module，不新增 crate、feature 或 lockfile entry。Load path 按以下顺序执行：

1. 去除至多一个 UTF-8 BOM 并完成 UTF-8 decode 后，一次线性 scanner 识别 strict-JSON strings、JSON-only whitespace、line/block comments 和 structural commas。它输出 ordered `CommentToken { kind, span }`、attribution 所需的 comma/line-boundary facts 以及 `has_jsonc_syntax`；所有 span 是 BOM-stripped source 的 half-open UTF-8 byte range。
2. Scanner 同时构建一个与 source 字节等长的 temporary parse view：comment 非换行 bytes 与已验证 trailing-comma byte 替换为 ASCII spaces，CR/LF 原样保留。Scanner 只接受 SP、HTAB、LF、CR 作为 grammar whitespace，line comment 在 LF、CRLF、lone CR 或 EOF 结束，block comment 在第一个 `*/` 结束。
3. Current `serde_json 1.0.150` `Deserializer` 及 adapter 现有 `NodeSeed`/`BuildState` 消费 parse view；等长 offset 仍精确指向 original source。因此 ordered tree、decoded names/duplicate rejection、raw number spelling、depth `127/128`、member/value regions 和 normalized serializer 继续只有一个 owner。
4. Tree 建立后执行一次 deterministic attribution pass，生成 binding-local direct indexes 与 anchor-local tail indexes。`JsonDocument` 保留 original source、ordered comment tokens 和这两类 indexes；temporary parse view 及不再需要的 scanner scratch 在 operation 前释放。

该方案只增加一个 `O(source bytes)` 的临时 parse buffer和与 comment/comma 数量成比的 evidence，不建立第二棵 logical tree。Broader syntax 仍由 closed scanner 与 Current strict parser 共同拒绝；scanner 不尝试实现第二套 JSON value parser。

### Decision 10: Source 与 normalized content type 必须分开

Descriptor 声明 `application/json` 与 `application/jsonc`。Info/full-read 按 source 是否实际含 comment/trailing comma选择二者；base read 始终是 normalized `application/json`；direct-comment与tail read因包含 comments始终是 `application/jsonc`。String 内 marker 不改变 source type。Format id 始终为 `json`。

### Decision 11: Ref resolution产生selected-first borrowed selection chain

Ref syntax parsing与document resolution是两个private阶段。第一阶段只恢复view与canonical JSON Pointer tokens；第二阶段在同一个`JsonDocument`上解析tokens并产生selected-first、随后direct parent、直至document root的borrowed frame chain。JSON只有唯一parent，因此该结构是有界线性chain，不是第二棵tree；frame不能clone ancestor subtree或提前序列化value。

下表定义必须保留的semantic shape；字段名和具体Rust container可以在实现中调整，但不得丢失或合并这些facts。

| Component | Required facts and invariants |
| --- | --- |
| Parsed ref | `view: Base | DirectComments | TailComments` 与 canonical pointer tokens；syntax parse不读取document |
| Resolved selection | 原始`view`与non-empty `frames`；`frames[0]`是selected binding或tail anchor，后续frames依次是parent直到root |
| Selection frame | `binding`、borrowed logical `value`、`direct_comments: Option<CommentBundle>`、`tail_comments: Option<CommentBundle>` |
| Binding | `Root`、`ObjectMember { decoded_key }` 或 `ArrayElement { canonical_index }`；不得用nullable string混合root、index与真实空字符串key |
| Comment bundle | Non-empty source-ordered comment indexes与exact raw-token spans；`None`表示absent，`Some`仍可按需派生空 summary，但不缓存第二份 normalized body |

Resolver先验证首frame是否满足所选view：`Base`只要求path存在，`DirectComments`要求direct bundle存在，`TailComments`要求tail bundle存在；后两者缺失时返回`REF_NOT_FOUND`。Target read只投影首frame与requested view，但selection consumer拥有实际projection控制权，未来可以沿chain读取parent key/index、value、direct comments与tail comments。Resolver不得删除ancestor facts，也不得预先把direct/tail或多个层级comments合并成一个bundle；本 change 的selected-only projection不是private selection contract的context上限。

## Implementation Audit

### Current baseline and contract corpus

审计从 `docs/navigation.md` 指向的 Current owners 进入，只把下列已核实事实当作实现基线：

| Surface | Current evidence recovered on 2026-08-04 |
| --- | --- |
| JSON owner | `docs/adapters/json.md` 与 main `openspec/specs/json-adapter/spec.md`；Current grammar 是 strict JSON |
| Adapter/ref/protocol/output boundaries | `docs/adapter-contract.md`、`docs/ref-contract.md`、`docs/protocol.md`、`docs/output.md`；adapter 拥有 ref，core 原样传递，`Entry.summary`/`ReadResult`/generic readable shape 已存在 |
| Parser/model | `serde_json 1.0.150` with `raw_value`/`unbounded_depth`；`JsonDocument` 保留 BOM-stripped source、ordered tree、regions、raw numbers、decoded-duplicate 与 max-depth `127/128` 行为 |
| Current verification | `cargo test -p docnav-json` 的 42 项测试通过；Case/release owners 覆盖 Current Linux/Windows canonical package path |
| Adjacent changes | `expand-json-adapter-pathname-hints` 仍等待本 change 归档后的 JSON grammar/descriptor baseline；其它 JSON-family hints 不进入本 corpus |

Delta spec 是 contract corpus 的唯一 observable owner；审计按下表检查完整性，没有从 parser 能力反向扩大 grammar：

| Corpus dimension | Owner requirements and decisive evidence |
| --- | --- |
| Closed grammar and strict regression | `JSON selected operations 必须验证实际文档`；comments/trailing comma 正例，VT/FF、broader JSON5、missing/doubled comma、multi-root 负例，raw number、decoded duplicate 与 depth `127/128` |
| Attribution | `JSONC comments 必须按syntax placement...`；root/object/array、leading/same-line trailing、header/suffix、empty-container-self、deepest context、tail boundary 与 root-tail merge |
| Navigation identity | Ref/outline requirements；base/direct/tail coexistence、conditional root entry、tail virtual entry/order、single-line summary、pagination 和 stale behavior |
| Read and retained context | Read requirement 与 Decision 11；三种 projection、exact raw tokens、complete JSONC value、selected-first all-frame facts 与 Target selected-frame projection |
| Find/source/diagnostics | Find、info/full-read 与 selected-operation requirements；comment-span mapping、source-derived content type、string-marker negative case、stable error privacy |
| Integration and bounds | Evidence requirement；opaque ref pass-through、unique-ref auto-read、raw/readable parity、Linux/Windows package path、large/deep/comment-heavy bounded work |

### Candidate spike and selection evidence

候选在 workspace 外的隔离 temp crate 中固定版本并使用 workspace Rust `1.96.0` 编译。共 10 个针对性用例覆盖 exact spans、LF/CRLF/lone-CR/EOF line comments、first-closer block comments、trailing comma、broader-syntax rejection、VT/FF rejection、raw/duplicate spelling 与 depth `127/128`，`cargo +1.96.0 test --all-targets` 全部通过。

| Candidate | Contract fit and maintenance result |
| --- | --- |
| **Selected:** Current `serde_json 1.0.150` + private scanner | Scanner spike 产生等长 view 与 exact comment spans，并拒绝全部契约外输入；Current seed 继续拥有 raw number、decoded duplicate、depth、order/region 与 serializer behavior。新增线性 lexical module，dependency delta 为零 |
| [`jsonc-parser 0.33.1`](https://docs.rs/jsonc-parser/0.33.1/jsonc_parser/struct.ParseOptions.html) ([source snapshot](https://github.com/dprint/jsonc-parser/tree/041f112d0dd6ffb7e181a471c2de5a15e9420b69)) | MIT、edition 2024、default dependency tree 无 transitive crate，可提供 AST/token/range/raw-number facts；但非可配置地接受 VT/FF whitespace、不把 lone CR 当 line-comment terminator，且内部 depth ceiling 为 512。仍需自定义 lexical validator、depth wrapper 和 AST projection，因此拒绝 |
| [`serde_json_lenient 0.2.4`](https://docs.rs/serde_json_lenient/0.2.4/serde_json_lenient/struct.Deserializer.html) ([source snapshot](https://github.com/google/serde_json_lenient/tree/111dd4522f5989efe43449e715a96e1aee533894)) | MIT/Apache-2.0、Rust 1.56、Serde-compatible；但不提供 comment spans，lone CR 行注释与契约不符，仍需完整 scanner 与 Current `BuildState` alignment。它只会引入第二 parser crate 及 `itoa`/`memchr`/`ryu`/`serde`，因此拒绝 |

Security/maintenance 记录是 point-in-time evidence，不是对未来版本的保证：`jsonc-parser` snapshot 为 2026-07-26 的 tag `0.33.1`，`serde_json_lenient` snapshot 为 2024-12-28；[RustSec advisory-db snapshot `d91a8fc`](https://github.com/RustSec/advisory-db/tree/d91a8fc9492378f23cba86b81770c6d16de6ebba) 的 package-name 核对没有命中这两个候选及上述 transitives。Selected path 不新增 package、license 或 target compatibility surface，继续使用现有 Linux/Windows 纯 Rust release path。

下表是同一 Linux 审计环境、Rust 1.96.0、100-property 输入下的 representative release micro-spike；三次 1,000-parse 取中值，三次 100 个 one-parse process launches 取中值。数据用于比较候选总量，不是 production performance budget；`serde_json_lenient` 数据尚未包含契约必需的 span scanner。

| Probe binary | Size | Delta vs strict baseline | 1,000 parses | 100 process launches |
| --- | ---: | ---: | ---: | ---: |
| Strict `serde_json` baseline | 587,176 B | — | 11.747 ms | 0.161 s |
| Selected scanner + `serde_json` | 604,352 B | +17,176 B | 15.767 ms | 0.205 s |
| `jsonc-parser` AST + value projection | 684,304 B | +97,128 B | 27.131 ms | 0.174 s |
| `serde_json_lenient` without span scanner | 666,488 B | +79,312 B | 14.350 ms | 0.197 s |

### Bounded challenge and minimality result

| Challenge | Finding and disposition |
| --- | --- |
| Base/direct/tail identity and stale refs | Distinct view markers coexist on one canonical path；comment-view absence maps to `REF_NOT_FOUND`，base behavior 不随 comment 自动改变 |
| Attribution ambiguity and root-tail grouping | Delta 的 lexical-line/deepest-context rules 使每个 token 恰好归属一次；root internal/document tail 可以是一个非连续、source-ordered bundle |
| Summary/readable output | 原先的 LF-joined summary 会把 generic readable entry 拆行；已改为按需派生的单行 `; `-joined summary，raw read 仍保留 exact tokens |
| Large comment memory | 原先缓存 `normalized_body: String` 会复制大段注释；已改为 bundle 只保留 non-empty source indexes/spans，summary/read 按需派生 |
| Selection context and renderer control | Every selected-first frame 保留 binding/value/direct/tail facts；Target read 只投影首 frame，但 resolver 不删除 ancestor context |
| Auto-read and output parity | Unique direct/tail ref 继续由 core 当 opaque ref 发起 existing read；delta evidence 和 tasks 明确要求 protocol-json/readable-view 的 nested content 同源 |
| Grammar defaults and hostile input | Closed scanner 拒绝 non-JSON whitespace/broader syntax，Current seed 保留 depth/duplicate rules；one linear scan、one temporary same-size buffer 和 ordered local indexes 排除 per-entry full scans |
| Source truth, document changes and downgrade | `has_jsonc_syntax` 只由 string-aware scanner facts 派生；operation 使用它实际打开的 document view；rollback 会拒绝已被接受的 JSONC，必须在 release notes 表达 |

最小实现面因此固定为：一棵 primary logical tree、一个 load-time 等长 parse buffer、ordered comment tokens、必要的 binding/anchor indexes、两个 comment ref view markers 和一条 borrowed selected-first selection chain。不引入 recursive parent tree、ancestor value clone、offset-based public identity、public dialect、shared protocol field、renderer parsing、per-item full comment scan 或无当前消费者的 JSON-family abstraction。

审计结束时没有未解决的 product/architecture decision。Spike 只证明选定路径可行，不代替 task 1–5 的 failing tests、production implementation、large/deep bounds 和 release verification。

### AI recovery check

从 README 进入的 reader 应能直接恢复以下答案：

| Question | Canonical answer |
| --- | --- |
| Direct 与 tail 有什么不同？ | Direct 归 navigation binding；tail 归 root（任意 value kind）或 non-empty container 的 independent-tail anchor slot |
| 三种 ref 是三个logical identities 吗？ | 不是；它们共享 canonical path，view marker 只选择 base/direct/tail projection |
| Empty-container comments 如何归属？ | 归 container 自身 direct binding；root 使用 root selector，nested container 使用 parent key/index path |
| Root internal tail 与 document tail 如何表示？ | 按 source order 合并为一个可非连续 root bundle，共享 `json:tail-comments:#` |
| Selection frame 必须保留什么？ | Binding、borrowed value、optional direct bundle 和 optional tail bundle；frames selected-first 直到 root |
| Target read 是 private context 上限吗？ | 不是；Target read 只投影首 frame/view，selection chain 仍保留所有 parent facts |
| 下一个任务是什么？ | Task 1.1：恢复 Current 测试 owner、Case policy 与 wrapper 闭合，然后才写 failing evidence |

## Protocol and Process-Boundary Effects

- `docnav-json` 独占 grammar、attribution、ref generation/parsing、summary、read projection 与 find mapping。
- Navigation/core 继续只负责 pathname/explicit selection、closed input、dispatch、pagination defaults 与 opaque ref pass-through，不读取 document 来选 dialect。
- Raw protocol shape、schema field set 和 generic readable renderer 不变；implementation 只需增加使用既有 `summary`、ref 和 content type values 的 contract examples/tests。
- `expand-json-adapter-pathname-hints` 必须等待本 change 成为 Current，再从 then-Current descriptor requirement 重建其 hint-only delta；它不拥有 grammar 或 comments。

## Risks / Trade-offs

- **[Risk] Human intent 与 placement rule 不一致。** → Contract 优先可重现；direct-binding、empty-container-self与tail-anchor rules可测试，用户可通过移动comment到documented slot明确选择direct或tail view。
- **[Risk] Comment-only edit 使 comment ref 失效。** → Base ref 始终稳定可用；direct-comment与tail ref明确是 source-dependent views，stale/no-comment结果固定为 `REF_NOT_FOUND`。
- **[Risk] Comment projection 移动或重排原 comments。** → Read contract明确只保留 raw tokens与 source order，不承诺原 whitespace/slot；需要原布局时使用 full-read。
- **[Risk] Parser default 悄悄接受更宽 grammar。** → 审计已用 closed scanner 与 negative corpus反证契约外syntax；implementation semantic tests继续固定该边界，dependency error/token type保持 private。
- **[Risk] Large comments 放大 summary、read 和 find work。** → Ordered target/anchor-local indexes、entry budget、text pagination 和 large-input measurements禁止 per-item full scan。
- **[Risk] Resolver 过早替 renderer裁剪或聚合 context。** → Chain 的每个 root/member/index frame保留本层 direct与tail bundles；Target read只选择首 frame与请求 view，但 resolver不删除其它 facts，未来 renderer policy也不得通过重新解析 ref来补回上下文。
- **[Trade-off] Tail ref锚定logical path而不是comment offset。** → 同一tail slot内增删、合并多条tokens仍使用一个ref；tail bundle被删除或移到另一anchor时旧ref返回`REF_NOT_FOUND`，避免把不稳定byte offset写入public identity。

## Migration Plan

1. 实施前 artifact、parser/model 与 bounded doubt-driven audit 已完成，exact implementation 已固定为 Decision 9。
2. 从 task 1.1 开始，按 then-Current test policy恢复 Case tree，先加入 strict regression 与 JSONC attribution/ref/read/find failing evidence。
3. 同步 JSON adapter owner/main spec及必要 schema-valid examples为 Target，再实现 descriptor、parser/model、attribution 和 operation vertical slice。
4. 运行 adapter/core/output/release 验证和 workspace verifier，分别检查 protocol-json 与 readable-view。
5. 归档时把 delta 同步为 Current，并让 downstream pathname-hint change从新基线重建其 descriptor delta。

Rollback removes `.jsonc`/`application/jsonc` descriptor facts、JSONC grammar、attribution和 direct-comment/tail ref generation while retaining base strict-JSON navigation。Downgrade 会让 previously accepted JSONC documents失败，因此 release notes必须明确该 incompatibility，不能宣称 transparent rollback。
