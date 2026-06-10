# Question Patterns

本文件在你已经触发 `interview-me`，但需要设计下一问或修正提问方式时读取。所有模式都服务同一个约束：one-question-at-a-time，且每个问题带一个 GUESS。

## 目录

- [核心格式](#核心格式)
- [开场定位问题](#开场定位问题)
- [拆解惯用 artifact](#拆解惯用-artifact)
- [处理 trade-off](#处理-trade-off)
- [识别 want vs should want](#识别-want-vs-should-want)
- [处理委托式回答](#处理委托式回答)
- [Confidence 更新](#confidence-更新)

## 核心格式

```text
Q: <one focused question>
GUESS: <your current best guess and why>
```

好问题有三个特征：

- 只消除一个关键不确定性。
- 让用户能纠正你的假设，而不是从空白开始写需求。
- 回答后会明显提高或降低 confidence。

差问题通常是：

- 一次问多个维度，例如同时问 who、timeline、budget、metrics。
- 让用户替你做抽象设计，例如 “你想要什么架构？”
- 没有 GUESS，导致用户只能提供泛泛答案。

## 开场定位问题

优先补齐 who、why now、success、constraint。每轮只选一个最影响方向的问题。

```text
Q: When this works, who notices first?
GUESS: You notice first, because the request sounds like a personal workflow gap rather than a team reporting gap.
```

```text
Q: What changed recently that makes this worth doing now?
GUESS: The current manual process started failing at a new volume, not because the idea is newly interesting.
```

```text
Q: What would make you say this was successful after one week?
GUESS: Fewer missed items matters more than a polished UI, because the pain sounds operational.
```

```text
Q: Which constraint is actually binding here: time, correctness, cost, maintainability, or user experience?
GUESS: Time is binding, because you asked for the smallest useful version rather than a broad redesign.
```

## 拆解惯用 artifact

当用户说 dashboard、app、bot、agent、API、automation、report 等 artifact 名称时，先验证 artifact 是否真是答案。

```text
Q: Is "dashboard" the thing you know you need, or just the first shape that came to mind for seeing the answer?
GUESS: It is a placeholder shape. The real need is to know status quickly before a recurring meeting.
```

```text
Q: If we removed the UI entirely, what decision would still need support?
GUESS: You need to choose which item to act on next, not browse data.
```

```text
Q: Is the problem that information is missing, scattered, hard to trust, or hard to act on?
GUESS: Scattered. You likely have the facts, but not in one place at decision time.
```

## 处理 trade-off

当两个价值冲突时，不要替用户决定。把 trade-off 放进一个可反驳的 guess。

```text
Q: Would you rather ship a narrow MVP this week, or design the flexible version even if it slips?
GUESS: Narrow MVP, because the request is framed around immediate use rather than long-term platform shape.
```

```text
Q: If correctness and speed conflict, which failure would be worse?
GUESS: A wrong answer is worse than a slow answer, because this sounds like it could drive a user-visible decision.
```

```text
Q: Should this optimize for the first-time setup or repeated daily use?
GUESS: Repeated use, because the pain sounds recurring rather than onboarding-heavy.
```

## 识别 want vs should want

这些回答需要继续追问：

- “应该要 scalable / clean / robust / modern。”
- “按 best practice 来。”
- “就用标准做法。”
- “我觉得我应该先做 spec / architecture / dashboard。”

可用追问：

```text
Q: If you didn't have to justify this to anyone, what would you actually want?
GUESS: You want the quickest reliable path, and the "proper" version is mostly pressure from how this is usually discussed.
```

```text
Q: What would you choose if no one reviewed the elegance of the solution?
GUESS: You would choose a boring, direct implementation over a more abstract design.
```

## 处理委托式回答

“Whatever you think” 不是确认。把它转成两个具体选项。

```text
Q: Should I optimize for A or B?
GUESS: A, because it better matches the outcome you described. B is safer only if future extensibility matters more than this week's result.
```

如果用户继续委托，说明无法达到 `>=95%` confidence，并要求他们选一个 binding priority。

## Confidence 更新

每轮后短句更新即可：

```text
Updated read: <what changed>. Confidence: ~60% - still missing: <gap>.
```

confidence 不该自动上升。只有在你能更准确预测用户反应时才上升。三轮后仍不上升，停止普通追问并说清楚：

```text
I've asked three questions and still can't predict your reactions. Something foundational is missing. Want to step back and name the real decision?
```
