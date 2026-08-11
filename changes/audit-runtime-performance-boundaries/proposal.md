# Proposal

本计划协调一次有限的 Docnav runtime performance 调查、人工 budget/gate 决定和 owner handoff；Change 只保存本次工作的范围与顺序，不充当调查报告、长期决策或稳定规则 owner。

## Why

有限导航同时受启动、CPU、I/O、内存、输出、发布体积和极端输入伸缩影响。现有局部数字缺少统一 workload、环境和归因，不能可靠升级为 baseline 或 gate，也不能直接决定 parser、cache、service、cost 或 pagination 方案。

## Outcome

形成一份独立的 runtime investigation report，定义有限 initial workload packet、记录可复现 measurements 并把 material observations 归到稳定 categories；随后由人类分别批准或拒绝 baseline、budget、blocking gate 和 owner handoff。形成时证据留在调查报告，获批的跨 change 方向进入决策，稳定规则进入对应 owner，本 Change 只跟踪这些交接是否完成。

## Scope

- 覆盖 startup/process、wall/CPU、I/O/重复准备、peak/retained memory、output/page、package size 和极端伸缩。
- Representative 与 stress/adversarial workload 分开记录；未选组合显式标为 `unmeasured/future`，不展开笛卡尔积。
- Observation 默认非阻断；budget、threshold、CI/merge gate 和具体优化都必须单独人工批准。
- 调查报告按 `investigation-report` 契约保存形成时背景、目的、范围、依据、结果与边界；Change 目录不另建第二份 audit report。
- 不选择或实现 parser、cache、service、document state、token estimator、find、renderer、allocator 或依赖优化。

## Success Criteria

- Initial workload packet、停止规则和每条 measurement 的完整复现条件已经进入一份结构有效、可独立阅读的 investigation report。
- 代表性与极端结果具有 measurement state、噪声边界和最小充分归因，未知项没有被伪装成零或结论。
- 人类分别决定 baseline、budget、gate 和 owner handoff；未批准项保持 observation，不能由执行子代理补成结论。
- 获批长期规则在相应写入获得明确授权后进入对应 owner/decision；具体优化只形成候选，只有用户明确要求时才创建或维护对应 Change。本计划通过 Change、调查、决策、文档和 workspace 验证。

## Affected Owners

- `docs/investigations/runtime/` 下按 `investigation-report` skill 创建的独立主题：本轮形成时背景、workload、measurement、attribution、结果和认识边界。
- [长期决策分工](../../docs/navigation.md#长期决策调查报告与-change-plan-分工)：经确认且跨 change 持续有效的 baseline/budget/gate 方向与理由，由适用决策领域中的 Markdown 记录拥有。
- 未来运行性能稳定规则的 owner 文档和 `docs/navigation.md` 入口，仅在人工批准后创建或更新；调查报告和 Change 都不拥有 Current 规则。
- [工程工具链](../../docs/tooling.md)与质量观测保持现有静态工程职责，不拥有产品 runtime budgets。
- 被发现问题对应的 CLI、navigation、adapter、cost、find、output、release 或 dependency owner。
