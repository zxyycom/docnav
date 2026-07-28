# 语义测试 Case 维护

本文是 Docnav 语义测试账本的项目级 owner。它定义 Case、topic、当前测试实体的
关系，以及测试变更时的查询和闭合流程。[测试策略](../testing.md)负责测试层级与
覆盖目标；`scripts/test-evidence/` 负责项目 runner profile、实体发现、解析与
严格检查；项目级
[`test-evidence-review` skill](../../.codex/skills/test-evidence-review/SKILL.md)
只提供通用评审方法。

账本使用以下关系：

```text
Topic
  └── Case：一项稳定测试目的
        └── current test entity：runner 当前可报告的执行证据
```

权威来源按责任划分：

1. 当前源码和 runner 报告拥有测试实体的存在性与身份事实。
2. [`cases/topics.json`](cases/topics.json) 拥有受控 topic 分类。
3. `cases/<topic>.md` 拥有 Case 语义及 Case 与实体的映射。
4. project wrapper 只发现和校验这些来源，不提交派生实体清单、查询索引或其它
   语义副本。

## 核心对象

**Case** 是人工维护的语义单元，说明一组测试实体共同证明什么。每个 Case 必须有
全局唯一且稳定的 ID、一个 topic、一个当前 owner heading、至少一个当前实体和
至少一条可证伪的 `Proves`。Case ID 跟随测试目的，不随机械函数名变化。

**当前测试实体** 是 supported runner profile 能从源码静态发现、并由 runner
报告的可寻址节点。Cargo test、Bun `test` / `it` 和 core smoke leaf 的具体身份
由 project wrapper 归一。实体不手写、不持久化为第二套账本，也不承担长期语义。

**Topic** 是有界查询和责任分区。每个受控 topic 对应一个同名 Markdown 文件，
文件内可以保存多个 Case；topic 不随单个 Case 的增删自动消失。

Scenario、参数表中的数据行、fixture、helper、hook、mock、断言和测试步骤不是独立
账本对象。lint、类型检查、schema、build 和 CI job 属于工程校验，不因进入验证链
就成为测试实体。

## Case 粒度

Case 按 owner 契约与可观察结果划分，不按 runner 节点数量划分：

- 多个输入变体或多个层级的实体证明同一目的时，归入一个 Case。
- 一个实体确实观察多个独立目的时，可以关联多个 Case。
- 只有 owner requirement 或可观察失败信号不同，才拆分 Case。
- 不得为了让代码能单独进入账本而拆测试；同一 setup、action 和 assertion shape
  的输入变体优先使用现有入口或参数表。
- 无法归入有意义 Case 的实体，应合并、删除，或确认它其实属于工程校验；不得用
  复述测试名、AST match 或“测试稳定契约”的模板 Case 填补缺口。

Case 不是完整契约文档。`Owner` 指向完整规则，`Proves` 只记录本 Case 能从失败
信号中判定的行为。历史事故可以帮助选择代表输入，但不能单独制造 Case 或断言。

## 存储格式

`docs/testing/cases/topics.json` 是版本化 topic 表。每个 topic
必须有非空说明，并有一个 `docs/testing/cases/<topic>.md` 文件。topic 文件以
同名 H1 开始，每个 H2 block 保存一个 Case：

```markdown
# core-cli

## Case BB-CORE-ARGS-001: Core 拒绝缺失的 operation 参数

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `smoke|core:cli-argument-failure|CORE-ARGS-001`

Proves:
- document command 缺少本 operation 拥有的必需参数时返回稳定 input failure。
```

字段顺序固定为 `Owner`、`Entities`、`Proves`，避免解析结果依赖启发式推断。
`Owner` 必须是当前 workspace 内可解析的相对 `.md#heading` 引用；`Entities`
使用 project wrapper 报告的完整 key，不允许通配符。Case 标题、ID、owner 和
`Proves` 由人维护，不能从测试名或 AST 自动生成。

账本只保存当前 implemented Case。尚无当前实体的 planned test intention 留在
行为 owner 或 active OpenSpec change，不在 Case 文档中增加 `Status` 或空映射。

## 全树闭合

版本化 runner profile 位于
`scripts/test-evidence/supported-runner-profile.json`。它定义 Cargo、Bun 和
core smoke 的当前支持范围；静态发现与 runner report 必须复用同一 profile。

严格 `check` 总是从完整当前树重新发现，不使用 Git diff、缓存清单或历史账本作为
发现范围，并验证：

1. 规范化后的静态实体集合与 runtime runner 集合完全相等。
2. 每个当前实体至少被一个 Case 引用。
3. 每个 Case 至少引用一个当前实体，且不引用未知实体。
4. Case ID、topic、topic 文件、owner 引用和字段结构有效且无重复。

`static-only`、`runtime-only`、duplicate、unsupported、未知实体、无 Case 实体和
无法解析的 Case 都是阻断错误。一个实体可以映射到多个 Case，因此闭合比较使用
所有 Case 的实体并集，不要求一对一。

闭合只能证明实体存在和映射合法，不能机械证明 `Proves` 仍与断言一致。测试正文
变化但实体 key 不变时，维护者仍必须重读相关 Case 和 owner。

## 修改流程

出现测试新增、正文修改、删除、重命名、移动、拆分或合并，或者修改 runner
profile、静态规则、runner report 与身份归一时：

1. 读取[测试策略](../testing.md)、相关行为 owner 和本文件。
2. 用 `topics`、有界 `list` 和 `show` 找到相关 Case；按实体查找时使用
   `list --entity-key`。
3. 先写清“owner 明确承诺的语义 -> 调用方可观察结果”，再决定复用、修改或新增
   Case。新建 Case 时选择现有 topic；只有出现稳定的新责任分区才扩展 topic 表。
4. 修改测试与 Case 映射，运行能独立报告目标实体的最窄 runner 命令。
5. 运行完整 `check`，处理发现闭合、owner、Case 与映射诊断。
6. 运行范围匹配的 workspace verification。

结构变化按语义连续性处理：

- **rename / move**：语义连续时保留 Case ID，只更新实体 key。
- **split**：把新实体分配给原目的；只有目的也分裂时才拆 Case。
- **merge**：实体合并不自动合并 Case；一个合并后的实体可以继续支持多个目的。
- **delete**：从所有 Case 移除实体；失去当前证据的 Case 必须删除、改写或重新
  关联。
- **正文变化**：即使实体 key 未变，也要重审 Case 的 owner、证明信号、可靠性与
  维护价值。

自动化测试需要复制被测实现、增加测试专用观测接口或依赖高成本脆弱环境时，在
owner 验证说明或 change 审查中记录 `Manual CR:`、审查对象和判定条件，不创建空
测试或名义 Case。

## 查询与验证

从仓库根目录运行：

```bash
bun run test-evidence -- topics --root .
bun run test-evidence -- list --topic <topic> --root .
bun run test-evidence -- list --entity-key <entity-key> --root .
bun run test-evidence -- list --owner-ref <docs/path.md#heading> --root .
bun run test-evidence -- list --query <text> --limit <1-100> --offset <n> --root .
bun run test-evidence -- show <CASE-ID> --root .
bun run test-evidence -- check --root .
```

`topics`、`list` 和 `show` 只读取 Case 目录并输出 JSON；`check` 还会执行本
checkout 的项目 runner 并验证完整闭合。查询命令不修改文件，也不存在需要同步的
派生制品。

修改测试代码时还要运行目标 runner；跨多个验证入口时，再运行
`bun run verify:docnav-workspace:required` 或完整
`bun run verify:docnav-workspace`。
