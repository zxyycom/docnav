本 tasks 只给出 typed JSON contract validation 的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认本 change 复用 typed-field engine，而不是把 manifest/probe/protocol JSON 归入标准参数。
- [ ] 1.2 阻塞级审计：确认 JSON Schema 仍作为契约材料保留，且本 change 不直接生成完整 schema 文件。
- [ ] 1.3 阻塞级审计：确认 runtime `jsonschema` 依赖移除必须等待 parity tests 和 error mapping 审计。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，列出 manifest、probe、protocol request/response 当前使用的 schema keywords。
- [ ] 2.2 将字段级 keyword 映射到 typed field metadata，将跨字段规则映射到 semantic validation。
- [ ] 2.3 按审计批准的首轮 surface 接入 typed decoder。

## 3. 验证

- [ ] 3.1 添加正反例，覆盖 unknown fields、missing required fields、wrong types、version constants 和 operation/result pairing。
- [ ] 3.2 运行 schema/example/fixture 和 protocol boundary 验证。
