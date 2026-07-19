**一句话核心：用一个 adapter-private JSON document model 接入既有 fixed strategy 和 static registry，并把实际接入摩擦作为后续优化证据；本文是仅位于本 change 目录下的未审核临时设计，不影响现有主规范或其它 change。**

## Context

当前 core registry 通过静态 factory slice 注册 `markdown_adapter_definition`，navigation 选择 `AdapterDefinition` 后把 closed `StandardOperationInput` 分派到固定的 outline/read/find/info strategy。Caller-configurable 参数只由 core catalog 定义；Markdown 的 `max_heading_level` 通过 compile-time binding 进入可选字段，而 adapter definition 不贡献输入声明。

JSON adapter 将成为第二个真实实现。它必须证明新的格式可以沿现有 crate、factory、registry、probe、closed input、protocol result 和 release package 路径落地，而不重开 runtime plugin、generic option bag 或 dynamic registration 设计。`serde_json` 已是 workspace dependency，不需要引入新的 parser package。

## Goals / Non-Goals

**Goals:**

- 导航常见 UTF-8 JSON 文件，并支持自动/显式选择及完整 operation surface。
- 用确定性树遍历和 adapter-owned JSON Pointer refs 支持 `outline/find -> ref -> read`。
- 保持现有 core parameter catalog、closed operation input、protocol 和 output shape。
- 在 package 中通过同一个 `docnav` binary 验证 Markdown 与 JSON。
- 实现后记录真实接入点和摩擦，作为是否需要结构优化的证据。

**Non-Goals:**

- 不增加动态安装、注册、更新、发现协议或独立 adapter executable。
- 不让 JSON adapter 定义 CLI/env/config/protocol 参数；首期没有 JSON-scoped option。
- 不支持 JSON5、JSON Lines、comments、trailing comma、YAML/TOML 或 schema-aware navigation。
- 不保留 JSON 原始 whitespace、number spelling 或 object member 源码顺序于 structured read。
- 不为大型 JSON 建 streaming parser/index、持久缓存或新 pagination abstraction；性能优化需要测量证据。
- 不因第二 adapter 顺手重构 Markdown、shared protocol 或 output renderer。

## Decisions

### Decision 1: JSON 只在 Beta 后的继续决定成立时启动

本 change 可以提前完成设计，但实现必须等待 `ship-markdown-cli-beta` 已公开、公开资产的 Quick Start 已复验，并由用户基于初步使用或反馈明确确认仍值得扩展第二格式。若 Beta 暴露的是核心产品价值不足，而不是格式覆盖不足，则暂停 JSON，不用第二 adapter 掩盖产品问题。

这个门禁不要求虚构用户数量、token 收益或统计显著性；它要求一次明确的产品判断及其可观察依据。JSON 同时具备真实用途和结构差异：Markdown 以 heading/section 导航，JSON 以 object key/array index/value tree 导航。它可以检验 ref、outline、find、read、info、probe、static registry 和 release smoke，而 parser 依赖已经存在，范围比 PDF、HTML 或表格更可控。

替代方案 YAML/TOML 与 JSON 结构接近但需要新增 parser 和格式特有语义；HTML/PDF 更能施压架构，却会把解析复杂度混入这次验证。首个第二样本选择 JSON。

### Decision 2: 新 crate 复制 adapter 形状，不新增 adapter framework

新增 `crates/adapters/json`，导出 `json_adapter_definition()`，core 在现有 static factory slice 中显式加入该 factory。JSON crate 依赖 adapter contracts、protocol、text-cost、serde/serde_json；core 只增加 workspace dependency 和 registry import。

Adapter definition 继续只组合 manifest、strategy 和 optional full-read capability。JSON 不修改 `AdapterDefinition`、`NavigationAdapterRegistry`、`StandardOperationInput`、`StandardInputBinding` 或 parameter catalog。若实现发现必须修改这些 closed contract 才能表达 JSON 的基础 operation，本 change 暂停并重新审视设计，而不是加入 generic escape hatch。

### Decision 3: Adapter 内部使用一次 parse 的 JSON document model

每次 strategy 调用加载 UTF-8 source、去除可选 BOM，并通过 `serde_json::Deserializer` 驱动的 adapter-private decode 层构造 JSON value。Decode 必须消费完整输入并检测同一 object 内的重复 member name；重复 key 会让 JSON Pointer 无法唯一定位，因此作为不支持输入拒绝，而不采用 last-value-wins。

Document model 保存原始 source 和 parsed value，并提供 traversal、ref resolution、normalized serialization、node count 与 depth mechanics。Object traversal 显式排序 key，避免依赖 map feature 或源码 member order；array 保持 index 顺序。

考虑过保留源码 span，以便 structured read 返回原始片段。它需要自定义 span parser，并把格式保存问题与第二 adapter 验证耦合；首期 read 返回规范化 JSON，只有 unstructured full-read 返回原文。

### Decision 4: Ref 采用带 adapter prefix 的 RFC 6901 pointer

内部 `JsonRef` 负责：

- `json:` 表示 root。
- `json:/a/0` 表示 RFC 6901 pointer。
- object token canonical escape `~0`/`~1`。
- array token只接受 canonical 非负十进制，不接受 `-` 或前导零。

Prefix 保证 ref 非空并让错误信息清晰；shared/core 仍不解析它。Grammar 解析与当前 value resolution 分开，因此非法 grammar 映射 `REF_INVALID`，合法 pointer 在当前树中无节点映射 `REF_NOT_FOUND`。Ref 是当前结构路径，不是内容 hash；文档改变后，同一路径可能指向新值或不再存在。

