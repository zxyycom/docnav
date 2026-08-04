本临时 design 说明如何用一个 source-aware JSON model 交付完整 JSONC comment-aware navigation；observable target 由 delta spec 拥有，本文只拥有实现选择、理由和风险。

## Decision Status

用户已确认本 change 必须把 JSONC syntax、direct-comment navigation、tail-comment ref和保留ancestor context的private selection chain作为一个结果交付。Delta spec记录完整observable target；Decision 1–11记录支撑该target的private model与process boundaries。

Observable product contract已经闭合：tail view使用`json:tail-comments:#<fragment>`及固定virtual-entry package，root internal tail与document tail合并为一个root bundle。Exact parser/dependency另须由tasks 0的bounded implementation audit从Decision 9候选中选择；完成全部task 0前不得修改production、owner docs、main specs、tests、schemas/examples、dependencies或release artifacts。

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

有direct comments的root/member/index logical entry使用direct-comment ref，并在normalized comment body非空时填充`Entry.summary`。Root object/array只有拥有root direct comments时才在descendants前新增`<root>` entry；无root direct comments时保持Current不返回root logical entry的行为。

每个非空tail-comment bundle生成一个virtual navigation entry，其固定facts是label `<tail comments>`、kind `tail_comments`、canonical tail ref与optional normalized `Entry.summary`；其它optional entry fields省略。Shared `Entry.kind`是adapter提供的非空字符串，因此该new kind不改变protocol schema shape。在expanded navigation tree中，tail entry是tail anchor的最后一个child：nested tail entry位于该container全部logical descendants之后，root tail entry位于全部root entries之后。Root container不因只有tail bundle而新增`<root>` logical entry；tail entry不进入logical JSON tree、node count或JSON Pointer tokens。

JSON entry pagination把 summary纳入 Unicode character budget；预算不足时先缩减或省略 summary，再沿用 Current label fallback，ref始终完整。

替代方案只在 ref 中放 marker；它虽然可继续 read，但 outline 消费者无法在不理解私有 spelling 的情况下看到注释信息。新增 metadata/shared field 又增加不必要的 protocol ownership，因此复用 summary。

### Decision 6: Comment-bearing read 是完整 JSONC projection

Direct-comment read按source order输出selected binding的exact raw comment tokens，每个token后加LF，再输出与base read相同的normalized strict-JSON value。Tail read以相同规则输出selected tail-comment bundle，再输出tail-anchor value的normalized strict-JSON serialization。两者都是完整`application/jsonc` document，而不是孤立member fragment或comment-only片段；它们不复制其它direct/tail、ancestor、descendant或sibling comments，也不恢复trailing comma。

这使现有 `ReadResult` 足以表达结果并保持 value 可被 JSONC parser 读取。直接返回 member source slice 会带入 comma/container context，未必构成完整 document；同时返回 normalized JSON 与 structured annotations 则要求扩展 shared protocol，二者都不采用。Source fidelity 仍由 full-read 提供，direct-comment/tail read明确是 projection。

这一决定只固定本 change 的 Current-target raw `ReadResult`，不把“comment + value”固化成 private selection model 的唯一渲染能力。Key、ancestor values与各级 comments仍由 Decision 11 的 selection chain保留；未来经过独立批准的 JSON-specific/custom renderer可以从同一 semantic context选择 member fragment、breadcrumb或其它表示，而无需 ref resolver预先裁剪信息。本 change 不新增该 renderer或 renderer API。

### Decision 7: Find 只在完整 comment occurrence 上切换 view

Literal search继续以BOM-stripped original source为语料。Occurrence完全位于direct-comment span内时返回该binding的direct-comment ref，完全位于tail-comment span内时返回tail-anchor ref；其它token与cross-boundary occurrence沿用Current deepest-covering region并返回base ref。这样所有独立comment occurrence都能进入相应read，又不会把普通logical-value query隐式切换为comment view。

### Decision 8: 一个 primary tree 加有界 comment indexes

最终model保留现有ordered `JsonNode` / `JsonMember` / array-element semantics，并为source comments保存ordered non-overlapping half-open byte spans、raw token kind、binding-local direct bundles与anchor-local tail bundles。Attribution在parse/model construction后至多执行一次线性pass；outline/read/find使用binding/anchor-local slices或按offset排序的lookup，不为每个entry/occurrence扫描完整comment set。Drop与maximum-depth paths不得递归处理未受控的第二棵AST。

