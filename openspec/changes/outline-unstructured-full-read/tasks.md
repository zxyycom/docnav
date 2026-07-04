本 tasks 只列实现和验收工作；字段级规则与场景以本 change 的 spec delta 为准。

## 1. Owner Docs And Schemas

- [x] 1.1 更新 `docs/navigation-input-resolution.md`，记录 `outline_mode`、path rules、adapter-scoped cost threshold selector、candidate filtering、unit 最小阈值合并、默认值和 pre-dispatch 位置。
- [x] 1.2 更新 `docs/adapter-contract.md`，记录非结构化全文 hook set、cost measurement hook/declaration、default UTF-8 fallback 和 hook 禁止返回 entries/ref/page/continuation/readable-only wrapper 的边界。
- [x] 1.3 更新 `docs/protocol.md`、`docs/output.md`、`docs/cli.md` 和 `docs/adapters/markdown.md`，同步 outline union、非分页 full-read result、readable `/content` block、CLI 配置 selector 行为和 Markdown 正常 outline 边界。
- [x] 1.4 更新 config/protocol/readable schema 与 examples，覆盖 structured outline、path-triggered unstructured outline、cost-triggered unstructured outline、invalid path rule 和 threshold 不命中场景。

## 2. Protocol And Adapter Surface

- [x] 2.1 更新 shared result/readable 类型，使 outline success result 支持 `kind: "structured"` 与 `kind: "unstructured"` 两个分支。
- [x] 2.2 更新 adapter contract 类型或 capability metadata，支持非结构化全文 hook set 与 requested-units cost measurement hook。
- [x] 2.3 更新 readable output mapping 和 renderer config，使 non-structured outline result 可进入 readable-json、protocol-json 和 readable-view `/content` block。

## 3. Navigation And CLI Behavior

- [x] 3.1 实现 `outline_mode` resolution：path rules 优先，cost thresholds 只在无 path result 时运行，默认 `structured`。
- [x] 3.2 实现 path pattern matcher 集成，使用维护中的 matcher 或标准能力，并保留 normalized path、source ordering、last matching rule wins 和 source-scoped diagnostics。
- [x] 3.3 实现 cost threshold candidate filtering 与 unit 最小阈值合并；无 selected-adapter candidate threshold 时不得调用 cost hook。
- [x] 3.4 实现 `unstructured_full` pre-dispatch execution：命中时跳过正常 outline handler，通过 adapter hook set 或 default UTF-8 fallback 返回完整 content。
- [x] 3.5 保持默认 structured outline、Markdown `doc:full` fallback、read/find/info 和 core CLI output mode 行为不变；不新增 public outline-mode override flag。

## 4. Tests And Verification

- [x] 4.1 增加 navigation resolution tests：path rule order/source priority、invalid rule diagnostics、cost candidate filtering、unit merge、missing measurement、runtime measurement unavailable 和 path structured opt-out。
- [x] 4.2 增加 pre-dispatch/hook/fallback tests：path-triggered full read、cost-triggered full read、normal handler not called、hook facts used、default UTF-8 fallback 和 non-UTF-8 failure/structured fallback。
- [x] 4.3 增加 protocol/readable/CLI coverage：structured discriminator、unstructured content shape、stable reason、no entries/ref/page/continuation、readable-view `/content` block 和 empty/non-empty cost facts。
- [x] 4.4 运行相关 Rust tests、schema/example validation、Markdown smoke，并在最终实现完成后运行 `bun run verify:docnav-workspace` 或记录环境阻塞。
- [x] 4.5 实现完成后运行 `openspec validate outline-unstructured-full-read --type change --strict --no-interactive` 并准备归档评估。
