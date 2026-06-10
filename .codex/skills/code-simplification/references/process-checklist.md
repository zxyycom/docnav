# 简化流程清单

## 1. 理解后再动手

先回答这些问题：

- 这段代码的责任是什么？
- 谁调用它？它调用谁？
- 正常路径、错误路径和边界条件是什么？
- 哪些测试定义了期望行为？
- 当前形态是否来自性能、平台、兼容性或历史约束？
- `git blame` 或相邻提交能否解释当时的取舍？

如果答不上来，继续读上下文。简化前先尊重已有结构，这就是 Chesterton's Fence：看见障碍物时，先理解它为什么存在，再决定是否移除。

## 2. 识别结构复杂度

| Pattern | Signal | Simplification |
|---|---|---|
| 深层嵌套，通常 3 层以上 | 控制流难跟踪 | 用 guard clause 或 helper 拆出条件 |
| 长函数，通常 50 行以上 | 多个责任混在一起 | 拆成有领域名称的小函数 |
| 嵌套 ternary | 需要脑内维护判断栈 | 改为 `if` / `else`、`switch` 或 lookup |
| Boolean flag 参数 | `doThing(true, false, true)` | 改 options object 或拆分函数 |
| 重复条件 | 多处出现同一 `if` | 提取为命名清楚的 predicate |

## 3. 识别命名和可读性问题

| Pattern | Signal | Simplification |
|---|---|---|
| 泛化命名 | `data`、`result`、`temp`、`val`、`item` | 改成表达内容的名称，如 `userProfile`、`validationErrors` |
| 非通用缩写 | `usr`、`cfg`、`btn`、`evt` | 使用完整词；保留通用缩写如 `id`、`url`、`api` |
| 误导性命名 | `get` 同时修改状态 | 名称反映真实行为 |
| 解释 what 的注释 | `// increment counter` | 删除，代码本身足够清楚 |
| 解释 why 的注释 | `// Retry because ...` | 保留，这类注释携带意图 |

## 4. 识别冗余

| Pattern | Signal | Simplification |
|---|---|---|
| 重复逻辑 | 多处重复 5 行以上 | 提取共享函数或集中策略 |
| 死代码 | unreachable branch、unused variable、注释掉的旧代码 | 确认后删除 |
| 无价值 wrapper | 只转发且不表达概念 | inline 后直接调用底层函数 |
| 过度工程 | factory-for-a-factory、single strategy | 换成直接实现 |
| 冗余类型断言 | 已可推断却再次 cast | 删除断言 |

## 5. 小步应用

每次只做一种简化：

1. 修改一处或一类问题。
2. 运行相关测试或最小验证。
3. 通过后继续下一步。
4. 失败时先定位或撤回该步，不要继续叠加。

refactor 应尽量与 feature 或 bug fix 分开。超过约 500 行的机械重构，不宜手工逐处改，优先考虑 codemod、脚本、`sed`、AST transform 或语言服务。

## 6. 结果复核

完成后比较 before / after：

- 新版本是否真的更快理解？
- 是否引入了和 codebase 不一致的新模式？
- diff 是否聚焦、可 review？
- 是否遗漏 unused import、unreachable branch、重复 helper？
- reviewer 能否在不猜测行为变化的情况下认可这次改动？

如果简化版本更难理解或更难 review，就撤回。不是每次简化尝试都值得保留。
