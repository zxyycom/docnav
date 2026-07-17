# 测试用例维护

本文定义测试函数、smoke case、fixture 证明目标与源码 `@case` 标记变更时的维护流程。[测试用例编号账本](cases.md) 只保存稳定语义 case，不是测试函数清单；本文拥有编号规则、case 归属判断和账本更新规则。

## 使用时机

出现以下任一变更时，先按本文判断是否查看或更新对应 case 条目：

1. 新增、删除、重命名或移动测试函数。
2. 修改测试断言，使测试证明目标、等价类或可观察结果发生变化。
3. 新增、删除或移动源码 `@case` 标记。
4. 把已有测试纳入或移出某个 case。
5. 为修复、重构或验证工具链改动补测试。

只改测试内部实现细节，且证明目标、断言语义、`@case` 归属和源码路径都不变时，账本保持不变；交付前确认本次 diff 没有扩大测试职责。

## 编号规则

测试用例编号使用 `类别-责任域-证明意图-NNN`：

1. `BB`: 黑盒测试，从真实入口观察用户链路、进程边界或输出边界。
2. `WB`: 白盒测试，从 owner 边界、函数、fixture 或 conformance 入口证明内部语义。
3. `AUX`: 辅助脚本语义 case，证明测试、验证、质量观测、打包或调度链路的当前工具链 contract。

责任域当前使用 `CORE`、`NAV`、`MD`、`PROTO`、`READABLE`、`ADAPTER`、`DIAG`、`CLIARGS`、`JSONIO`、`OUTPUT`、`PARAM`、`WORKSPACE`、`SMOKE`、`PARALLEL`、`QUALITY`、`RELEASE`、`CASE`。`NAV` 覆盖 `docnav-navigation` 的 routing、adapter selection、navigation input resolution、request construction 和 adapter dispatch owner 边界。`SDK` 仅保留为历史 case id 兼容标签；新 case 涉及 linked adapter strategy contract、descriptor、capability 或 adapter strategy semantic validation 时使用 `ADAPTER`，core catalog、source resolution 和 closed input construction 则按 `CORE` / `NAV` owner 归类。新增责任域时，先在本文登记责任域，再同步账本条目和源码 `@case` 标记。

## 测试变更流程

1. 先按 [测试策略](../testing.md) 判断测试层级和 owner 边界。
2. 用 [覆盖矩阵](coverage.md) 判断改动是否新增了证明目标、责任层级或等价类。
3. 在 [测试用例编号账本](cases.md) 查找最接近的 case 条目。
4. 判断本次变更是否仍属于现有 case 的同一责任边界或同一行为链路。
5. 证明目标属于现有 case 时，在同一 patch 更新该 case 的 `Proves:` 或 Mermaid 叶子断言；嵌入已有 case 的新增断言必须共享该 case 的责任边界，并指向明文 owner 语义。
6. 证明目标不同且需要独立的可执行证明时，新增或拆分 case，并添加源码 `@case` 标记。
7. 新增、拆分或扩展测试断言前，先写出“owner 明确承诺的语义 -> 当前可观察结果”的证明目标；证明只描述当前输入、行为和结果，不描述测试可以防止什么未来回归，也不反向推断实现采用了哪条内部路径。涉及 strict public input、primary `DiagnosticRecord` 或 success payload shape 时，证明目标必须指向对应 input owner、protocol/output owner、schema/example 或覆盖矩阵。涉及 diagnostic code/details helper、surface 投影、schema/example/fixture 校验或消费方行为时，证明目标必须区分 helper 自身可观察行为和 public surface owner。历史回归、默认不存在或“没有发生某事”只能作为输入选择或风险说明，不能替代证明目标，除非 owner 文档、schema、示例或覆盖矩阵明文要求校验该否定边界。
8. Config source、`docnav config inspect` 或 adapter native option namespace 的测试变更必须先绑定到 owner 文档中的 source inspection、parameter aggregation projection、`options.<adapter-id>.<option-key>`、selected adapter namespace、direct edit/read 或 protocol/readable 行为隔离语义；旧 `config get|set|unset|list` 只作为同一 breaking legacy rejection 等价类，选择一个有写入风险的代表。
9. 只提升重构信心的改动使用现有测试、局部验证命令或代码审查证明。
10. Owner 明确承诺的行为需要自动化时，断言调用方可观察结果；owner 未承诺或当前层级无法观察时，缩小 `Proves:`，不补防御性断言。
11. 同一语义类型或等价类只选一个代表。语法写法、枚举字面量和同类非法值不单独扩展 case，除非它们进入不同的 owner-defined 分支。
12. 自动化需要复制实现、测试专用观测接口或高成本脆弱环境时，可以不建测试；在 owner 验证说明或变更审查记录中留下 `Manual CR:`、审查对象和判定条件，不创建空测试或假 implemented case。

