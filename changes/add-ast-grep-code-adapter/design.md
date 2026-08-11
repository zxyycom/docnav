# Design

该设计把 ast-grep 限制在一个 linked adapter 的私有解析边界内，并复用现有 Docnav routing、operation、ref 和输出契约。

## Context

当前 core 通过 manifest pathname hints 在目标文档 I/O 前选择 linked adapter，随后为 normalized document path 创建 invocation-private `AdapterDocument`。Core 只传递 opaque ref；adapter 拥有格式解析、导航、ref 和分页。产品决策要求本 Change 暂停，但没有否定已完成的技术计划。

恢复实施时必须从届时 Current owner 重审版本、registry、routing、schema/example 和 release 基线；以下设计不得覆盖后来形成的稳定契约。

## Goals / Non-Goals

**Goals**

- 在同一 `docnav` 进程内解析 Rust、TypeScript/TSX、JavaScript/JSX 和 Python。
- 把 ast-grep outline 立即转换为 Docnav 拥有的确定性 entry、opaque ref 和 operation result。
- 保持 static registry、closed input、`outline -> ref -> read`、protocol/readable 分层和稳定错误映射。
- 对空文件、无符号文件和可恢复语法错误保留有界读取路径。

**Non-Goals**

- 不发现或调用外部 ast-grep CLI，不接受调用方 YAML、pattern、language injection 或 parser plugin。
- 不提供跨文件语义、增量更新或 edit-stable identity。
- 不新增 CLI flag、config、protocol field、operation、output mode、通用 engine trait 或 shared parser crate。

## Decisions

1. `docnav-code` 直接链接并锁定相互兼容的 `ast-grep-core`、`ast-grep-language` 和 `ast-grep-outline`；版本和 features 在恢复时按 Current Rust/toolchain 重新确认。
2. 一个 definition 承载五个 format，并使用下面的 closed pathname/content-type mapping；manifest hints 与 adapter-private mapping 必须由同一 package 事实生成，或由 contract tests 保持一致：

   | Format | Pathname suffix | Content type |
   | --- | --- | --- |
   | `rust` | `.rs` | `text/x-rust` |
   | `typescript` | `.ts`、`.mts`、`.cts` | `text/typescript` |
   | `tsx` | `.tsx` | `text/typescript` |
   | `javascript` | `.js`、`.jsx`、`.mjs`、`.cjs` | `text/javascript` |
   | `python` | `.py`、`.pyi` | `text/x-python` |

   Core automatic routing 只消费 manifest pathname hints；显式选择只跳过 automatic routing，不跳过 selected document 对 pathname mapping 的验证。未知 mapping 不做 content sniffing。
3. 关闭全语言默认 features，只编译 Rust、JavaScript、TypeScript/TSX 和 Python parser；新增语言需独立更新 format、fixture、feature 和 release 证据。
4. Ast-grep 借用类型立即转换为 owned `CodeSymbol`；依赖类型、rule id、AST kind 和原始错误不进入共享 crate、protocol 或 ref。
5. Outline 先按 source start、source end、kind 和 name 排序 top-level items，再把按 source range 排序的 direct members 放在各自 parent 后；相同 ref 只保留排序后的第一项。公开 entry mapping 固定为：
   - `label` 使用 trim 后的 symbol name；空 name 使用 `<kind>@<1-based-line>`。
   - `kind` 使用 adapter-owned snake_case category；import 固定为 `import`，未知 future category 映射为 `other`。
   - `location` 从 UTF-8 byte range 映射为 one-based inclusive line range；exclusive end 取其前一个 byte 所在行，empty range 的首尾行相同。
   - `summary` 折叠 signature 的连续 whitespace，最多 240 个 Unicode scalar（包括截断符 `…`）；空值或与 label 相同则省略。
   - item metadata 包含 `role: "item"` 与 `exported`；member metadata 包含 `role: "member"`、`public` 与 `parent_ref`。

   分页作用于完整确定性 sequence。没有 symbol 时返回唯一、覆盖完整文件的 `kind: "file"` fallback entry；空文件允许 `0..0` range。
6. Ref 使用 `code:v1:<format>:<start-byte>:<end-byte>:<sha256>`：offset 是无多余前导零的十进制 UTF-8 byte offset，digest 是所选原始 bytes 的 64 位小写 SHA-256。Prefix、version、字段、非 canonical 数字、hash 或未知 format 的语法错误映射 `REF_INVALID`；grammar 合法但 format 与当前文档不一致、range 越界、offset 不是 UTF-8 boundary 或 digest 失配时映射 `REF_NOT_FOUND`。该 grammar 不产生 `REF_AMBIGUOUS`，也不承诺源码修改后的稳定性。
7. Read 返回 ref 对应的原始 UTF-8 source slice，不格式化或补上下文；cost 针对分页前完整 slice 计算，content 使用现有 Unicode-safe text pagination。Find 在完整 normalized symbol sequence 的原始 name/signature 上做大小写敏感 literal search，一个 symbol 最多返回一次并保持 outline 顺序；pattern-like 字符没有特殊语义。
8. Info 只公开稳定文档事实：content type、UTF-8、原文件 byte size、adapter id、format id、去重且不含 fallback 的 `symbol_count`，以及分页前、包含 fallback 的 `outline_entry_count`。Ast-grep version、AST kind、rule id 和 parser tree 保持私有。
9. Rollout 只增加 adapter、registry entry、owner、fixtures 和 package 证据，不迁移现有 ref、配置或其它 adapter；回滚只删除这些新增 surface，并保留届时 Current definitions。
10. 本 design 是实施期间唯一承载 change-local Target 的载体，并登记以下 owner delta：新增 code adapter owner；在 architecture/adapter/routing/ref/protocol/output/examples 中登记实际成立的 linked adapter surface；在 testing、Case、release owner 中登记新增证据入口。实现期间稳定 owner 只提供 Current 基线，不提前写入 Target。只有 tests、integration 和 canonical package 行为证据通过后，才把已成立的 delta 同步为 Current，并再次验证 design、owner、实现和证据一致。

## Risks / Trade-offs

- Ast-grep API 与内置 rules 会变化：精确锁版本、私有 owned mapping，并用每语言 fixtures 把升级变成显式审查。
- 静态 parser 增加编译时间和 binary size：只启用首批 parser，记录 feature closure、license 与 release binary delta。
- Partial tree 和语言覆盖不均：接受可恢复 symbol，无 symbol 时 fallback；未覆盖语义作为限制，不让输出静默漂移。
- Byte-range ref 在编辑后失效：digest 阻止旧 range 读取错误区域，并明确只保证同一 compatible view 的 round trip。
- Registry 和 release smoke 可能与其它 Change 重叠：按 adapter identity 断言，保留届时 Current definitions，不依赖固定数量或顺序。

## Open Questions

无改变目标或设计的未决问题。恢复时的依赖版本、Current rebase 和产品排序属于 tasks 中的实施门禁，不授权绕过既有产品决策。