每个navigation binding至多拥有一个optional direct-comment bundle：`None`唯一表示没有direct comment；`Some`必须包含至少一个source-ordered comment index，同时保存允许为空字符串的normalized body。多条comments共享一个bundle和一个direct-comment ref；`//`、`/**/`或只含空白body的真实comment不会与absent混淆。Raw read始终从comment indexes对应的source spans取得token，不能从normalized body反向重建。

每个canonical tail anchor至多拥有一个optional tail-comment bundle；`None`与`Some`语义和direct bundle相同。一个bundle是source-ordered comment-index sequence，不要求对应一个连续source span；因此root internal tail与document tail可以合并，同时保留每个token的exact span。Empty-container comments归container自身direct binding，不因没有child进入tail slot。同一comment不得同时出现在direct与tail bundle，也不得属于两个anchors。多条tail tokens共享一个path-anchored tail ref；不为每条token或source offset生成identity。

若 chosen parser 自己产生 AST/CST，adapter 必须在 load boundary 把需要的 facts投影到这个 private model，并及时释放 dependency tree；dependency types不能进入 operation、protocol 或 ref types。

### Decision 9: Parser audit 只比较能完成完整 contract 的最小候选

Task 0 只比较以下现实候选，不再把相邻 JSON profiles 当作本 change 的交付范围：

| Candidate | 必须证明的决定性条件 |
| --- | --- |
| Current `serde_json` + offset-preserving JSONC scanner/neutralized view | 一个 scanner 同时验证闭合 grammar、记录 spans、保留 byte offsets，并让 existing seed/cursor model继续保持 raw number、duplicate、depth 与 serializer behavior；custom scanner 的维护和 hostile-input safety 可接受 |
| JSONC parser with AST/token/comment ranges | 所有 broader defaults 可关闭；source order、raw token、decoded duplicate、depth、exact spans和 first-closer comment semantics 可投影到一个 tree，且 dependency/license/toolchain/size 代价低于 custom scanner |
| Serde-compatible lenient parser + bounded comment scanner | 只有在它比前两者减少总维护面、且 parser 与 scanner 不形成两套冲突 grammar source 时才可选 |

Audit 必须记录 exact crate/version/features 或 exact custom modules、contract corpus 结果、dependency/license/advisory facts、workspace target compatibility 与代表性 size/latency delta。它只需要回答“哪个实现最小且正确”，不建立 JSON-LD、GeoJSON、JSON5、NDJSON、CBOR/BSON 等产品矩阵。Strict-profile JSON 可作为 generic regression sample；broader/multi-root syntax只作为负例。若没有候选满足完整contract，audit必须报告no-fit并停止在task 0，不能选择“最接近”的候选、暗中缩减comment context或扩大grammar；后续必须先修订design/delta并取得所需决定。

### Decision 10: Source 与 normalized content type 必须分开

Descriptor 声明 `application/json` 与 `application/jsonc`。Info/full-read 按 source 是否实际含 comment/trailing comma选择二者；base read 始终是 normalized `application/json`；direct-comment与tail read因包含 comments始终是 `application/jsonc`。String 内 marker 不改变 source type。Format id 始终为 `json`。

### Decision 11: Ref resolution产生selected-first borrowed selection chain

Ref syntax parsing与document resolution是两个private阶段。第一阶段只恢复view与canonical JSON Pointer tokens；第二阶段在同一个`JsonDocument`上解析tokens并产生selected-first、随后direct parent、直至document root的borrowed frame chain。JSON只有唯一parent，因此该结构是有界线性chain，不是第二棵tree；frame不能clone ancestor subtree或提前序列化value。

下表定义必须保留的semantic shape；字段名和具体Rust container可以在实现audit中调整，但不得丢失或合并这些facts。