## Case 归属规则

每个 `@case` 标记绑定一个稳定语义 case 的主要验证入口；多个测试函数可以共享同一 case，前提是它们证明同一责任边界或同一行为链路。只服务局部实现、测试 helper 或重构信心的 supporting tests 可以没有 `@case`，case catalog validator 也不枚举测试函数来要求逐项登记。

否定性断言只在 owner 文档把该行为定义为稳定契约、安全边界、诊断投影或输出通道边界时进入测试。该限制同时适用于新增 case、拆分 case、为已有 case 增加分支，以及在已有正向行为链路中增加辅助断言。

CLI smoke case 以外部链路类型作为归属基座：

1. operation、output mode、非法参数和 fixture 是覆盖维度，并入对应链路 case，不展开为笛卡尔积 case。
2. 连续链路可以在一个 case 内串行执行多个 CLI 命令。
3. 只有独立链路才拆成独立 task 调度。
4. 现有 smoke 报告继续使用 `CORE-*` 和 `MD-*` task id；账本 case id 只用于审计编号和证明目标。

保留多分支 case 的条件：

1. 多个分支共享同一初始状态、执行上下文或行为链路。
2. 拆分会制造额外 fixture、状态同步或审计成本。
3. `Proves:` 或 Mermaid 能写清共享基座、分支条件、处理阶段和各叶子断言。

拆分 case 的条件：

1. 变更证明了新的 owner 边界、观察入口、可观察行为类型或等价类。
2. 存在清晰且稳定的共享基座、fixture builder 或状态获取函数。
3. 拆分后能降低审计范围和维护成本。

## 账本更新规则

新增或调整账本条目时：

1. 在 [测试用例编号账本](cases.md) 新增或更新一个 `### CASE-ID ...` entry，并填写 `Status:` 和 `Proves:`。
2. `Status: implemented` 必须填写单个主要验证入口 `Code:`，并在负责该测试语义的入口位置添加唯一 `@case CASE-ID` 标记。Supporting files 不重复登记 `Code:` 或 marker；只有出现独立 owner 语义时才拆成新 case。
3. `Status: planned` 可先不填写 `Code:`，也不得提前添加源码 `@case` 标记；实现时改为 `implemented`。
4. 标记位置优先放在入口处：smoke task object、`describe(...)`、测试文件入口、Rust `mod tests` 内的 case 段落开头，或同一语义分组的第一个测试前。
5. 默认按单一路径描述 case：输入或触发 -> 被测行为 -> 可观察结果。
6. 若同一 case 必须覆盖多个分支，使用 Mermaid `flowchart LR`，按“输入或触发 -> 分支判断 -> 处理阶段 -> 可观察结果”组织；每个叶子节点应对应一个断言分支或验证点。
7. 使用具体验证点替代“覆盖多种场景”这类概括。

## 验证

修改测试用例维护文档、账本、源码 `@case` 标记或 `Code:` 路径后，运行：

```bash
bun run validate:docs -- cases
```

若同时修改测试代码，继续运行覆盖该 owner 边界的最窄测试命令；跨多个验证入口时再运行 `bun run verify:docnav-workspace:required` 或更高层级验证。
