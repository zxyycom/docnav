# Examples and Failure Modes

## 目录

- [完整例子](#完整例子)
- [常见合理化](#常见合理化)
- [Red Flags](#red-flags)
- [Interaction Guidance](#interaction-guidance)

## 完整例子

### 不使用 interview-me

```text
User: build me a dashboard for our metrics
Agent: starts proposing chart libraries, data models, and layouts
```

问题在于 agent 已经假设了 who、metrics、success、cadence 和 decision。任何一个假设错了，后续 spec 和 implementation 都会偏。

### 使用 interview-me

```text
User: build me a dashboard for our metrics

Agent:
HYPOTHESIS: You want a way to answer "how are we doing?" in standup, and "dashboard" is the conventional shape that came to mind.
CONFIDENCE: ~30% - missing: who it is for, what "metrics" means, and what success looks like.

Q: When you say "how are we doing?", who is asking: you, the engineering team, or leadership?
GUESS: The engineering team in standup, because "we" often points there and dashboards are commonly used for team status.

User: Actually it is for me. I keep losing track of experiments and their early signals.

Agent:
Updated read: this is not a team dashboard. It is a personal experiment-tracking problem. Confidence: ~60% - still missing what "early signal" means and what done looks like.

Q: Is the gap that you do not know which experiments exist, or that you cannot see their results in one place?
GUESS: The second one. I think you have a list somewhere, but the results live across tools.

User: First one. I literally do not have a list. They are spread across different docs.
```

两轮后，真实需求从 dashboard 变成 experiment list。artifact、scope、acceptance criteria 都变了。

## 常见合理化

| Rationalization | Reality |
|---|---|
| “需求已经够清楚了。” | 如果你现在不能用一句话写出用户想要的 outcome，就还不清楚。先写 hypothesis。 |
| “问太多浪费时间。” | 4-6 个高信息量问题比做错 artifact 便宜得多。 |
| “边做边发现。” | implementation 后再发现意图偏差就是 rework。 |
| “用户说 whatever you think，所以我决定就好。” | 这是委托，不是确认。用两个具体选项让用户选择。 |
| “我应该先给几个方案。” | 用户还不知道真实目标时，方案会扩大搜索空间；问题才会收敛。 |
| “附上 GUESS 会引导用户。” | 让用户反驳你的 guess 正是机制。风险是用户礼貌同意，所以要显得可以被纠正。 |
| “聊得够多了，我懂了。” | 测试它：你能预测用户对接下来三个问题的反应吗？ |
| “用户说 yes 了。” | 如果 yes 跟在模糊 restate 后，它可能只是礼貌结束。具体 restate 后再确认。 |

## Red Flags

- 一条消息里问三个或更多问题。
- 问题没有 GUESS。
- 接受 “whatever you think is best” 作为终点。
- 在明确确认前产出 spec、plan、task list 或 implementation。
- 把 “best practice 是什么？” 当成 “你真正想要什么？”
- 用户给出 “scalable / clean / modern / robust” 后没有追问具体 outcome。
- 三轮后 confidence 没有明显上升。
- `~70%` 以下 confidence 没有说明 unresolved gap。
- restate 里没有 `Out of scope`。
- 用户确认前保存 intent doc。

## Interaction Guidance

`interview-me` 是 pre-decision intent extraction。它在所有 downstream work 之前运行。

- `planning`：只有确认 intent 后才写 plan；plan 必须引用确认后的 outcome、constraint 和 out of scope。
- `OpenSpec`：Docnav 变更若需要 OpenSpec，先用本 skill 确认 intent，再进入 `openspec-explore` 或 `openspec-propose`。
- `MVP`：如果用户其实在问 “最小可用版本是什么”，先确认 success 和 binding constraint，再定义 MVP。
- `incremental-implementation`：多文件实现应在 intent 或 spec 明确后使用。
- `doubt-driven-development`：这是 post-decision review；不要用它替代前置访谈。
- `source-driven-development`：用于验证框架或 API 事实；它不回答用户真正想要什么。

当目标是 “我想要 X 但不知道怎么 scope”，先完成 intent，再做 option shaping。不要在用户尚未确认 intent 时生成正式方案。
