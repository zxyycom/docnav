本任务清单用于实施 adapter-owned ref 共享契约、Markdown 结构 ref 和 `REF_INVALID` 错误边界。设计审计已经通过，实施门禁已经解除；后续按“契约确认 -> 文档先行 -> 并行实现/测试准备 -> 集成验证”的顺序执行。

## 1. 阻塞级审计门禁（已完成）

- [x] 1.1 审计 proposal、design、spec delta 和 tasks 是否共同落实已确认决策：共享层强制 `outline/find -> ref -> read` 流程并原样传递非空 opaque ref；解释结果归 adapter；正确性责任按所有权分层；Markdown 使用 `H:L{line}:H{level}:I{index}`、结构快照语义和三字段精确匹配；outline display 承载 heading 导航语义，find display 保留匹配片段；非法 grammar 使用 `REF_INVALID`；旧 heading ref 不兼容读取。
- [x] 1.2 确认审计范围仅包含 `openspec/changes/refine-adapter-owned-ref-contract/` 下的 change artifacts，审计完成前未修改现行主规范、schema、示例或实现。

## 2. 执行切片与并行边界

- [x] 2.1 将文档先行作为第一个未完成交付 slice：先落地共享 ref 契约收敛、Markdown adapter 专页和导航入口，再开始实现代码、schema/example 生成或 golden output 更新。
- [x] 2.2 每个并行 worker 只领取一个 slice，并在开始前写清 owned files、read-only context、guard / verification 和 stop conditions；不得同时修改其它 worker 的 owned files。
- [x] 2.3 文档先行 slice 完成后，可并行推进三个 lane：稳定错误与共享协议、Markdown adapter 实现、测试用例与 fixture 准备；跨 lane 的 schema/example/golden output 同步留给集成 slice 收口。
- [x] 2.4 保持串行门禁：契约确认完成后再改主文档；主文档完成并自检后再合并代码行为；所有 lane 完成后再运行 workspace 级验证。
- [x] 2.5 每个 slice 完成时记录 changed files、evidence、contracts touched、known limits 和 next slice，方便并行 worker handoff。

## 3. 契约确认

- [x] 3.1 确认共享 ref 契约强制 `outline/find -> ref -> read` 调用流程，并要求 adapter 所有的非空 opaque string、共享字段 shape 校验和 core/MCP 原样传递；流程保证不扩展为唯一性、稳定性、消歧、完整消费或读取成功保证。
- [x] 3.2 确认每个 adapter 自行定义 ref grammar、适用 operation、定位或查询语义、保证范围、文档变化行为和错误分类。
- [x] 3.3 确认正确性责任按所有权分层：共享层负责 adapter 选择、ref 原样传递和稳定错误映射；adapter 负责其生成、解释、定位和失败行为符合自身契约。
- [x] 3.4 确认 Markdown heading ref 为 `H:L{line}:H{level}:I{index}`，三个数字使用不带前导零的十进制正整数，`index` 基于全文有效 headings 并在可见性过滤前分配。
- [x] 3.5 确认 Markdown ref 是结构快照，不提供 title、section 内容或文档版本身份；文档变化后旧 ref 可以失效、继续匹配或匹配当前结构中的其它 heading。
- [x] 3.6 确认 `REF_INVALID` 表示 adapter 无法解释非空 ref grammar，稳定 details 为 `ref` 和 `reason`；符合 Markdown canonical grammar 但当前无匹配时使用 `REF_NOT_FOUND`。
- [x] 3.7 确认 `doc:full` 仅由 Markdown adapter 定义，用于整篇文档读取。
- [x] 3.8 确认 outline display 承载标题或 breadcrumb，find display 保留匹配位置附近的文本片段并可补充 heading 导航语义；超长 display 可以截断，完整 ref 不受影响。
- [x] 3.9 将 proposal 和 design 的 Confirmed Decisions 作为后续评审边界；仅在现行主规范实质冲突、明确不可实现条件、可复现契约缺陷或用户明确修改决策时重新开启对应事项。

## 4. 文档与契约材料（先行）

- [x] 4.1 新增 `docs/adapters/markdown.md`，记录 Markdown 的 outline、read、find、ref grammar、结构快照语义、全文读取、错误分类、保证范围和验证入口。
- [x] 4.2 更新 `docs/navigation.md`，将 Markdown adapter 实现与审计任务指向该专页。
- [x] 4.3 将 `docs/refs.md` 收敛为共享最小契约，并明确”可作为 read 字段传输”不等于 adapter 必须接受、完整消费、唯一定位或成功读取。
- [x] 4.4 同步更新架构、adapter contract、测试策略、编码规范和 README 中的全局 ref 表述，移除共享唯一性、消歧和文档变化失败保证。
- [x] 4.5 更新 Markdown 主文档中的 outline/find/read/ref/error 表述；保留 Markdown 私有 `doc:full`。
- [x] 4.6 更新 Markdown outline/find 文档，明确 ref 负责 adapter 私有解释；outline display 提供 heading 导航，find display 保留匹配片段；display 截断不影响 ref。
- [x] 4.7 全仓检查旧 heading ref 的文档引用，仅在 breaking migration 说明和非法 grammar 测试说明中保留旧格式。
- [x] 4.8 确认 protocol/readable ref 字段文档继续只要求非空字符串，不加入 Markdown 私有字段或 grammar。
- [x] 4.9 文档 slice 完成后运行最窄文档验证或局部 diff review，确认主文档已经足以指导后续实现 lane。

