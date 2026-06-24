本 change 只起草 typed field definition 的底层想法和审计入口；当前内容是未审核临时文档，不影响现有其它文档或主规范。

## Why

标准参数、配置、manifest、probe 和协议 JSON 都需要描述 JSON path、类型、范围、枚举、默认值 metadata 和错误归属。继续把这些能力直接写进标准参数 change 会让标准参数承担过多 owner 责任。

先起草一个更底层的 typed field definition 能力，可以把“字段定义与校验 metadata”从“标准参数来源合并与 CLI 行为”中拆开。

## What Changes

- 新增一个 typed field definition 草案，用于表达 typed key、JSON path、schema metadata、基础 decoder/validator 和错误 path。
- 第一版只输出 schema metadata，不直接拥有完整 JSON Schema 文件生成。
- 记录 definition fingerprint / consistency check 的方向，用于后续防止同名字段语义漂移。
- 不迁移现有 CLI、config、manifest、probe 或 protocol 行为。
- 不修改 `unify-standard-parameter-definitions`，后续是否迁移由本 change 的审计门禁决定。

## Capabilities

### New Capabilities

- `typed-field-definitions`: typed field/path/value definition 的底层 metadata 与校验边界。

### Modified Capabilities

当前草案不直接修改已归档主 spec；审计门禁会确认是否需要改为 existing capability delta。

## Impact

- 可能影响未来共享 crate 边界、schema metadata 输出、标准参数底座和 JSON contract validation。
- 不改变当前 CLI、adapter、protocol-json、readable output、schema 文件或测试 fixture。
- 后续实现若改变 observable behavior，必须在对应 owner change 中补充 schema、example、docs 和 tests。