| Component | Required facts and invariants |
| --- | --- |
| Parsed ref | `view: Base | DirectComments | TailComments` 与 canonical pointer tokens；syntax parse不读取document |
| Resolved selection | 原始`view`与non-empty `frames`；`frames[0]`是selected binding或tail anchor，后续frames依次是parent直到root |
| Selection frame | `binding`、borrowed logical `value`、`direct_comments: Option<CommentBundle>`、`tail_comments: Option<CommentBundle>` |
| Binding | `Root`、`ObjectMember { decoded_key }` 或 `ArrayElement { canonical_index }`；不得用nullable string混合root、index与真实空字符串key |
| Comment bundle | Non-empty source-ordered comment indexes、exact raw-token spans与`normalized_body: String`；`None`表示absent，`Some { normalized_body: "" }`表示present-empty |

Resolver先验证首frame是否满足所选view：`Base`只要求path存在，`DirectComments`要求direct bundle存在，`TailComments`要求tail bundle存在；后两者缺失时返回`REF_NOT_FOUND`。Current read只投影首frame与requested view，但selection consumer拥有实际projection控制权，未来可以沿chain读取parent key/index、value、direct comments与tail comments。Resolver不得删除ancestor facts，也不得预先把direct/tail或多个层级comments合并成一个bundle；当前selected-only projection不是private selection contract的context上限。

## Protocol and Process-Boundary Effects

- `docnav-json` 独占 grammar、attribution、ref generation/parsing、summary、read projection 与 find mapping。
- Navigation/core 继续只负责 pathname/explicit selection、closed input、dispatch、pagination defaults 与 opaque ref pass-through，不读取 document 来选 dialect。
- Raw protocol shape、schema field set 和 generic readable renderer 不变；implementation 只需增加使用既有 `summary`、ref 和 content type values 的 contract examples/tests。
- `expand-json-adapter-pathname-hints` 必须等待本 change 成为 Current，再从 then-Current descriptor requirement 重建其 hint-only delta；它不拥有 grammar 或 comments。

## Risks / Trade-offs

- **[Risk] Human intent 与 placement rule 不一致。** → Contract 优先可重现；direct-binding、empty-container-self与tail-anchor rules可测试，用户可通过移动comment到documented slot明确选择direct或tail view。
- **[Risk] Comment-only edit 使 comment ref 失效。** → Base ref 始终稳定可用；direct-comment与tail ref明确是 source-dependent views，stale/no-comment结果固定为 `REF_NOT_FOUND`。
- **[Risk] Comment projection 移动或重排原 comments。** → Read contract明确只保留 raw tokens与 source order，不承诺原 whitespace/slot；需要原布局时使用 full-read。
- **[Risk] Parser default 悄悄接受更宽 grammar。** → Exact options与 negative corpus是 task 0 和 semantic tests 的 blocking evidence，dependency error/token type保持 private。
- **[Risk] Large comments 放大 summary、read 和 find work。** → Ordered target/anchor-local indexes、entry budget、text pagination 和 large-input measurements禁止 per-item full scan。
- **[Risk] Resolver 过早替 renderer裁剪或聚合 context。** → Chain 的每个 root/member/index frame保留本层 direct与tail bundles；Current read只选择首 frame与请求 view，但 resolver不删除其它 facts，未来 renderer policy也不得通过重新解析 ref来补回上下文。
- **[Trade-off] Tail ref锚定logical path而不是comment offset。** → 同一tail slot内增删、合并多条tokens仍使用一个ref；tail bundle被删除或移到另一anchor时旧ref返回`REF_NOT_FOUND`，避免把不稳定byte offset写入public identity。

## Migration Plan

1. 先完成tasks 0的artifact、parser/model与bounded doubt-driven audit，固定exact implementation；审计前不修改change目录外的产品材料。
2. 按 then-Current test policy恢复 Case tree，先加入 strict regression 与 JSONC attribution/ref/read/find failing evidence。
3. 同步 JSON adapter owner/main spec及必要 schema-valid examples为 Target，再实现 descriptor、parser/model、attribution 和 operation vertical slice。
4. 运行 adapter/core/output/release 验证和 workspace verifier，分别检查 protocol-json 与 readable-view。
5. 归档时把 delta 同步为 Current，并让 downstream pathname-hint change从新基线重建其 descriptor delta。

Rollback removes `.jsonc`/`application/jsonc` descriptor facts、JSONC grammar、attribution和 direct-comment/tail ref generation while retaining base strict-JSON navigation。Downgrade 会让 previously accepted JSONC documents失败，因此 release notes必须明确该 incompatibility，不能宣称 transparent rollback。
