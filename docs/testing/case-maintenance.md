# 测试证据维护

本文拥有 Docnav 项目内测试变更与测试证据目录的维护流程。通用的触发边界、原生
入口粒度、case 格式、固定路径、查询和派生索引契约由项目级
[`test-evidence-review` skill](../../.codex/skills/test-evidence-review/SKILL.md)
拥有。

权威源是 `docs/test-evidence/<topic>/<slug>.md` 中的独立 case Markdown；
[`test-evidence-topics.json`](../test-evidence/test-evidence-topics.json) 拥有受控
topic。`test-evidence-index.json` 只是可删除重建的查询投影，不手工编辑，也不成为
第二份账本。

## 使用时机

出现以下任一情况时，按本文维护对应 case：

1. 新增、修改、删除、重命名或移动原生测试入口。
2. 修改断言、fixture、mock 或时序，使测试契约、证明信号或可靠性发生变化。
3. 查询、整理、修复或审查已有测试证据。
4. 拆分或合并 runner 能独立报告的测试节点。

只运行既有测试，或只修改被测对象而没有修改测试时，不更新 case。只改测试内部
实现且入口定位、契约、证明信号和可靠性都不变时，保留原 case；仍需确认当前 diff
没有扩大测试意图。

## 原生入口粒度

一个保留的最小原生测试入口恰好对应一个 case。判断顺序如下：

1. runner 能否稳定单独命名并报告该节点的通过或失败。
2. 节点是否拥有一项完整测试意图。
3. 节点内部是否还包含结果可以分别归因的更小原生测试节点。

Rust 的 `#[test]` 函数、Bun 的 `it` / `test` 和 core smoke 展开后的 leaf task
通常是原生入口。以下对象不是独立 case：

- 测试文件、module、`describe`、package script、runner 和 CI job；
- setup、fixture、helper、mock、hook、断言和测试步骤；
- lint、类型检查、schema、生成物一致性、安全扫描等工程校验。

一个自定义测试程序只有在产生单一、不可再归因且意图单一的最终结果时，才可作为
一个入口。参数化测试按 runner 的真实报告粒度登记。一个入口混合多个可独立命名、
独立失败的意图时，先拆测试节点，再分别建立 case。

## 修改流程

1. 按[测试策略](../testing.md)和[覆盖矩阵](coverage.md)确定测试层级、行为 owner
   与需要观察的结果。
2. 列出本次新增、修改、删除和审查后仍保留的最小原生入口。
3. 运行 `topics`，再按测试名、入口、契约或既有 ID 使用 `list` / `show` 查找 case。
4. 对每个入口检查契约背景、直接证明信号、可观察性、fixture/时序可靠性、证据
   独立性和维护价值。
5. 新增或保留入口时，新建或更新唯一 case；删除入口时删除对应 case。只改变路径
   或测试名时更新原 case，不另建身份。
6. case 正文变化后运行 `sync-index --write`；随后运行目标测试和目录严格检查。

历史事故只能作为风险线索或代表性输入来源，不能独立制造 `Contract` 或断言。
planned 行为应留在 owner 文档或 active OpenSpec change，不创建没有当前测试入口的
case。自动化需要复制实现、测试专用观测接口或高成本脆弱环境时，在 owner 验证说明
或变更审查中记录 `Manual CR:`，不创建空测试或名义 case。

## ID 与 Topic

case ID 在全部 topic 中唯一，并符合：

```text
^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+){2,}-\d{3}$
```

同一原生入口只改变定位、实现或 topic 时保留 ID。入口被删除或旧聚合身份无法由
单一入口完整承接时终止旧 ID；终止的 ID 不复用。新增 ID 使用能够表达稳定责任和
意图的前缀，不要求沿用历史 `BB` / `WB` / `AUX` 分类。

topic 只表达稳定测试责任，不表达测试层级或文件布局。先用 `topics` 查询受控值；
没有合适 topic 时，先在 topic 表定义清楚责任和描述，再创建非空直属目录。移动
case 到另一个 topic 不改变 case ID。

## Case 内容

每个 `<topic>/<slug>.md` 恰好保存一个 case，并且各有且只有一个 `Entry:`、
`Contract:` 和 `Proves:`：

```markdown
### Case CORE-CLI-OUTPUT-001: Protocol output keeps stdout pure

Entry:
- `tests/output.test.ts > protocol output keeps stdout pure`

Contract:
- `docs/output.md` defines the protocol output channel boundary.

Proves:
- The selected protocol output writes one envelope to stdout and no JSON to stderr.
```

- `Entry` 中的全部定位必须指向同一个最小原生入口；只写文件或 suite 不足以定位
  case。
- `Contract` 压缩理解测试所需的 owner 语义，不替代 owner 文档。
- `Proves` 每项描述一个直接、可判断的可观察结果，不写“防止回归”或内部实现路径。
- 目录不使用 `Status`、`Code`、`Verification`、角色字段或源码 marker。

同一入口可以有多个共同服务一个意图的断言；若观察点已经可以独立命名和失败，应先
拆入口。否定性断言只有在 owner 明确定义稳定失败、安全、诊断或通道边界时才进入
case。

## 查询与验证

从仓库根目录运行：

```bash
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs topics --root .
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs list --topic <topic> --root .
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs show <case-id> --root .
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs sync-index --write --root .
node .codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs check --root .
bun run validate:docs -- cases
```

修改测试代码时还要运行覆盖该入口的最窄 runner 命令。跨多个验证入口时，再运行
`bun run verify:docnav-workspace:required` 或完整
`bun run verify:docnav-workspace`。
