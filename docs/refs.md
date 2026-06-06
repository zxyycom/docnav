# Ref

本文是 Docnav v0 ref 定位和唯一性语义的主规范。

## 核心边界

- `path` 负责定位文档，并作为 `docnav` 选择 adapter 的依据。
- `ref` 只负责定位该文档内部区域。
- ref 是 adapter 生成和解析的非空字符串；共享协议、`docnav` 和接入层只原样传递 ref。

## 生成与消费

1. 适配器在 outline 或 find 中生成 ref。
2. 调用方把相同 path 和 ref 原样传给 read。
3. `docnav` 根据 path 选择 adapter，并将 ref 原样传入。
4. 适配器解析 ref 并读取唯一文档区域。

read 不得使用最近位置、首个匹配或其他启发式方法静默消歧：

- 无匹配返回 `REF_NOT_FOUND`。
- 多匹配返回 `REF_AMBIGUOUS`。
- path 对应的适配器不可用返回 `ADAPTER_UNAVAILABLE`。

## 格式定位所有权

Markdown、JSON 和其它格式的定位语法均由对应 adapter 拥有。这些语法不属于共享协议；共享协议、`docnav` 和接入层只传递字符串。

### Markdown heading ref

Markdown heading ref 的 canonical 格式为：

```text
L{line}:{path}
L{line}#{ordinal}:{path}
```

- `line` 是 heading 在 Markdown 源文档中的 1-based 行号。
- `path` 是 heading breadcrumb，不是文件路径。嵌套 heading 使用 ` > ` 连接，例如 `Guide > Install`。
- `ordinal` 是相同完整 heading breadcrumb 的 occurrence 序号，只在重复 path 的第 2 次及以后出现在 canonical 输出中。
- 首个 occurrence 的 canonical 输出省略默认序号 `#1`。
- Markdown 解析器接受显式 `#1` 输入，例如 `L1#1:Guide`，但生成器仍输出 `L1:Guide`。

无重复 heading path 示例：

```text
L1:Guide
L5:Guide > Install
```

重复完整 heading path 示例：

```text
L1:Repeat
L9#2:Repeat
L5:Repeat > Child
L13#2:Repeat > Child
```

`doc:full` 是 Markdown 全文 fallback ref，用于读取整篇文档。它不属于 heading ref 格式，也不带行号、breadcrumb 或 occurrence ordinal。

适配器生成的 ref 在当前文档中必须唯一并可直接用于 read。文档变化后，调用方应重新 outline 或 find；读取失败时 adapter 返回 `REF_NOT_FOUND` 或 `REF_AMBIGUOUS`。
