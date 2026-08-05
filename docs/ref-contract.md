# Ref

本文是 Docnav v0 共享 ref 契约的主规范。它定义 ref 在共享层中的载体、调用流程、所有权和传递规则。每个 adapter 的私有 ref grammar、定位语义、保证范围和错误分类由对应 adapter 专属文档定义。

## 核心边界

- `path` 负责定位文档，并作为 `docnav` 选择 adapter 的依据。
- `ref` 是 adapter 生成和解释的非空 opaque string。它表达当前文档内部的一个区域、位置、查询或 adapter 私有概念。
- 共享协议、`docnav` core 和其它调用入口只在 explicit ref 输入边界校验 ref 是非空字符串，并把收到的值原样传给选定 adapter。缺失、非字符串或空字符串是入口输入错误；合法非空字符串的 grammar、定位和语义解释仍属于 adapter owner。

任何成功返回 caller-visible ref 的 adapter 行为都是 **ref producer**；当前 producer
是 `outline` 和 `find`。`read` 中负责解释 ref 并物化 adapter-defined selection 的行为是
**ref consumer**。未来能力一旦发出 ref，就同时承担 producer 契约；成本、metadata、
full-read source facts 或 rendering input 等不发出 ref 的辅助能力不成为 ref identity owner。

## 兼容文档视图

Ref 的 producer/read 一致性以兼容文档视图为边界。两个视图对某个 ref 兼容，至少满足：

1. 使用相同 adapter identity 和 ref 语义；
2. adapter 消费的 source bytes/text 与所有影响 ref 生成、解释和定位的固定配置或事实相同；
3. `read` 的定位与物化只依赖既有 `ReadInput`、opaque ref 和该视图，不依赖 producer-only 状态。

同一个 prepared view 与自身兼容；使用相同 source 和相关事实独立重新准备的视图也必须
兼容。Adapter owner 可以声明更宽的等价关系，但共享层不从相同 path 推断兼容性。Source、
相关配置或 ref 语义变化后，视图可以不兼容。

## 共享调用流程

共享 ref 契约强制保留以下调用流程和成功保证：

1. Adapter producer 在成功结果中生成完整、非空、符合自身 canonical grammar 的 ref。
2. 调用方将相同 path 和 ref 原样提交给 `read`。
3. `docnav` core 根据 path 选择 adapter，并将 ref 原样传入。
4. 在同一个 prepared view 或独立准备的兼容视图上，adapter `read` 必须接受该 ref，返回
   经过共享结果校验的 success，原样回显 ref，并物化 producer 所记录的 adapter-defined
   selection。

Core 不解析 ref，也不在 production runtime 对每个 producer entry 额外调用一次 read。
Built-in adapter 通过共享黑盒 conformance harness 证明 producer/read 契约；harness 只收集
opaque ref 并调用既有 read boundary，不重建 ref 或检查私有 grammar。

该保证不扩展为以下语义：

- ref 必须一对一定位区域；多个 ref 可以选择同一区域，同一个 ref 也可以出现在多个 find occurrence 中。
- read content 必须逐字包含 find query；adapter owner 可以定义 normalized 或 container-level correspondence。
- ref 在不兼容的文档变化、相关配置变化或 ref 语义变化后仍保持含义。
- 共享层可以根据 display、path 或其它 presentation facts 重建 ref。

## 什么是“可作为 read 字段传输”

“ref 可作为 read 字段传输”表示 ref 满足共享字段形状校验（非空字符串），并可以跨共享层
原样传递到 adapter。字段形状本身不证明 producer/read 一致性；该证明来自 adapter owner
契约和 conformance evidence。对于 adapter 已成功发出的 ref，兼容视图成功保证仍然成立。

共享字段规则不表示：

- 共享层解析或判断 adapter 私有 grammar。
- 任意 caller 自行构造的非空 ref 必须被接受。
- ref 必须唯一定位，或必须与 selection 一对一。
- 不兼容视图必须继续返回原 selection。

Adapter 保留解释 caller-supplied ref、定义 selection 和处理不兼容视图的权力；同时必须让
自身 producer 成功发出的 ref 满足兼容视图 read 契约。

## Adapter 的所有权

每个格式 adapter 自行定义并记录以下语义：

- ref grammar 和内部字段。
- ref 适用的 operation（outline、find、read 等）。
- 定位或查询含义、读取粒度和返回区域。
- 同一个 ref 是否在同一次解析结果中唯一。
- 多个 ref 是否可以指向同一区域。
- Producer evidence 与 read selection 的 correspondence。
- 哪些 source、配置和 ref facts 构成兼容视图，以及是否声明比相同 source 更宽的等价关系。
- 文档或 parser 变化后的行为。
- 非法 ref、未匹配 ref、歧义等失败如何映射到稳定错误。
- 哪些非空特殊 ref（例如全文读取 ref）由该 adapter 接受，以及它们属于 navigation behavior 还是入口输入校验。

共享文档通过链接指向 adapter 专属文档，不复制 adapter 私有语义。Markdown 的 ref grammar、结构快照语义和错误边界见 [Markdown Adapter](adapters/markdown.md)。

## 正确性责任分层

本边界是正确性责任的分层，不是放弃正确性：

| 层 | 正确性责任 |
| --- | --- |
| 共享层（`docnav` core、协议、schema） | 按 path 选择正确 adapter；保持 ref 原值不变；一致映射稳定错误；提供不解析 ref 的黑盒 conformance boundary |
| Adapter producer | 生成完整、非空、canonical ref，并记录 adapter-defined correspondence |
| Adapter read | 在相同或独立准备的兼容视图上成功解释 producer ref、原样回显 ref 并物化对应 selection；在不兼容视图上遵循本格式 stale/error 语义 |

共享层不了解 adapter grammar、文档状态和定位模型，因此不通过检查 opaque string 证明
一致性；adapter contract 和每个 built-in adapter 的 owner evidence 共同把兼容视图 disagreement
归类为实现缺陷。

## 共享 Ref 错误

共享层保留以下稳定 ref 相关错误，供 adapter 按自身契约返回：

| 错误码 | 含义 |
| --- | --- |
| `REF_INVALID` | 选定 adapter 无法按其当前私有 grammar 解释该 ref。details 包含 `ref` 和 `reason`。 |
| `REF_NOT_FOUND` | ref 格式合法但 adapter 当前未能匹配任何区域。 |
| `REF_AMBIGUOUS` | ref 格式合法但 adapter 匹配到多个区域。 |

共享层不要求每个 adapter 必须产生上述全部错误。每个 adapter 在其专属文档中说明实际可能返回的错误及其边界。`REF_AMBIGUOUS` 保留为可用稳定错误，供能够检测多重匹配的 adapter 使用。

对于一个由同一 adapter producer 成功发出、随后在兼容视图和合法 read input 上读取的 ref，
因 producer/consumer disagreement 返回 `REF_INVALID`、`REF_NOT_FOUND`、`REF_AMBIGUOUS` 或其它
failure 是 adapter contract defect。视图不兼容时，adapter 仍可按专属契约返回这些错误或解析
到当前 selection。

## 格式定位所有权

共享协议、`docnav` 和调用入口不得解析、拼接、规范化或从 display 推断 ref，只能把 adapter 返回的 ref 原样传递给 read。adapter 可以在没有局部导航区域时为文档定义私有全文 ref；该 ref 的具体拼写和语义属于对应 adapter。
