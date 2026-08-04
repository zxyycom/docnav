# add-jsonc-comment-aware-navigation

本目录规划一个完整 JSONC 用户结果：`docnav-json` 接受闭合 JSONC grammar，并把 direct comments 与独立 tail comments 都通过 `outline -> ref -> read` 交给读取者。语法接入、comment attribution、ref 定位和 read 是同一个 vertical slice；本 change 不发布 syntax-only 中间状态。

## Scope at a Glance

| Surface | Target |
| --- | --- |
| Grammar | Strict JSON + comments + non-empty-container trailing comma，所有 selected paths 使用同一 grammar |
| Logical model | 一个 ordered JSON tree，加 direct-comment 与 tail-comment bundles；selected-first selection chain保留每一级 binding/value/direct-comments/tail-comments context |
| Base ref | `json:#<fragment>`；read 继续返回 normalized `application/json` |
| Direct-comment ref | `json:comments:#<fragment>`；为有 direct comments 的 root、object member 或 array element 生成 |
| Tail ref | `json:tail-comments:#<fragment>`；fragment锚定拥有 tail slot 的 logical value |
| Outline | Direct comments复用 logical entry/`summary`；tail bundle生成 anchor subtree 的末位 virtual entry，不增加 shared protocol field |
| Comment read | Exact selected comment tokens + normalized anchor value，返回完整 `application/jsonc` document |
| Find | Direct-comment occurrence进入 direct-comment ref，tail-comment occurrence进入 tail ref，其它 occurrence保持 positional base-ref mapping |

其它 JSON-family pathname hints由下游 `expand-json-adapter-pathname-hints` 单独拥有；本 change 不拥有相邻 JSON-family profile。

## Canonical Vocabulary

| Term | Meaning in this change |
| --- | --- |
| Navigation binding | `Root`、`ObjectMember(decoded_key)` 或 `ArrayElement(canonical_index)`；它给 logical value 提供 navigation identity，不等同于 `JsonNode` value 本身 |
| Direct-comment bundle | 唯一归属于一个 navigation binding 的一个或多个 source comments；不存在为 `None`，存在但按需派生的 summary 为空仍是 `Some` |
| Tail anchor | 拥有 independent tail slot 的 root 或 non-empty container logical value；tail ref 复用该 value 的 canonical path，不创建 key、index 或 logical node |
| Tail-comment bundle | 唯一归属于一个 tail anchor 的 source-ordered comments；一个 anchor 至多一个 bundle，多条 comments 共享一个 ref |
| Comment view | `DirectComments` 或 `TailComments`；它只选择 read projection，不改变 base logical identity |
| Selection chain | Adapter-private、selected-first 的 borrowed frame chain；每个 frame保留 binding、value、direct bundle与tail bundle，调用方看不到该结构 |

## Implementation Readiness

- Proposal、design、delta spec 与 tasks 完整描述 Target；在实施、验收和归档前，Current 行为仍以长期 owner、代码、测试和 release evidence 为准。
- Task 0.1–0.7 的实施前审计已闭合：attribution、三种 ref view、empty-container、root-tail grouping、selection chain 和 public output boundary 无未决产品决策。
- Parser/model 已选择“adapter-private offset-preserving JSONC scanner + Current `serde_json` seed/model”；不新增 crate，不修改 workspace dependency graph。候选 spike、反证、代价与回滚证据由 [`design.md`](design.md#implementation-audit) 统一记录。
- 下一个允许执行的步骤是 [`tasks.md`](tasks.md) 1.1：先恢复 Current 测试策略和 Case 映射闭合，再修改任何测试或产品代码。

## Reading Order and Ownership

1. [`proposal.md`](proposal.md) 定义问题、完整用户结果、scope 与 impact。
2. [`specs/json-adapter/spec.md`](specs/json-adapter/spec.md) 是唯一完整 observable target owner，包括 attribution、ref/view、read 和 find contracts。
3. [`design.md`](design.md) 解释 implementation choices、alternatives、process boundaries 与 risks；它不重复取代 delta contract。
4. [`tasks.md`](tasks.md) 是执行顺序 owner；已完成的 task 0 由 design 审计证据支撑，继续实施时从 task 1.1 开始。

长期 Current behavior 在实施与归档前仍由 `docs/navigation.md` 指向的 owners、main specs、code、tests 和 release evidence 决定。本目录不得把 Target 写成当前二进制已经支持。
