---
name: interview-me
description: "在 planning、spec 或代码前，用 one-question-at-a-time interview 提取用户真正想要的东西，而不是他们以为自己应该要的东西。用于需求缺少 who/why/success/constraint、用户显式说 interview me/grill me/are we sure/stress-test my thinking，或你正在默默填补模糊需求时。"
---

# Interview Me

## 作用

用户说出的 artifact 名称不一定等于真实意图。这个 skill 用在 planning、spec、OpenSpec、MVP、代码或任务拆分之前，通过一轮一问的访谈把用户真正要达成的 outcome 说清楚。

交付物不是方案、spec 或 task list，而是用户明确确认过的 intent statement。后续 planning、OpenSpec、implementation 或其他 skill 都应消费这个 intent，而不是原始的模糊请求。

## 何时触发

使用本 skill 当：

- 请求缺少至少一项：**who**、**why now**、**success**、绑定 **constraint**。
- 请求是惯用说法而不是具体目标，例如 “build me X”、"make it faster"。
- 你正准备默默补全需求，或在两个合理价值之间替用户取舍。
- 用户明确说：interview me、grill me、are we sure、stress-test my thinking。

不要使用当：

- 请求已经明确且自包含，例如重命名、修 typo、移动文件、格式化。
- 用户明确要求速度优先，不做澄清。
- 这是纯信息问题，例如解释代码或概念。
- 你已经能以 `>=95%` 信心预测用户对接下来三个问题的反应。

本 skill 需要实时用户反馈。不要在 CI、scheduled runs、`/loop` 或 autonomous-loop 中调用；若需求模糊，把缺少互动反馈作为 blocker 说明。

## 按需读取

保持 `SKILL.md` 只承载核心流程。需要细节时只读取直接引用：

- [question-patterns.md](references/question-patterns.md)：设计下一问、附带 GUESS、处理 trade-off、更新 confidence。
- [examples-and-failure-modes.md](references/examples-and-failure-modes.md)：完整例子、anti-patterns、rationalizations、red flags、与其他 skills 的衔接。
- [original-skill.md](references/original-skill.md)：历史原文，仅作审计来源；不要修改。

## Workflow

1. 先写一个 hypothesis。

   在问用户之前，用一句话说明你当前认为用户真正想要什么，并给出诚实的 confidence：

   ```text
   HYPOTHESIS: <one-sentence read of the real intent>
   CONFIDENCE: ~30% - missing: <what is still unresolved>
   ```

   低于 `~70%` 时必须附一行 reason，让用户知道差距在哪里。

2. 一次只问一个问题，并附上你的 guess。

   ```text
   Q: <one focused question>
   GUESS: <your current best guess and why>
   ```

   等用户回答后再问下一题。不要一次抛出三个以上问题；那是 survey，不是 interview。

3. 区分 want 和 should want。

   当用户给出 best-practice、convention 或 sophistication-signaling 答案时，追问实际偏好：

   ```text
   If you didn't have to justify this to anyone, what would you actually want?
   ```

4. 循环更新 confidence。

   每轮回答后更新你的 read。若多轮之后 confidence 没有上升，停止堆问题，重新框定缺口。

5. 在 `>=95%` 前不要进入 planning。

   停止条件是：你能预测用户对接下来三个可能问题的反应。能预测则进入 restate；不能预测则继续问一个最有信息量的问题。

6. Restate 并要求明确确认。

   用用户自己的语言写 5-8 行，让用户逐行确认或修正：

   ```text
   Here's what I now think you want:

   - Outcome: <one line>
   - User: <one line>
   - Why now: <one line>
   - Success: <one line>
   - Constraint: <one line>
   - Out of scope: <one line>

   Yes / no / refine?
   ```

   `Out of scope` 必须出现。未说清不做什么，通常就是后续错位的来源。

7. 只接受明确 yes。

   "Whatever you think"、"sounds good"、"sure, let's go" 和沉默都不是明确确认。把它们转成具体选择或询问是否要 refine。若用户修正，合并修正后重新 restate。

## Output Contract

本 skill 的输出是已确认的 intent statement，加上用户明确 yes。不要提前产出 spec、plan、task list 或 implementation。

如果 intent 需要跨 session 或交接，只能在用户确认后提议保存到 `docs/intent/[topic].md`；用户再次确认前不要写文件。

## Verification

完成一次 interview-me 后检查：

- [ ] 第一轮有 hypothesis 和 confidence。
- [ ] 每个低于 `~70%` 的 confidence 都说明了 unresolved gap。
- [ ] 每次只问一个问题，且包含 GUESS。
- [ ] 遇到 convention / best-practice / sophistication-signaling 时追问了真实偏好。
- [ ] 写回 Outcome / User / Why now / Success / Constraint / Out of scope。
- [ ] 用户给出明确 yes，而不是委托、含糊同意或沉默。
- [ ] 停止时能预测用户对接下来三个问题的反应。
- [ ] 后续 planning、OpenSpec 或 implementation 基于确认后的 intent，而不是原始模糊请求。
