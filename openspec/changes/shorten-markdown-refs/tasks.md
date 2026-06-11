本任务清单在 change 审核通过后执行。

## 1. 契约确认

- [ ] 1.1 确认 Markdown heading ref 为 `H:L{line}:H{level}:I{index}`，其中 `L`、第二个 `H` 和 `I` 分别标识 line、heading level 和全文 index；三个数字均使用不带前导零的十进制正整数。
- [ ] 1.2 确认 `index` 基于全文有效 headings，并在可见性过滤前分配。
- [ ] 1.3 确认 `doc:full` 仅由 Markdown adapter 定义，用于整篇文档读取。
- [ ] 1.4 确认共享 ref 契约只包含 adapter 所有权和不透明值原样传递。

## 2. Markdown 实现

- [ ] 2.1 生成 `H:L{line}:H{level}:I{index}` heading ref。
- [ ] 2.2 在 Markdown `read` 中解析 canonical 标记格式，并精确匹配 line、level 和 index；无匹配返回 `REF_NOT_FOUND`。
- [ ] 2.3 让 outline 和 find 复用同一 heading ref 生成逻辑。
- [ ] 2.4 保留 `doc:full` 的生成和读取行为。
- [ ] 2.5 仅生成和接受新 heading ref；旧 `L{line}:{path}`、`L{line}#{ordinal}:{path}` 和显式 `L{line}#1:{path}` 返回稳定 ref 错误。

## 3. 文档与契约材料

- [ ] 3.1 新增 `docs/adapters/markdown.md`，记录 Markdown 的 outline、read、find、ref、整篇文档读取行为、保证范围和验证入口。
- [ ] 3.2 更新 `docs/navigation.md`，将 Markdown adapter 实现与审计任务指向该专页。
- [ ] 3.3 将 `docs/refs.md` 收敛为共享所有权和传递边界，并链接到 adapter 专页。
- [ ] 3.4 同步更新架构、adapter contract、测试策略、编码规范和 README 中的全局 ref 表述。
- [ ] 3.5 更新 Markdown spec、示例、fixture 和 golden output；保留 Markdown 私有 `doc:full`。
- [ ] 3.6 全仓检查旧 heading ref，仅在迁移说明和负向测试中保留旧格式。
- [ ] 3.7 确认 protocol/readable schema 继续将 ref 校验为非空字符串，不加入 Markdown 私有字段。

## 4. Tests

- [ ] 4.1 覆盖 `L`、`H`、`I` 字段标识、canonical 数字表示和字段不匹配时的 `REF_NOT_FOUND`。
- [ ] 4.2 覆盖不同 `max_heading_level` 下同一 heading 的 ref 一致性。
- [ ] 4.3 覆盖重复 heading，并验证同一次解析结果中的 ref 唯一性。
- [ ] 4.4 覆盖极长、深层和 Unicode heading，验证 ref 不包含标题、breadcrumb 或摘要。
- [ ] 4.5 覆盖 `outline -> ref -> read`、`find -> ref -> read` 和 `doc:full -> read`。
- [ ] 4.6 覆盖旧 heading ref 和非法数字格式的稳定错误。

## 5. Verification

- [ ] 5.1 运行 Markdown adapter Rust 测试和 CLI smoke。
- [ ] 5.2 运行 docs、schema 和 example 验证。
- [ ] 5.3 运行 `pnpm run verify:docnav-workspace`。
- [ ] 5.4 检查局部 diff，确认改动限于 Markdown 私有行为和必要的共享文档收敛。
