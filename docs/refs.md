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

Markdown、JSON 和其它格式的定位语法均由对应 adapter 拥有。具体 ref 语法属于 adapter 私有实现或 adapter 自有兼容契约，不属于共享协议、`docnav` CLI 或 MCP 公共格式。

共享协议、`docnav` 和接入层不得解析、拼接、规范化或从 display 推断 ref，只能把 adapter 返回的 ref 原样传递给 read。

适配器生成的 ref 在当前文档中必须非空、唯一并可直接用于 read。adapter 可以为没有局部导航区域的文档定义全文 ref；该 ref 的具体拼写属于对应 adapter。

文档变化后，调用方应重新 outline 或 find；读取失败时 adapter 返回 `REF_NOT_FOUND` 或 `REF_AMBIGUOUS`。
