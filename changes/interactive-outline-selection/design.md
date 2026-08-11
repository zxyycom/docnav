# Design

该设计把 interactive outline 作为 core CLI 的 human-only orchestration，并复用现有 outline、opaque ref、read 和 readable output 语义。

## Context

Adapter 已拥有 outline/ref/read 业务语义，core CLI 拥有参数、output mode、adapter handoff 和人类输出编排。交互选择只需要把 outline display/ref 映射成终端选项，再把选中 ref 交回既有 read path。当前产品决策要求基础 contract 稳定后再扩张此 surface。

## Goals / Non-Goals

**Goals**

- 为 `docnav outline <path> --interactive` 定义第一版 TTY 多选流程。
- 复用现有 adapter selection、outline invoke、read invoke、ref 透传、错误映射和 readable renderer。
- 明确 machine output、非 TTY、取消、空选择、空 outline 与多选顺序。
- 用最小依赖和可测试 seam 支持 Windows 与常见终端。

**Non-Goals**

- 不改变 adapter protocol、ref、outline entry、read result 或 schema/example。
- 不增加 batch protocol，也不把 interactive 参数传给 adapter。
- 第一版不实现树形折叠、实时 preview、full-screen event loop、鼠标或持久布局。

## Decisions

1. Interactive workflow 属于 core CLI：先走现有 outline path，选中后逐个走现有 read path；path、explicit adapter intent、page、limit 与稳定错误映射按届时既有 operation semantics 继承，adapter 不接收 UI 参数或多选状态。
2. 第一版采用 prompt-style MultiSelect，而不是 full-screen TUI。依赖在实现门禁中比较当前 `inquire`、`dialoguer` 或等价维护良好方案后确定。
3. 每个选项把 entry display 用于人类展示，把 opaque ref 原样保存；不得从 display、location 或 hierarchy 重新构造 ref。
4. `--interactive` 与 `--output protocol-json` 互斥；interactive 是 human-only UI，不定义多个 read result 的 machine envelope。
5. stdin 或 stdout 不满足所需 TTY 条件时返回稳定 invalid request，且不能先输出 prompt 控制序列或部分 machine content。
6. 用户取消和空选择默认视为成功且不执行 read；空 outline 不启动选择器，并给出简短人类结果。
7. 多个选择按 UI 确认后的稳定顺序依次执行 read，并使用现有 readable renderer 分隔结果；若产品要求可滚动 preview，则建议把它作为独立 full-TUI Change，只有用户明确要求时才创建。
8. 本 design 是实施期间唯一承载 change-local Target 的载体，并登记以下 owner delta：在 CLI/navigation/output 中登记实际成立的 human-only 参数、TTY 和 outline-to-read orchestration；在 testing/release 中登记交互 seam、终端与 package 证据。实现期间稳定 owner 只提供 Current 基线，不提前写入 Target。只有 CLI behavior、TTY/error、selection-to-read 和 package 行为证据通过后，才把已成立的 delta 同步为 Current，并再次验证 design、owner、实现和证据一致。

## Risks / Trade-offs

- Prompt UI 不能表达真实树形结构：第一版只承诺 flat entry selection，复杂交互单独规划。
- 终端库在 CI、Windows 或重定向环境中行为不同：先做依赖 spike，抽离选择 seam，并保留本地 Windows smoke。
- 多个 read 的视觉边界可能含混：复用稳定 readable result framing，不创造 machine batch shape。
- 取消与空状态容易被当成错误：显式编码“不执行 read”的成功路径并用 call-count 证据覆盖。
- 新 CLI surface 会跟随基础 contract 变化：恢复时先重基线；实施期间只由本 design 承载 change-local Target，行为证据通过后才同步 Current owner。

## Open Questions

- 实现时选择哪一个当前受支持的 prompt-style 多选库，取决于 features、Windows 行为、维护状态和依赖体积；实现子代理可以准备 spike 与证据，由指定 architecture/dependency owner 按 `tasks.md` 的 `1.3` 关闭。
- 多个结果采用直接顺序 readable rendering 还是简化的分页查看器；本设计推荐前者，但该推荐不代替产品批准。实现子代理只准备候选与证据，必须由用户或指定 CLI/product owner 按 `tasks.md` 的 `1.4` 明确批准。若选择后者，必须先更新 scope、design 和验收条件。

`1.3` 与 `1.4` 分属不同门禁 owner：architecture/dependency owner 可以关闭依赖选择；只有用户或指定 CLI/product owner 可以关闭第一版人类 UX。实现子代理不得自行勾选 `1.4`。两项关闭前不得修改 production，但这不会把完整计划降回 Draft。