考虑过自定义 field-tagged path grammar。JSON Pointer 已有成熟 escaping 和用户认知，自定义 grammar 没有产品收益。

### Decision 5: Outline 是确定性的 descendant preorder

对有 descendant 的 root object/array，outline 不加入 root entry，依次输出每个 descendant；这样常见单成员 JSON 仍能产生唯一 ref 并复用现有 auto-read。Root scalar、空 object 或空 array 没有 descendant 时，返回 `json:` fallback。

Entry label 使用最后一个 key/index 加必要 path context，kind 使用 `object|array|string|number|boolean|null`，metadata 只加入确有稳定消费价值的 depth/pointer facts。Location 不伪造源码行号。先构造确定性 entries，再复用现有 pagination helper；超大树的 streaming 留给测量后的独立优化。

### Decision 6: Structured read 规范化，full-read 保留原文

Read resolution 得到一个 value 后，以确定性 pretty JSON 序列化，再复用 Unicode-safe text pagination。Result 保留调用方 ref，content type 为 `application/json`，cost 针对分页前的完整规范化 text。

Unstructured full-read 是不同语义：返回去除可选 BOM 后的原始 text，并声明 `application/json` 与 lines/bytes/tokens cost capability。这样小文档直返不会无意改写用户输入，structured node read 则保持稳定、可读。

### Decision 7: Find 搜索 pointer 与 scalar，不递归序列化 container

Find 按与 outline 相同的 traversal 顺序访问每个 node：

- 每个 node 搜索 canonical pointer。
- scalar node额外搜索规范化 scalar text。
- container 不搜索整个递归序列化内容，避免一个深层文本在所有祖先重复命中。
- 同一 node 最多产生一个 match。

Match ref 指向命中 node，read 能继续取得该 value。若命中来自 pointer，read 不保证 content 内重复出现 path，但 ref 仍精确指向用户查找的节点。Query 使用大小写敏感 literal semantics，避免在本 change 引入 regex、ranking 或 locale policy。

### Decision 8: Info、cost 和 presentation 复用现有边界

Info 只输出稳定摘要：content type、encoding、byte size、adapter/format、root kind、包含 root 的 node count 和 max depth；root depth 定义为 `0`。Outline/find/read 使用既有 `Entry`、`Cost`、`Page` 和 operation result；readable renderer 从这些 facts 派生显示，JSON adapter 不新增 public result field或 readable-only wrapper。

### Decision 9: 验证保持 owner 分层，实施观察单独记录

JSON crate owner tests 证明 parser、重复 key、traversal、refs、read/find/info、pagination 和 errors。Core tests 只证明 static registry/selection 和 closed-input 交接；CLI smoke 证明真实进程路径；release smoke 证明 package 中同一 binary 同时包含 Markdown/JSON。

实现结束后在本 design 追加 `## Implementation Observations`，记录：

- 实际新增/修改的接入点。
- 是否需要 shared contract 或 parameter catalog 变化。
- 哪些代码只是在新 adapter 内实现格式语义。
- 是否出现相同语义的跨 adapter 复制、不可预测修改点或职责绕行。
- 被推迟的问题及其可观察影响。

观察本身不授权同一 change 做预防性重构；需要结构调整时另起有证据的 change。

### Decision 10: Protocol 和 process boundary 保持兼容

JSON 通过现有 document commands、output plans、protocol envelopes、diagnostic projection 和 process exits 暴露。新增 observable behavior 只有 `.json` 可被自动/显式选择、`adapter list` 多一个静态 entry，以及对应 format-owned item/ref/content facts。现有 Markdown 输出和参数解析不得变化。

## Risks / Trade-offs

- [完整 parse 和完整 entry materialization 对超大 JSON 占用内存] → 首期与 Markdown 一样采用内存模型，并用现有输出 limit 控制返回；只有测量显示瓶颈后才设计 streaming。
- [Object key 排序不同于源码展示顺序] → JSON object 语义不依赖顺序，规范明确 deterministic order，full-read 仍保留原文。
- [重复 key 检测需要比普通 `serde_json::from_str` 更多 adapter-private 代码] → 将检测限制在 decode 层并用 owner tests 证明；不把它提升为 shared abstraction。
- [Pointer 命中后 read 内容可能不包含命中的 path 文本] → Match label 展示 pointer，ref 精确指向 node；不为此改变 read 的 value 语义。
- [Array 插入会让旧 ref 指向不同 index] → 文档化 refs 是当前结构路径而非持久身份；不存在的 index 返回 `REF_NOT_FOUND`。
- [第二 adapter 可能暴露 shared contract 摩擦] → 先记录真实阻塞；只有基础 operation 无法表达时暂停 change，不用 generic bag 或动态注册绕过边界。
- [自动 discovery 顺序造成格式争用] → JSON 只有 extension 与完整 parse 同时成功才支持，Markdown 对 `.json` 不声明支持；显式 adapter 仍按既有 declared selection path。

## Migration Plan

1. 先新增 JSON capability owner 文档、fixtures 和 adapter crate owner tests。
2. 实现 adapter-private document/ref/traversal mechanics，再实现固定 strategy 和 definition factory。
3. 在 core static registry 显式加入 JSON，更新 adapter list/doctor、automatic/declared selection 和 CLI smoke。
4. 更新 package smoke、case ledger、coverage mapping 和 workspace/release validation。
5. 完成全部验证后追加 Implementation Observations；审计只保留必要实现，不在本 change 顺手做后续抽象。
6. 该能力是 additive，无现有 JSON contract 需要迁移。若发布前撤回，删除 static registry entry 和 JSON crate即可；Markdown 与 shared protocol 保持不变。

## Open Questions

无未回答开放问题，可以进入实现前审计。
