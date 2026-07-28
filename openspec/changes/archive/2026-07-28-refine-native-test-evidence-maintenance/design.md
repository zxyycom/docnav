本 change 的技术设计把普通 Bun 测试面改为目录匹配，并让 Entry 拆分与 Claim 保留分别服从失败归因和长期信息增量；本文是临时设计，不表示实现已生效。

## Context

当前 project wrapper 已能把 Cargo、Bun 和 core smoke 的静态声明、runner report 与 committed inventory 双向闭合，但 Bun profile 仍逐文件列举普通 `.test.ts`。该集合内部可以闭合，却不能发现“新增文件同时未进入 profile”的遗漏。现有 17 个 smoke leaf 全部拥有 Claim，且所有 Claim 都只引用一个 Entry，说明旧大 case 的语义分组仍部分控制新账本。

本 change 只改变开发验证链路和测试组织。产品 CLI、protocol、adapter contract 与 release package 均不受影响。实现以 `enforce-native-test-evidence-coverage` 的当前代码和文档为基线；后续归档必须先归档该基线 change。

## Goals / Non-Goals

**Goals:**

- 普通 Bun 测试文件由受控目录和 include/ignore patterns 自动纳入，不依赖逐文件登记。
- 单文件声明只补充目录规则之外的特殊测试面，并能检测冗余补充。
- Entry 对应能够独立选择、独立报告且拥有单一可归因意图的 runner leaf。
- Claim 数量和关联形态由长期信息增量决定，允许一个 Claim 关联多个 Entry。
- profile、任务 selector、command audit label、inventory 和 Claim 之间保持可追踪一致。

**Non-Goals:**

- 不建立通用 glob library、动态 runner plugin 或任意仓库发现框架。
- 不把每个断言、命令步骤、输入变体或 helper 拆成 Entry。
- 不追求一次重写全部 536 个测试，只处理当前可明确判断的混合意图和 Claim 偏差。
- 不兼容旧 profile shape、旧 smoke entryKey 或旧派生 inventory。

## Decisions

### Decision 1: Bun profile 使用 source roots、include、ignore 与 supplemental files

`bun` profile 使用以下 v2 形态：

```json
{
  "sourceRoots": ["scripts", "test"],
  "include": ["**/*.test.ts"],
  "ignore": [],
  "supplementalFiles": []
}
```

`include` 与 `ignore` 都是相对每个 source root 的正向 POSIX glob，不接受
minimatch 的 `!` negation 或 `#` comment 控制语法；先取所有 include match，再
移除 ignore match，最后合并 workspace-relative `supplementalFiles`。展开结果
排序去重后同时交给 ast-grep 与 Bun runner，二者不得各自重新发现文件。

选择该方案而不是逐文件清单，是为了让新增普通测试自动进入闭合。选择显式 `supplementalFiles` 而不是 glob negation，是为了让特殊补充具有单一、可审计的责任；已被目录规则纳入的补充文件视为冗余并阻断。

### Decision 2: profile 展开是显式边界

source root、pattern 和 supplemental path 必须是安全的相对 POSIX 值。配置路径
必须词法位于当前 checkout 内且各级不得经过符号链接；source root 必须是普通目录，
supplemental file 必须是普通文件。每个 include pattern 至少匹配一个文件，最终
集合不得为空。

选择阻断而不是静默忽略，是为了让 typo、目录迁移和测试面收缩可见。ignore pattern 可以暂时无匹配，因为它也可表达对未来生成目录或平台文件的稳定排除。

### Decision 3: Entry 按独立失败意图拆分，不按命令数量拆分

当一个 runner leaf 同时包含不同 owner requirement、不同外部命令族或可以独立定位的成功/失败边界时拆分。多个步骤共同构成一次 round trip、同一输出 parity 或同一 source precedence matrix 时可以保留在一个 leaf。

选择语义归因而不是固定行数、命令数或 assertion 数阈值，是为了避免把端到端场景拆成失去证明意义的微型测试。

### Decision 4: Claim 与 Entry 使用稀疏多对多关系

