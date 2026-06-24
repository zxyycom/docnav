本 tasks 只给出 lexopt frontend 的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认 `lexopt` 是实现目标，但 CLI 可观察 contract 写成行为而不是 crate 名。
- [ ] 1.2 阻塞级审计：确认 frontend 不拥有参数语义、默认值、operation applicability 或 strict validation。
- [ ] 1.3 阻塞级审计：确认 help 输出、warning 行为和 adapter native options 有明确验证入口。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，引入 thin lexopt frontend 并接入现有 command context。
- [ ] 2.2 将 core CLI 和 adapter direct CLI 的 argv tokenization 切到 frontend mapping。
- [ ] 2.3 将 help generation 连接到 standard parameter metadata 和 owner native option metadata。

## 3. 验证

- [ ] 3.1 添加或更新 help/warning/strict-validation 对照测试。
- [ ] 3.2 运行与 core CLI 和 adapter direct CLI 匹配的 smoke 验证。
