# refine-adapter-owned-ref-contract

当前 change 已通过设计审计并解除实施门禁；其 artifacts 尚未应用到现行主规范或实现。

本 change 明确 adapter-owned ref 的共享契约边界，并以 Markdown adapter 作为首个具体落点：共享层强制保留 `outline/find -> ref -> read` 调用流程，core、协议和 MCP 只承载并原样传递非空 opaque ref；ref 的 grammar、有效条件、唯一性、消歧、读取结果和错误分类由对应 adapter 的专属契约定义。

这是正确性责任的分层，不是放弃正确性。共享层负责 adapter 选择、ref 原样传递和稳定错误映射；adapter 负责其生成、解释、定位和错误行为符合自身契约。共享层不承诺 `read` 必然成功、唯一定位或返回特定区域。

Markdown adapter 提议使用不包含标题内容的 `H:L{line}:H{level}:I{index}` heading ref，通过三字段精确匹配解释当前结构坐标，并用 `REF_INVALID` 区分非法 grammar 与合法但未匹配的 `REF_NOT_FOUND`。outline display 承载标题或 breadcrumb；find display 保留匹配位置附近的文本片段，并可补充 heading 导航语义。

以上属于本 change 的已确认决策。后续审查应验证 artifacts、实现、测试和错误映射是否落实这些决策，不因通用 API 偏好、旧 ref 兼容偏好、内容身份偏好或要求共享层提供更强定位保证而重复开启讨论。只有发现现行主规范的实质冲突、明确不可实现条件、可复现契约缺陷，或用户明确修改决策时，才重新评估对应事项。
