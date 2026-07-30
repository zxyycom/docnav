**本文是 `audit-runtime-performance-boundaries` 的全未执行任务清单：它先阻断式审计 artifacts，再测量和归因，最后等待人工决定；它不预先创建任何具体优化任务。**

## 1. Blocking artifact audit

- [ ] 1.1 先创建 change-local `audit-report.md`；首句声明它是本 change 的 runtime performance artifact audit、workload、measurement、attribution、human decision 与 owner handoff 记录，并建立同名用途章节。它不是 README、通用项目说明或已批准 baseline。
- [ ] 1.2 在 `audit-report.md` 记录阻断式审计结论：proposal、design、`specs/runtime-performance/spec.md` 和 tasks 均围绕首句目标；capability ID 为 `runtime-performance` 且与 spec 目录一致；数字/artifacts 未冒充 Current baseline、budget/gate 或可直接实施方案；没有未回答的 artifact 歧义、范围外 main spec/docs/schema/example/code/test/其它 change 修改、quality/tooling owner 越权，或对四个相关 change 的依赖。
- [ ] 1.3 对四类 artifacts 运行 OpenSpec status/strict validation，对 change 内全部 Markdown（含 `audit-report.md`）运行 `dnm outline` 和至少一次对应 `dnm read`，并做 scoped diff 与 whitespace 检查；把命令和结果记录到 `audit-report.md`。任务 1.1–1.3 全部完成前，禁止执行任何后续任务。

## 2. Workload records and reproducible baseline evidence

- [ ] 2.1 在任何测量前，按 design Decision 2 把有限 initial workload packet、每个 required cell、fixture/format 选择规则、unavailable/not-applicable 条件和停止规则写入既有 `audit-report.md`；未选 format × operation × output × page × stress 组合统一标为 `unmeasured/future`，不得扩展为笛卡尔积。
- [ ] 2.2 从 Current release/source evidence 和已批准 dependencies 恢复被测 binary、build 与 process boundary，为 packet 分别固定一个 startup cell 和一个 package cell、primary format 的 outline/find/ref-derived read 与一个 later-page cell、至多一个 secondary-format outline cell，以及三层 scale、find miss、root read、长 key/label/ref 和适用时 retained-memory lifecycle 的 stress cells。
- [ ] 2.3 按 design Decision 3 只对 task 2.1 的 required cells 记录完整 command、flags、fixture、output/page/limit/query/ref、build、host、cache、repeats、测量定义和原始样本，并测量适用的 wall time/CPU、I/O/准备次数、peak/retained memory、stdout/stderr/page、package size 与伸缩结果；每个 cell 完成或有证据地标为 unavailable/not-applicable 后停止首轮测量。
- [ ] 2.4 在相同记录规则下尝试复现 design Decision 7 的 JSON、超长 key/output 和旧 tokenizer seed observations；无法恢复完整条件或无法复现的数字继续标记为 seed observation。受未批准依赖影响的 `5 MiB` / BPE 工作树数据不得升级为 Current 或 release baseline。

## 3. Attribution and report

- [ ] 3.1 使用 design Decision 4 的 categories 对每个 material observation 做最小充分归因，保存比较、profile、instrumentation、计数或排除证据，并把不能证明的部分保留为 `unattributed`。
- [ ] 3.2 在 `audit-report.md` 中分别总结 representative 与 stress/adversarial 结果、输入伸缩、输出/分页、package、peak/retained memory、测量噪声、未知项和不可比条件；不得从单次 wall time、RSS 或 bytes 直接推荐机制。
- [ ] 3.3 为经证据支持的 finding 写出对应 adapter、core/navigation、token-cost、document-state、find、protocol/output 或 release/dependency owner handoff；报告只描述问题与验收 workload，不在本 change 选择或实现 owner-specific 修复。
- [ ] 3.4 审计报告覆盖矩阵中的未测、unknown 和 not-applicable 均已显式记录，并确认正常 JSON 接受项仍是非阻断 observation、stress/adversarial 风险仍保留待决。

## 4. Human workload, budget, and gate decisions

- [ ] 4.1 向人类提交完整 workload/measurement/attribution packet，由人类逐项批准或拒绝哪些 representative 与 stress/adversarial records 可以成为 reproducible baselines；未获批准的记录保持 observation。
- [ ] 4.2 由人类明确决定是否需要任何数字 budget；每个获批 budget 必须同时固定 workload、指标、值或范围、统计口径、build、host、cache、噪声和复核条件。没有明确批准时记录“无 budget”，不得推导阈值。
- [ ] 4.3 由人类与 budget 分开决定是否需要 blocking gate；只有 enforcement owner、执行入口、失败语义和更新/移除流程均获批准后才可创建 gate 工作。默认决定为非阻断。
- [ ] 4.4 由人类审阅 owner handoffs 并明确授权哪些后续 owner change 可以创建优化任务。任务 4.1–4.4 完成前，不得创建或实施任何具体优化任务，也不得跨 change 写入。

## 5. Approved handoff and capability finalization

- [ ] 5.1 仅对任务 4.4 明确批准的 finding，在相应 owner change 或新 owner-specific change 中创建带 before/after workload 验收的优化任务；本 change 不承接优化实现，未批准 finding 不创建任务。
- [ ] 5.2 在获得 apply/archive 授权后，创建并同步 `docs/runtime-performance.md` 的稳定正文、`runtime-performance` main spec，以及 `docs/navigation.md` 中“建立或解释 runtime 性能 baseline、budget、audit 或 optimization 时读取”的阅读路径和规则 owner 映射；现有 tooling/quality owner 保持原职责，四个相关 change 保持独立。
- [ ] 5.3 对最终 change 和获准新增的 owner 文档运行 strict validation、全部 Markdown 的 `dnm outline/read`、链接/导航、scoped diff/whitespace 检查；若后续 owner change 改变 public protocol 或 invocation behavior，则由该 owner 运行其自动化 contract/workspace verification，本 change 不用报告替代行为证据。