## 5. 稳定错误与共享协议

- [x] 5.1 在稳定错误定义和生成规则中增加 `REF_INVALID`，details 固定为 `ref` 和 `reason`。
- [x] 5.2 更新 protocol response、readable error schema、错误规则和生成产物，使 `REF_INVALID` 可在 raw、readable、CLI 和 MCP 错误映射中保持一致。
- [x] 5.3 新增 `REF_INVALID` protocol 和 readable 示例，并验证共享层只传递 ref、不解析 adapter grammar。
- [x] 5.4 保留 `REF_NOT_FOUND` 和 `REF_AMBIGUOUS` 作为可用稳定错误，但不在共享 ref 契约中要求每个 adapter 必须产生歧义或唯一定位语义。

## 6. Markdown 实现

- [x] 6.1 生成 `H:L{line}:H{level}:I{index}` heading ref，并让 outline 和 find 复用同一生成逻辑。
- [x] 6.2 在 Markdown `read` 中解析 canonical heading grammar，并精确匹配当前解析结果中的 line、level 和 index。
- [x] 6.3 将非 canonical 且不属于其它 Markdown 合法 ref 的输入映射为 `REF_INVALID`，将 canonical 但当前无匹配的输入映射为 `REF_NOT_FOUND`。
- [x] 6.4 保留 `doc:full` 的生成和读取行为。
- [x] 6.5 删除旧 heading ref 的专属兼容判断；旧格式仅作为非 canonical 输入进入统一 `REF_INVALID` 路径。
- [x] 6.6 更新 Markdown display：outline 包含可截断的 title 或 breadcrumb，并保留现有 level、cost 等必要摘要；find 保留匹配位置附近的文本片段，并可补充 heading 导航语义。

## 7. Tests、示例与 fixtures

- [x] 7.1 覆盖 `L`、`H`、`I` 字段标识、canonical 数字表示、前导零、缺失字段和未知 ref 类型。
- [x] 7.2 覆盖不同 `max_heading_level` 下同一 heading 的 ref 一致性。
- [x] 7.3 覆盖重复 heading，并验证 Markdown 当前解析结果中的三字段 ref 唯一性，同时避免把该行为断言为共享 adapter 契约。
- [x] 7.4 覆盖极长、深层和 Unicode heading，验证 ref 不包含 title、breadcrumb 或摘要，且长度只受结构数字位数影响。
- [x] 7.5 覆盖 `outline -> ref -> read`、`find -> ref -> read` 和 `doc:full -> read` 的 Markdown 私有行为。
- [x] 7.6 覆盖非法 grammar 返回 `REF_INVALID`，并断言 details 包含 `ref` 和 `reason`；旧格式只作为其中的输入样本。
- [x] 7.7 覆盖 canonical grammar 无当前匹配返回 `REF_NOT_FOUND`，证明其与 `REF_INVALID` 的边界。
- [x] 7.8 覆盖文档变化后的结构快照行为：read 只检查当前 line、level 和 index，不使用旧 title、breadcrumb 或 section 内容校验身份。
- [x] 7.9 覆盖 `REF_INVALID` 在 adapter direct CLI、invoke protocol、core CLI readable/protocol 输出和 MCP 映射中的一致错误 code 与 details。
- [x] 7.10 覆盖 outline display 包含非空 title 或 breadcrumb，find display 保留非空匹配片段并可补充 heading 导航；超长 Unicode display 按预算截断时包含显式截断标记，ref 保持完整且分页能够前进。
- [x] 7.11 覆盖共享调用链：outline/find ref 可原样提交给 read，core/MCP 不解析 ref，adapter 返回读取结果或稳定错误；测试不得把流程保证误写为共享读取成功或唯一定位保证。
- [x] 7.12 同步更新 Markdown spec 示例、protocol/readable 示例、fixture 和 golden output；该任务在稳定错误与 Markdown 实现 lane 都完成后执行。

## 8. Verification

- [x] 8.1 运行 Markdown adapter Rust 测试和 CLI smoke。
- [x] 8.2 运行稳定错误生成、docs、schema 和 example 验证。
- [x] 8.3 运行 core CLI 与 MCP 边界验证，确认 ref 仍被原样传递且共享层不解释 Markdown grammar。
- [x] 8.4 运行 `pnpm run verify:docnav-workspace`。
- [x] 8.5 检查局部 diff，确认改动限于 Markdown 私有行为、共享 ref 契约收敛和 `REF_INVALID` 必要映射。
