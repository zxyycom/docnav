本 spec delta 定义 Docnav 共享 crate 的全局所有权、依赖方向和文档先行要求，确保去重不改变协议层、阅读层或 adapter-owned 语义。

## ADDED Requirements

### Requirement: 共享 Rust crate 所有权必须保持 Docnav 契约分层

Docnav 共享 Rust crate MUST 保持原始协议、document output 编排、JSON IO、readable renderer、diagnostics、direct CLI argv compatibility、adapter SDK 行为和格式 adapter 语义之间的既有分层。共享 crate MUST 只上移稳定契约和机械流程；routing、ref interpretation、format parsing、process runtime 和用户可见 surface policy 等 owner-specific 判断 MUST 留在既有 owner，除非后续 spec 明确改变。

#### Scenario: 共享 crate 依赖方向保持无环

- **WHEN** 实现新增 `docnav-diagnostics`、`docnav-cli-args`、`docnav-json-io` 或 `docnav-output`
- **THEN** 这些 crate 只依赖其契约所需的下层共享 crate
- **THEN** `docnav-json-io` 不依赖 `docnav-output`、`docnav-readable`、`docnav` core 或 `docnav-adapter-sdk`
- **THEN** `docnav-output` 可以依赖 `docnav-json-io`，但不依赖 `docnav` core 或 `docnav-adapter-sdk`
- **THEN** `docnav` core 和 `docnav-adapter-sdk` 可以依赖 `docnav-output`
- **THEN** 格式 adapter 不需要依赖 `docnav` core 即可产生 direct CLI document output

#### Scenario: 共享 crate 不接管 adapter-owned 语义

- **WHEN** adapter 生成、解析或拒绝一个 ref
- **THEN** 共享 protocol、output、diagnostics 和 CLI argv crate 将该 ref 视为 opaque value
- **THEN** 共享 crate 不推断 heading structure、唯一性、region boundary 或格式专属 navigation behavior

#### Scenario: 文档先行约束先于代码迁移

- **WHEN** 本 change 进入实现
- **THEN** 实现者先同步主规范、schema/example/fixture/testing 文档中的 owner 和验证说明
- **THEN** crate 新增或代码迁移在主规范和验证材料对齐之后开始
- **THEN** 代码实现不得作为定义共享 crate contract 的第一来源

#### Scenario: 本 change 的共享 crate 集合保持限定

- **WHEN** 本 change 进入实现
- **THEN** 不为 core path display normalization 引入 path utility crate
- **THEN** 不为 adapter process startup 或 registry command path handling 引入 process runner crate
- **THEN** 不为 manifest/probe/invoke ownership 引入 adapter boundary crate
- **THEN** `docnav-json-io` 不成为 schema、manifest/probe 或 document output policy owner
