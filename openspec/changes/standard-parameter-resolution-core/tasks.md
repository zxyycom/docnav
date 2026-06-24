本 tasks 只给出标准参数解析核心的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认本 change 只定义标准参数来源解析核心，不迁移 core CLI、adapter SDK 或 `clap`。
- [ ] 1.2 阻塞级审计：确认 typed field definition 是字段 metadata owner，standard parameter resolution 只消费它。
- [ ] 1.3 阻塞级审计：确认 passthrough 与 owner validation 的边界没有提前限制 adapter native options。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，定义标准参数 registration、source model 和 merge result 的最小结构。
- [ ] 2.2 实现 direct/config/default 来源合并和 typed runtime value 查询。
- [ ] 2.3 实现 passthrough 交接结构，不在标准参数层校验未映射字段。

## 3. 验证

- [ ] 3.1 添加小范围 fixture，证明来源优先级、required/default、typed value 和 passthrough 行为。
- [ ] 3.2 运行与共享参数解析层匹配的验证命令。
