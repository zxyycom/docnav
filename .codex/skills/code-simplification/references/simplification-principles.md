# 简化原则

## 目标

代码简化不是追求更少行数，而是在保持行为完全一致的前提下，让代码更容易阅读、理解、修改和调试。判断标准很直接：新成员是否能比读原实现更快理解这段代码。

## 行为必须完全一致

每次改动前都问：

- 对所有输入是否产生相同输出？
- 错误类型、错误信息、错误时机是否一致？
- 副作用和执行顺序是否一致？
- 边界条件、空值、异常路径是否仍被覆盖？
- 现有测试是否无需修改即可通过？

如果无法确认行为一致，就先不要改。为证明行为增加测试可以，但不要为了让简化后的实现通过而修改既有期望。

## 跟随项目约定

简化的目标是让代码更像这个 codebase，而不是套用外部偏好。动手前先确认：

1. `AGENTS.md`、项目规范和相邻代码。
2. 同类问题在附近模块中的处理方式。
3. import 排序、模块系统、函数声明风格、命名约定。
4. 错误处理、日志、类型标注深度和测试风格。

破坏一致性的“简化”只是 churn。项目中还要特别保护公开契约、owner boundary、输出分层、稳定 identifier，以及相邻语言和模块的既有模式。

## 清晰优先于聪明

紧凑代码只有在读者能立即理解时才是简化。需要脑中维护状态栈的表达式，通常应该拆开。

```typescript
// 不清晰：嵌套 ternary 需要停下来解析
const label = isNew ? "New" : isUpdated ? "Updated" : isArchived ? "Archived" : "Active";

// 更清晰：控制流直接表达优先级
function getStatusLabel(item: Item): string {
  if (item.isNew) return "New";
  if (item.isUpdated) return "Updated";
  if (item.isArchived) return "Archived";
  return "Active";
}
```

```typescript
// 不清晰：reduce 中混合复制、初始化和计数
const result = items.reduce((acc, item) => ({
  ...acc,
  [item.id]: { ...acc[item.id], count: (acc[item.id]?.count ?? 0) + 1 },
}), {});

// 更清晰：中间结构有名字，步骤可检查
const countById = new Map<string, number>();
for (const item of items) {
  countById.set(item.id, (countById.get(item.id) ?? 0) + 1);
}
```

## 保持平衡

过度简化常见于这些情况：

- 过度 inline，删除了能表达概念名称的 helper。
- 把两个简单函数合成一个多责任函数。
- 移除仍服务于 extensibility、testability 或边界隔离的抽象。
- 只优化行数，却让理解速度下降。

抽象没有价值时可以删除；抽象仍在给概念命名、隔离变化或降低测试成本时，应保留。

## 范围控制

默认只简化最近修改或用户明确指定的代码。无关重构会制造噪音，增加回归风险，也让 review 难以判断意图。发现更大问题时，记录为后续任务，而不是混进当前 diff。

## 常见自我说服

| 说法 | 现实 |
|---|---|
| “能跑就别碰。” | 能跑但难读的代码，未来出问题时会更难修。 |
| “行数更少就是更简单。” | 一行嵌套 ternary 往往不如五行 `if` 清楚。 |
| “顺手把旁边也改了。” | 未限定范围的改动会放大 review 和回归成本。 |
| “类型已经自解释。” | 类型说明结构，命名和边界说明意图。 |
| “这个抽象以后可能有用。” | 没有当前用途的 speculative abstraction 是成本。 |
| “原作者一定有原因。” | 可能有。先查上下文和 `git blame`，再判断原因是否仍成立。 |
| “我边加功能边重构。” | feature、bug fix 和 refactor 混在一起会让 review、回滚和历史理解变难。 |

## 红旗

- 简化后必须修改既有测试才能通过。
- “简化”版本更长、更绕或更难 review。
- 为了个人偏好重命名，而不是匹配项目语言。
- 删除错误处理、校验、日志或边界保护。
- 尚未理解代码就开始改。
- 把多种简化批量塞进一个大改动。
- 未经要求重构任务范围外的代码。
