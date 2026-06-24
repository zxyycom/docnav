本 tasks 只给出 core 与 adapter SDK 标准参数迁移的粗粒度推进顺序；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：确认本 change 不替换 CLI frontend，不处理 manifest/probe/protocol response typed validation。
- [ ] 1.2 阻塞级审计：确认 core 和 adapter SDK 迁移范围是否仍适合一个 change；若过大，先拆成两个 adoption change。
- [ ] 1.3 阻塞级审计：确认保留 unknown argv、extra positional、unused operation flag warning 和 consumed field strict validation。

## 2. 轮廓实现

- [ ] 2.1 审计通过后，为 core document operations 接入标准参数 registration 和 typed values。
- [ ] 2.2 为 adapter direct CLI 与 adapter invoke 接入对应 registration 和 typed values。
- [ ] 2.3 将 help/default 文案读取路径切到标准参数 metadata。

## 3. 验证

- [ ] 3.1 添加行为对照测试，覆盖 warning、strict validation、help/default 和 request construction。
- [ ] 3.2 运行与 core CLI、adapter SDK 和协议边界匹配的 smoke/integration 验证。
