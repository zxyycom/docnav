本 tasks 清单用于 `add-outline-preview-skim-pack`：实现前必须先审计 outline preview 的 deterministic selection、预算和输出边界；当前 change 只在 `openspec/changes/add-outline-preview-skim-pack/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级实现前审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕 outline preview skim pack，不夹带 obvious auto-read、摘要生成、智能排序、多 query 搜索或 adapter 新 operation。
- [ ] 1.2 审计 capability ID 是否只使用现有 `core-cli` 和 `output-contract`，没有把 change name 当作长期 capability。
- [ ] 1.3 审计 `## Open Questions` 是否无未回答问题，并在 CLI owner 文档中定稿显式 preview surface、preview count 和默认预算后才开始实现。
- [ ] 1.4 审计 `protocol-json` 边界是否明确：第一版不得把 preview read content 混入 outline raw protocol result。
- [ ] 1.5 审计完成前不得执行任何实现任务。

## 2. Contract 和验证材料

- [ ] 2.1 更新 `docs/cli.md`，记录 outline preview 的显式 surface、selection inputs、预算参数、输出模式限制和与普通 outline/read 的关系。
- [ ] 2.2 更新 `docs/output.md`，记录 readable-view/readable-json 如何表达 outline entries、preview blocks、skipped reason、read diagnostic 和 continuation。
- [ ] 2.3 更新对应 `docs/schemas/`、`docs/examples/` 或 renderer config/conformance materials，覆盖 readable-json shape 和 readable-view block path。
- [ ] 2.4 明确 `protocol-json` 在该 surface 下是 unsupported combination 还是另有 documented raw contract；若选择 raw contract，必须补 schema/example/test。

## 3. Core 和 readable 实现

- [ ] 3.1 在 core CLI 参数解析中增加显式 outline preview control，并与 output mode、operation、strict argv policy 做互斥/支持校验。
- [ ] 3.2 实现 deterministic preview candidate selection：按 outline result order、preview count、非空 ref 和总预算选择，不使用 inferred importance 或模型判断。
- [ ] 3.3 复用现有 read pipeline 为 selected refs 读取 preview 内容，保持 path、config source、adapter selection 和 ref handoff 边界一致。
- [ ] 3.4 在 readable payload 中表达 preview success、skipped、pending 和 read diagnostic 状态，不把单个 preview read failure 升级为 outline primary failure。

## 4. Tests 和验证

- [ ] 4.1 增加 outline preview 选择前 N 个可读 entries 并保留 outline result 的测试。
- [ ] 4.2 增加预算耗尽、无 ref、read diagnostic 和分页/continuation 的 preview status 测试。
- [ ] 4.3 增加 deterministic selection 测试，证明结果不依赖模型判断、环境顺序或 adapter 私有 ref grammar。
- [ ] 4.4 增加 protocol-json 与 outline preview control 的行为测试，证明 raw protocol 不混入 preview composition payload。
- [ ] 4.5 运行范围匹配的格式化、单元/integration 测试、schema/example 验证；若跨输出边界或多个包，运行 `bun run verify:docnav-workspace`。
