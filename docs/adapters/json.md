# JSON Adapter

本文是 `docnav-json` 的长期 JSON adapter owner。共享 adapter interface、protocol
result shape、opaque ref 的原样传递和输出编排分别由
[适配器契约](../adapter-contract.md)、[原始协议](../protocol.md)、
[Ref](../ref-contract.md) 和[输出模式](../output.md)拥有；本文只拥有 JSON 的
格式识别、grammar、私有 source model、ref grammar、导航、JSON-owned diagnostics
和验证边界。

## Current 基线与 pathname-hint contract

本节定义 `docnav-json` 的 Current pathname-hint contract。Current descriptor 以单一
`json` identity 提供 JSONC-capable generic structural navigation，其精确有序集合为：

| 字段 | Current 有序值 |
| --- | --- |
| `extensions[]` | `.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` |
| `filenames[]` | `.prettierrc`、`.watchmanconfig`、`Pipfile.lock`、`deno.lock` |
| `content_types[]` | `application/json`、`application/jsonc` |

代码、tests、core CLI smoke 与 release-package smoke 是上述 Current contract 的实现证据，
不构成第二份 pathname allowlist owner。

`.json`、`.code-workspace`、`.jsonc`、`.prettierrc`、`.watchmanconfig` 与两个
content types 是既有 JSONC 基线；其余七个 suffixes 与两个 exact filenames 是扩展后的
Current routing hints。任一 hint 命中后都只选择既有 `docnav-json`，不把 matched hint、
profile 或 content-type 推断加入 strategy input。Selected operation 继续使用下文的同一
JSONC-capable grammar、generic navigation、ref、output 与 diagnostic 契约；hint 命中不
承诺 JSON-LD、GeoJSON、HAR、Web Manifest、Notebook、SARIF 或 lockfile 的 profile
validity，selected failure 也不会重新路由或 fallback。

## Current：交付与公共边界

Current descriptor 作为 core static registry 中的 linked adapter，由 package 内同一个
`docnav` executable 交付，executable set 精确为该单元素集合。它的 `extensions[]`、
`filenames[]` 与 `content_types[]` MUST 精确匹配上一节的 Current 有序集合；不得声明其它
JSON-family pathname hint、adapter identity 或 format identity。format id 始终为 `json`。

Current fixed strategy 只提供 outline、read、find 和 info（以及既有 unstructured
full-read content/cost capability），不引入 routing probe。每个 operation 只消费其 closed
standard input；matched pathname、suffix、source content type 和 format identity 都不进入
strategy input。JSONC 与 pathname hints 不得增加 core parameter、`StandardInputBinding`、
CLI、env、config 或 protocol input，且 adapter-private 安全上限仍由单一硬编码配置源拥有。

所有由 pathname hint 或 explicit adapter intent 选中的 JSON documents MUST 使用同一套
JSONC-capable grammar；pathname 和 descriptor content type 不选择 strict/JSONC dialect。
共享 ref owner 继续把任何 non-empty ref 当 opaque string 原样传给 selected adapter。

## Current：grammar 与私有 source model

每次 selected operation MUST 从实际 document view 移除至多一个开头 UTF-8 BOM、解码
UTF-8，并恰好解析一个 root value；其后只能是 grammar trivia。root depth 为 `0`，最大
深度为 `127`，同一 object 的 decoded member name 必须唯一。strict JSON value、string
和 number grammar 保持不变；number 保留 strict-JSON source token，object 保持 source
member order，array 保持 index order。

在 string 外，comments 只允许出现 strict JSON 原先允许 whitespace 的位置。Grammar
trivia 只包括 strict-JSON SP/HTAB/LF/CR、`//` line comment 或 `/* ... */` block comment：
line comment 在 LF、CRLF、lone CR 或 EOF 前结束；block comment 在第一个 `*/` 结束且
不嵌套。非空 object/array 的 member/element 后最多可有一个 trailing comma。`{,}`、
`[,]`、缺失或 doubled comma 仍无效；single quote、unquoted name、hex/leading-plus
number、`NaN`、infinity、JSON5 extension、multiple roots 与其它更宽 JSON-family syntax
均必须拒绝。

