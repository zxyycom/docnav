本 tasks 只给出 typed field definition 的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认 proposal、design、specs 和 tasks 都只围绕 typed field/path/value metadata，不包含标准参数来源合并、CLI argv、manifest/probe 迁移或 schema 文件生成细节。
- [ ] 1.2 阻塞级审计：确认本 change 没有修改 `unify-standard-parameter-definitions`，也没有要求实现前同步旧 change。
- [ ] 1.3 阻塞级审计：确认 capability ID 是否应保留为 `typed-field-definitions`，或改为复用现有主 spec owner。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，定义最小 typed field/path/value metadata 结构。
- [ ] 2.2 接入一个小范围 consumer fixture，证明 metadata、decode/validation 和 error path 可以被复用。
- [ ] 2.3 保留 schema metadata 输出，不实现完整 schema 文件生成。

## 3. 验证

- [ ] 3.1 添加小范围正反例，证明类型、枚举、范围和 required metadata 的基础校验行为。
- [ ] 3.2 运行与共享 crate 和 OpenSpec change 匹配的验证命令。
