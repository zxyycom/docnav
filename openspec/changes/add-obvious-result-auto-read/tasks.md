本 tasks 清单用于 `add-obvious-result-auto-read`：实现前必须先审计唯一明确结果自动 read 的 scope、surface 和输出边界；当前 change 只在 `openspec/changes/add-obvious-result-auto-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级实现前审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕 outline/find 唯一明确结果自动 read，不夹带 outline preview、智能选择、多候选排序或 adapter 新 operation。
- [ ] 1.2 审计 capability ID 是否只使用现有 `core-cli` 和 `output-contract`，没有把 change name 当作长期 capability。
- [ ] 1.3 审计 `## Open Questions` 是否无未回答问题，并在 CLI owner 文档中定稿显式 composition surface 的具体拼写后才开始实现。
- [ ] 1.4 审计 `protocol-json` 边界是否明确：第一版不得把 read content 混入 outline/find raw protocol result。
- [ ] 1.5 审计完成前不得执行任何实现任务。

## 2. Contract 和验证材料

- [ ] 2.1 更新 `docs/cli.md`，记录 obvious-result auto-read 的显式 surface、适用命令、输出模式限制、退出行为和与普通 outline/find/read 的关系。
- [ ] 2.2 更新 `docs/output.md`，记录 readable-view/readable-json 如何表达 base result、auto-read success、skipped reason、read diagnostic 和 continuation。
- [ ] 2.3 更新对应 `docs/schemas/`、`docs/examples/` 或 renderer config/conformance materials，覆盖 readable-json shape 和 readable-view block path。
- [ ] 2.4 明确 `protocol-json` 在该 surface 下是 unsupported combination 还是另有 documented raw contract；若选择 raw contract，必须补 schema/example/test。

## 3. Core 和 readable 实现

- [ ] 3.1 在 core CLI 参数解析中增加显式 obvious-result auto-read control，并与 output mode、operation、strict argv policy 做互斥/支持校验。
- [ ] 3.2 在 document operation pipeline 中实现 outline/find 成功后的单候选检测：恰好一个 item、非空 ref、预算允许、当前结果未处于需要先继续分页的状态。
- [ ] 3.3 复用现有 read pipeline 执行追加 read，保持 path、config source、adapter selection 和 ref handoff 边界一致。
- [ ] 3.4 在 readable payload 中表达 auto-read success、skipped、pending 和 read diagnostic 状态，不把追加 read failure 升级为 base operation primary failure。

## 4. Tests 和验证

- [ ] 4.1 增加 outline 单 entry 自动 read、outline 多 entry 不自动 read、outline 无 ref/预算不足 skipped 的测试。
- [ ] 4.2 增加 find 单 match 自动 read、find 多 match 不自动 read、find 无 match skipped 的测试。
- [ ] 4.3 增加追加 read diagnostic 的 readable output 测试，证明 base operation result 保留。
- [ ] 4.4 增加 protocol-json 与 obvious auto-read control 的行为测试，证明 raw protocol 不混入 readable composition payload。
- [ ] 4.5 运行范围匹配的格式化、单元/integration 测试、schema/example 验证；若跨输出边界或多个包，运行 `bun run verify:docnav-workspace`。