Comments、trailing commas 与 trivia 不形成 logical JSON node、logical child entry、pointer
token 或 JSON value kind。一个 primary ordered source-aware logical tree MUST 同时服务
traversal、ref resolution、read、info 和 source-occurrence mapping；它保存 BOM-stripped
source、original byte size、logical tree/kind/depth/node count、member/element/value regions、
raw number tokens、source order 和所需索引。ordered comment spans、direct bundles 和 tail
bundles可补充此树；不得创建第二棵完整 logical tree，parser/CST types、attachment heuristic
及其 messages 均保持私有。

## Current：JSON ref grammar 与三种 view

JSON adapter MUST 生成和解析三个非空 ASCII-safe ref view：

```text
json:#<fragment>
json:comments:#<fragment>
json:tail-comments:#<fragment>
```

三种 ref 使用同一个 RFC 6901 URI-fragment pointer 并锚定同一 logical JSON path。base
view 选择 logical value；`comments:` 选择该 navigation binding 的 direct-comment bundle；
`tail-comments:` 选择以该 logical value 为 anchor 的 tail-comment bundle。它们不是
logical node、pointer token、format identity 或按 source offset 持久化的 comment identity。
root refs 分别为 `json:#`、`json:comments:#`、`json:tail-comments:#`。

Object token MUST 先把 `~`/`/` 转为 `~0`/`~1`，再以 UTF-8 和大写 hexadecimal percent
escape 编码 URI fragment 禁止的 byte，且生成 ref 不含 raw NUL/control character。空 key
三种 refs 分别为 `json:#/`、`json:comments:#/`、`json:tail-comments:#/`。Object token 按
decoded member name 解释；array token 只能是 `0` 或无 leading zero 的 decimal index，`-`
不可读取。同一 token 在 object 可为 member name，在 array 则按 index grammar 检查。

Outline 对拥有至少一个 direct comment 的 root/member/index 生成 direct-comment ref；对
每个 non-empty tail bundle 生成 tail-comment ref；base ref 始终继续可读。prefix 缺失或未知、
non-root fragment 无 `/`、percent/`~` escape 非法或非 canonical、array token 非 canonical
时 MUST 返回 `REF_INVALID`。anchor path canonical 但不存在，或 path 存在但所选 direct/tail
bundle 不存在时 MUST 返回 `REF_NOT_FOUND`。文档变更后 ref 不承诺跨版本身份：base path
可指向新 value，stale comment-view ref 可返回 `REF_NOT_FOUND`。

## Current：comment attribution

Closed grammar 成功后，每个 source comment MUST 恰好归于 logical root、一个 object
member、一个 array element 或一个 `Tail(tail_anchor)` slot。root、member、element 以 root
selector、decoded key、canonical index 作为 direct navigation binding；nested container 的
tail anchor 为其 canonical path，complete root 后 document tail 的 anchor 为 root path
（包括 root scalar）。每个 canonical tail anchor 最多一个 tail slot；root container closing
前的 internal tail 与 document tail 合并到同一 source-ordered root bundle。lexical line 在
LF、CRLF、lone CR 或 EOF 结束；此规则不改变 Current LF-counted `find.location`。

Attribution MUST 按 source token、region 和 lexical-line placement 而非 parser 的
previous/next attachment 决定：

1. Root value 前的 comments 归 root direct；root complete token 或 container closing token
   后、从同一 lexical line 开始的 comments 也归 root direct。独立后续 line 开始的
   document-tail comment 归 `Tail(root)`。
2. 其它 comment 先在最深 enclosing object/array context 中判断，nested value 的 comment
   不得归 ancestor binding。
3. Object member name 后至 value 前的 header trivia，以及 complete member value 后至
   separator comma 前的 suffix trivia，归当前 member。opening token/previous comma 后至
   next member name 前的 comment 归 next member；若有 previous member 且 comment 与其
   value 或 comma 同行，则归 previous member。last member（或 optional trailing comma）
   后至 closing token 前的 comment：同行归 last member，独立后续 line 归该 object tail。
4. Array 以相同规则处理 index：element 后至 comma 前归当前 index；opening/previous comma
   后至 next element 前归 next index，除非与 previous element/value comma 同行；last element
   或 trailing comma 后至 closing token 前，同行归 last index，否则归该 array tail。
