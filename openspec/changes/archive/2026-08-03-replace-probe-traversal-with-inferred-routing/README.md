本 README 是 `replace-probe-traversal-with-inferred-routing` 的入口与状态说明：它为“用 manifest-owned pathname hints 和精确 registry lookup 替代 adapter probe 遍历，并完整删除 probe surface”的已批准 Target 定位权威来源、实施就绪状态与读取顺序；该 Target 已通过实施前审计，但尚未应用为 Current。

# replace-probe-traversal-with-inferred-routing

从 adapter manifests 派生 basename suffix、exact filename 与 format identity 索引，在任何 document filesystem I/O 前用一次纯 pathname 匹配和一次精确 registry lookup 替代按注册顺序遍历 adapter probe；不新增 routing dependency，并实施完整的 no-probe 迁移。

## Planning and Readiness Status

- **已批准的 routing architecture**：automatic routing 先从调用 pathname 词法派生 routing basename，不对目标文档执行 metadata、open、canonicalize、read 或 parse。大小写敏感的 exact `formats[].filenames[]` 优先；未命中时，把 `formats[].extensions[]` 作为可含多个点的完整-basename suffix，按 ASCII 大小写归一化匹配并选择最长命中，随后取得 manifest-owned format identity。命中后才进入文件路径与内容处理；routing 不验证真实性，也不使用外部 MIME/inference/regex dependency。
- **已批准的 dispatch semantics**：pathname 与内容冲突时仍按 pathname 选择；known hint 的 missing/empty/malformed/non-UTF-8 文档先选中 adapter，再由 selected adapter 返回 owner diagnostic；selected failure 不回退。JSON syntax、trailing input、duplicate member 和 depth overflow 迁移为 `DOCUMENT_CONTENT_INVALID` 与稳定 JSON reason，不再使用 probe 或 post-probe internal error。显式 `--adapter` 只做 exact id lookup 并强制该 adapter 解析。
- **已批准的 breaking scope**：完整删除 probe surface、candidate traversal 与 compatibility/inspection fallback。Manifest routing conflicts 在 construction/doctor/release validation 阶段阻断；逃到 runtime 时是 global internal failure，不按 registry order 猜测。
- **已批准的初始 JSON hints**：普通 `.json` 继续路由 JSON；`.code-workspace` 作为 pathname hint 路由 JSON；exact filename `.prettierrc` 和 `.watchmanconfig` 路由 JSON。它们只表达 selection intent：`.prettierrc` 的 YAML 分支和带 comments 的 `.code-workspace` 在 JSONC/YAML support 落地前仍可被 selected JSON adapter 拒绝。
- **实施就绪状态**：tasks 0.1–0.11 的调查、批准、removal inventory、cross-change handoff 和 artifact audit 已完成，sections 1–7 可以按 `tasks.md` 开始；尚未完成任何 production、owner-doc、schema、test 或 release implementation task。JSONC grammar support 由独立 change 规划，不扩大本 change。

术语约定：`routing pathname` 是从调用路径与 cwd 词法派生、尚未经过目标文件 metadata/open/canonicalize/read 的 invocation-private pathname；`pathname hint resolution` 是 automatic path 在其完整 basename 上一次性匹配 manifest-declared suffix 或 exact filename，并映射为 project-owned format identity。Change 名称中保留的 `inferred-routing` 是历史标识，不表示使用 inference library、content detection 或通用 regex。

## Authority and Reading Order

1. 先读正式调查主题 [`docs/investigations/dependencies/format-routing-inference.md`](../../../docs/investigations/dependencies/format-routing-inference.md)，恢复候选比较、pathname alias 复查、限制和形成时建议。报告保存证据，不替代本 change 中已经确认的决策。
2. 读长期活动决策 [`route-by-manifest-basename-hints`](../../../docs/decisions/adapter-selection/route-by-manifest-basename-hints.md)，恢复跨 change 的已批准默认方向；其 `unaligned` 表示主规范和实现尚未完成迁移。前序 `route-by-manifest-pathname-hints` 已由该记录修订并归档。
3. 读 [`proposal.md`](./proposal.md)，确认问题、breaking scope、capability 影响面和 non-goals。
4. 读 [`design.md`](./design.md)，恢复获批 mechanism、exact outcomes、被拒绝方案与已完成的实施前审计。
5. 按需读 [`specs/`](./specs/) 中的 capability delta；每份 delta 都是可独立读取、尚未应用的 Target，不覆盖主规范的 Current 状态。
6. 最后读 [`tasks.md`](./tasks.md)，从已关闭的 planning gate 继续遵守测试、同步、实现和验证顺序。

正式调查主题拥有各形成时点的证据及其边界；本 change 拥有已经批准但尚未应用的 Target、影响面和实施顺序；`route-by-manifest-basename-hints` 拥有跨 change 的默认方向。Planning 已就绪不等于 implementation 已完成：apply 完成前，主规范、代码、测试和 release artifacts 仍是 Current 实现证据。
