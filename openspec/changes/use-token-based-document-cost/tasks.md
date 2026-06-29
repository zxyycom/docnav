本 tasks 清单记录 SDK cost helper 与 Markdown token-informed cost 的后续探索和实施入口；当前只在 `openspec/changes/use-token-based-document-cost/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“SDK 提供 cost/budget 机制，Markdown adapter 自行决定 token-informed cost 展示”这一核心目标；审计未完成前不得执行实现任务。
- [ ] 1.2 审计本 change 是否依赖 `explore-structured-protocol-fields` 确认 raw protocol `cost` shape，且没有在本 change 中提前固定协议字段结构。
- [ ] 1.3 审计 capability ID 是否只复用现有 `adapter-protocol` 和 `markdown-navigation`。
- [ ] 1.4 审计当前 change 是否只包含 `openspec/changes/use-token-based-document-cost/` 下的未审核临时 artifacts。

## 2. 方案细化

- [ ] 2.1 根据协议结构化探索结果，确认 SDK cost measurement 类型和 formatter 边界。
- [ ] 2.2 审计 tokenizer 依赖、encoding 选择、许可、离线构建、性能和 release 影响。
- [ ] 2.3 确认 Markdown adapter 首期展示哪些 cost unit，以及哪些内容只属于 readable 聚合。

## 3. 实施与验证

- [ ] 3.1 实现 SDK cost / budget helper，并保持 adapter policy 可覆盖。
- [ ] 3.2 更新 Markdown adapter cost 计算和展示。
- [ ] 3.3 同步主规范、schema/example、fixture 和测试。
- [ ] 3.4 运行范围匹配的 Rust、schema/example 和 workspace 验证。