5. Empty object/array 内 container-only comments 归 container value 自身的 direct binding：
   nested container 用 parent key/index，root empty container 用 root selector；它们不创建
   tail slot。complete empty root 后的 document tail 仍归 `Tail(root)`。

每个 navigation binding 与 tail anchor 各自最多一个 optional bundle。`None` 仅表示无
comments；`Some` 含至少一个 source-ordered comment index 和每 token 精确的 BOM-stripped
half-open UTF-8 byte span。span 不要求连续，root merged tail 仍按 offset 排序；一条 comment
不得进入多个 direct/tail bundles。line token 包含 `//` body 不含 terminator；block token
包含 `/*` 至第一个 `*/`。Outline summary 仅在派生时移除 delimiters、把每 body 的 Unicode
whitespace run collapse 成单个 ASCII space、trim、丢弃空 body，以 `; ` join；不得含 CR/LF。
空 summary 不删除 bundle 或 comment view，只省略 `Entry.summary`，且 bundle 不缓存第二份
完整 normalized comment text。

## Current：outline

Outline MUST 对 expanded navigation tree 做 depth-first preorder。logical object member 与
array element 形成 ordinary entry：完整 ref、非空 label、`object|array|string|number|boolean|null`
kind；object member 依 source order，array 依 index order。empty object key 的 label 为 `""`，
array label 为 `[<index>]`。任何 non-empty tail bundle 则在 anchor subtree 的所有 logical
descendants 后形成一个 virtual child entry：`label: "<tail comments>"`、`kind: "tail_comments"`
及 canonical tail ref。它不进入 logical tree、node count 或 pointer grammar；root tail entry
在所有 root entries 后。

拥有 direct bundle 的 logical root/member/index entry MUST 使用 direct-comment ref；其它
logical entry 用 base ref。non-empty derived summary 通过现有 optional `summary` field 返回。
不返回 source `location`、JSON-specific `metadata` 或其它 JSON raw field。tail virtual entry
精确只含 `{ref, label, kind}` 和 optional `summary`，省略 `location`、`metadata`、`excerpt`、
`rank`、entry-level `cost`。

Root object/array 仅在有 root direct bundle 时，才在 descendants 前增加唯一 `<root>` logical
entry；无 root direct bundle 时继续没有 root-container entry。无 root direct bundle 但有 root
tail bundle 时只增加 tail virtual entry。无 direct、descendant、tail 的 empty object/array
返回 empty entries、null page。Root scalar 始终有 `<root>` entry，按 direct bundle 存在与否
使用 direct-comment/base ref，随后（若存在）返回 root tail entry。

既有 limit/page 契约保持 preorder 和 forward progress：完整 ref 必须保留，预算不足先在
Unicode scalar boundary 截断（以 `...` 标识）或省略 optional summary，随后才用既有 label
截断；无法保留正常 label 内容时为 `.`，但不得替代 empty-key 正常 label `""`。所有 ref
都能原样传给 `read`。

## Current：read 与 find

Read MUST 由 ref 的 selected view 返回确定性 content，并保留输入 ref、对分页前完整 content
计算 cost、按既有 Unicode-safe text pagination 返回 content/page。base view 把 selected logical
value 序列化为 strict JSON：object source order、two-space container layout、raw strict number
token，以及 pinned serializer 的 string/scalar spelling/terminal newline；不包含 comments 或
trailing commas，content type 为 `application/json`。

Direct-comment view 仅按 source order 输出 selected navigation binding 的完整 raw comment
tokens；tail view 仅输出 selected tail slot 的 tokens；两者随后输出同一 strict-JSON
serialization。每个 token 后插入 LF `0x0A`，line token 本身不含 terminator。两种 projection
均为完整 valid JSONC document，不能混入 ancestor/descendant/sibling 或其它 direct/tail
comments，也不恢复 source trailing comma；content type 为 `application/jsonc`。Generic
`readable-view` 仅从同一 raw result 派生既有 header 与 `/content` block，不重读 document、
解析 ref 或重建 comments。

