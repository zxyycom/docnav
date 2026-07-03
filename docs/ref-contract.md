# Ref

本文是 Docnav v0 共享 ref 契约的主规范。它定义 ref 在共享层中的载体、调用流程、所有权和传递规则。每个 adapter 的私有 ref grammar、定位语义、保证范围和错误分类由对应 adapter 专属文档定义。

## 核心边界

- `path` 负责定位文档，并作为 `docnav` 选择 adapter 的依据。
- `ref` 是 adapter 生成和解释的非空 opaque string。它表达当前文档内部的一个区域、位置、查询或 adapter 私有概念。
- 共享协议、`docnav` core 和其它调用入口只在 explicit ref 输入边界校验 ref 是非空字符串，并把收到的值原样传给选定 adapter。缺失、非字符串或空字符串是入口输入错误；合法非空字符串的 grammar、定位和语义解释仍属于 adapter owner。

## 共享调用流程

共享 ref 契约强制保留以下调用流程：

1. Adapter 在 `outline` 或 `find` 中生成 ref。
2. 调用方将相同 path 和 ref 原样提交给 `read`。
3. `docnav` core 根据 path 选择 adapter，并将 ref 原样传入。
4. Adapter 按其私有契约解释或拒绝 ref，返回读取结果或稳定错误。

该流程保证属于共享层：core 负责按 path 选择 adapter、保持 ref 原值并一致映射稳定错误。流程保证**不**扩展为以下语义：

- ref 一定被 `read` 接受。
- ref 一定唯一定位一个区域。
- `read` 一定成功读取。
- ref 在文档变化后仍保持含义。
- 多个 ref 不能指向同一区域。

## 什么是“可作为 read 字段传输”

“ref 可作为 read 字段传输”只表示 ref 满足共享字段形状校验（非空字符串），并可以跨共享层原样传递到 adapter。**不**表示：

- 共享层保证 adapter 接受该 ref 值。
- 共享层保证 adapter 完整消费该 ref。
- 共享层保证 ref 唯一定位。
- 共享层保证 read 成功返回。

Adapter 保留接受、拒绝和解释 ref 的全部权力，并在专属契约中定义结果语义。

## Adapter 的所有权

每个格式 adapter 自行定义并记录以下语义：

- ref grammar 和内部字段。
- ref 适用的 operation（outline、find、read 等）。
- 定位或查询含义、读取粒度和返回区域。
- 同一个 ref 是否在同一次解析结果中唯一。
- 多个 ref 是否可以指向同一区域。
- 文档或 parser 变化后的行为。
- 非法 ref、未匹配 ref、歧义等失败如何映射到稳定错误。
- 哪些非空特殊 ref（例如全文读取 ref）由该 adapter 接受，以及它们属于 navigation behavior 还是入口输入校验。

共享文档通过链接指向 adapter 专属文档，不复制 adapter 私有语义。Markdown 的 ref grammar、结构快照语义和错误边界见 [Markdown Adapter](adapters/markdown.md)。

## 正确性责任分层

本边界是正确性责任的分层，不是放弃正确性：

| 层 | 正确性责任 |
| --- | --- |
| 共享层（`docnav` core、协议、schema） | 按 path 选择正确 adapter；保持 ref 原值不变；一致映射稳定错误 |
| Adapter | ref 生成、解释、定位和失败行为符合自身公开契约 |

共享层不了解 adapter grammar、文档状态和定位模型，因此既不负责也不应替 adapter 保证 ref 一定被 `read` 接受或成功读取。

## 共享 Ref 错误

共享层保留以下稳定 ref 相关错误，供 adapter 按自身契约返回：

| 错误码 | 含义 |
| --- | --- |
| `REF_INVALID` | 选定 adapter 无法按其当前私有 grammar 解释该 ref。details 包含 `ref` 和 `reason`。 |
| `REF_NOT_FOUND` | ref 格式合法但 adapter 当前未能匹配任何区域。 |
| `REF_AMBIGUOUS` | ref 格式合法但 adapter 匹配到多个区域。 |

共享层不要求每个 adapter 必须产生上述全部错误。每个 adapter 在其专属文档中说明实际可能返回的错误及其边界。`REF_AMBIGUOUS` 保留为可用稳定错误，供能够检测多重匹配的 adapter 使用。

## 格式定位所有权

共享协议、`docnav` 和调用入口不得解析、拼接、规范化或从 display 推断 ref，只能把 adapter 返回的 ref 原样传递给 read。adapter 可以在没有局部导航区域时为文档定义私有全文 ref；该 ref 的具体拼写和语义属于对应 adapter。