普通 Entry 不建立 Claim。一个长期判断需要多个代表入口共同证明时，保留一个 Claim 并列出全部当前 `entryKey`；Claim 不再为了继承旧 case ID 而与 smoke leaf 一一对应。statement 与 observations 的范围不得超过 `supportedBy` 能直接观察的结果。

选择删除低信息 Claim 而不是重写成更长说明，是为了减少 AI 后续审查时的重复事实和错误权威感。

### Decision 5: 直接重建派生身份

profile 版本升级、task split 和 rename 会重建 inventory/index，并按语义连续性更新保留 Claim 的 `supportedBy`。不提供旧 Entry alias、双 inventory 或兼容查询。

该选择符合当前项目不要求旧测试账本兼容的约束，并避免把一次性迁移身份变成长期 contract。

### Decision 6: Sync 不以旧 inventory 新鲜为运行前提

runner 会执行测试证据工具自身的 Bun tests，因此这些自测可以校验 committed
profile、inventory、Claim 和 index 的独立 schema shape，但不得在 runner report
产生前要求旧 inventory 已经与当前源码语义闭合。源码变化后的语义新鲜度由 sync
重建派生制品后紧接的 strict check 证明。

选择这一边界是为了避免自举死锁：若 runner 自测先要求旧 inventory 新鲜，任何
测试 rename、split 或 fingerprint 变化都会同时阻止负责重建 inventory 的 sync。

### Decision 7: Smoke fingerprint 跟随 Entry 自有实现

Smoke Entry 的 source fingerprint 包含 leaf task 声明、`run` 绑定，以及从该绑定
可达且位于 smoke `sourceRoots` 内的顶层实现声明。相对导入位于 roots 外时只记录
依赖绑定，不递归吸收共享 fixture、harness 或 assertion 的实现；同一模块中不可达
的声明不进入 fingerprint。

选择受控实现图而不是只 hash task object，是为了让 case helper 的真实行为变化
产生 `implementation-changed`；选择不 hash 整个模块或全部共享依赖，是为了避免
无关 helper 与公共测试框架变化让大量 Entry 同时陈旧。

## Risks / Trade-offs

- [Pattern 过宽会纳入非目标测试] → source roots 保持受控，profile schema 校验 pattern，展开测试覆盖 include/ignore/supplement 顺序。
- [Pattern 过窄会造成遗漏] → 普通约定使用目录级 `**/*.test.ts`，每个 include 必须有匹配，新增文件自动进入 runner。
- [split 增加运行调度开销] → 只拆独立意图；smoke 仍复用现有并发 harness 和构建产物。
- [Claim 删除降低自然语言说明量] → Entry/source/owner 仍可查询；只删除 owner 加入口名称已经能恢复的重复说明。
- [两个 active change 修改同一 capability] → 当前实现以已完成基线为前置；归档顺序固定为先基线、后本 change，并在 verification 中记录。
- [工具自测依赖旧派生状态会阻止重建] → schema shape 自测与语义新鲜度检查分层，sync 后立即运行 strict check。
- [Smoke 实现移动到 helper 后变化不可见] → fingerprint 跟随 source roots 内可达的顶层声明，并用无关声明不扰动的回归测试约束归因边界。

## Migration Plan

1. 先用失败测试固定 profile v2 展开、ignore、supplement 和冗余补充行为。
2. 实现 profile parser/schema 与单一 Bun 文件展开函数，切换静态和 runtime 两侧。
3. 将现有 profile 转为目录规则，确认展开集合仍覆盖当前全部 Bun tests。
4. 拆分明确混合的 smoke/Rust leaf，修正 command label。
5. 删除或重新关联 Claim，重建 inventory/index。
6. 运行目标测试、逐 leaf smoke、严格 test-evidence check 和完整 workspace verification。

回滚可整体恢复 profile v1、测试 leaf 和旧 inventory/index；本 change 不产生产品数据迁移或远端状态。

## Open Questions

无未回答开放问题，可以进入实现前审计。