Find query 长度至少为 1；它在 BOM-stripped original source 上做 case-sensitive、left-to-right、
non-overlapping literal search，包括 comments、trailing commas、original string/number spelling。
Canonical ref 与 read serialization 不扩充语料。完全位于 direct-comment span 的 occurrence
返回 direct-comment ref；完全位于 tail span 的 occurrence 返回 tail ref。其它 occurrence
（包括 member name/value、ordinary whitespace、trailing comma、string marker，或跨 comment
boundary/region）按 deepest-covering source region 返回 base ref；跨 child regions 归最近
covering container。每个 occurrence 形成一个按 source offset 排序的 `kind: "match"` entry，
带完整 ref、source-derived non-empty bounded excerpt label、source line location；同 ref 的
occurrences 不合并，entry pagination 保持 order/forward progress。label construction 的
state/context scan work 受 label budget 约束，不随 line 或连续 whitespace 长度增长。

## Current：info、full-read、安全与 diagnostics

Info MUST 返回 source-derived content type、UTF-8、含 optional BOM 的 original byte size、
adapter id `docnav-json`、format id `json`，以及精确 `{root_kind, node_count, max_depth}`
metadata（root included，root depth 0）。source 实际含 accepted comment 或 trailing comma
则 content type 为 `application/jsonc`；否则为 `application/json`；string 内 `//` / `/* */`
markers 不触发 JSONC type。

Unstructured full-read MUST 返回 BOM-stripped original source，不删除 comments、trailing
commas、whitespace、escapes 或 number spelling；使用与 info 相同 source-derived content type
和实际 returned source 的 lines/bytes/tokens cost。它只补充既有 unstructured full-read
content/cost facts，不增加 entries/ref/page/continuation/readable-only wrapper。

JSON-owned failures MUST 保持 stable mapping：missing/path-access failures 用既有
`DOCUMENT_NOT_FOUND` / `DOCUMENT_PATH_INVALID`；invalid UTF-8 用
`DOCUMENT_ENCODING_UNSUPPORTED`；malformed JSON/JSONC、unterminated comment 或 rejected
leniency 用 `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`；complete root 后 non-trivia 或
second root 用 `DOCUMENT_CONTENT_INVALID / JSON_TRAILING_INPUT`；duplicate decoded name 与
depth overflow 分别为 `JSON_DUPLICATE_MEMBER`、`JSON_MAXIMUM_DEPTH_EXCEEDED`；malformed
view/pointer 为 `REF_INVALID`，canonical but missing anchor/bundle 为 `REF_NOT_FOUND`。
`DOCUMENT_CONTENT_INVALID.details` 仅有 normalized `path` 与 stable `reason`；parser
message/type、unstable offset、duplicate name、dependency trace 和 recovery state 私有。selected
failure 不 retry parser mode、routing 或 adapter；operation 以实际打开的 document view 诊断，
不发出已移除的 `json-document-changed-after-probe`。

## Current：验证边界

Owner、adapter tests、Case ledger、coverage mapping、core CLI smoke 与 release package smoke
MUST 分层证明 strict/no-comment behavior不回归；closed JSONC grammar、deterministic
attribution、root/member/index 与 empty-container binding、tail anchor/virtual entry、base/direct/
tail refs/views、comment find-to-read、info/full-read source facts、manifest、source offsets/raw
numbers/duplicates/depth、stable diagnostics、Unicode pagination/cost、generic readable output、
automatic/explicit selection、opaque pass-through、no fallback 与同一 release binary linked
behavior。Large/deep/comment-heavy inputs 还必须证明 comment indexes、summary、attribution,
find 与 drop 的 work/memory bounded，且不对每个 entry/occurrence 全量扫描 comment set。

Pathname-hint evidence 还必须证明完整有序 manifest 集合、九个新增 complete basenames 的
automatic selection、代表性 suffix 与 exact filename 的 `outline -> ref -> read`，以及
grammar-invalid selected input 的 JSON-owned no-fallback diagnostic。Shared schema 与 examples
只约束既有 manifest field shape；本能力只改变 pathname arrays 的 values。

Shared protocol envelope、`Entry`、`ReadResult`、generic `readable-view`、pagination/cost shape
仍由 shared owners 拥有；本 adapter owner 不重新定义它们。格式专用 readable renderer 仍为
独立的 Planned change，不是本 adapter 契约。
