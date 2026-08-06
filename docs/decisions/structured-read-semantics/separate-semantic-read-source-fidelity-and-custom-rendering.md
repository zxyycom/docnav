---
title: 分离语义读取、源码保真与自定义渲染
status: archived
alignment: null
createdAt: 2026-07-28T11:11:57Z
purpose: 让结构化读取、原文保真和面向人的自定义渲染各有稳定职责，避免格式接入被源码复刻或展示偏好绑架。
background: 结构化格式经过解析和序列化后会自然规范化部分词法表示，而成员顺序等源码事实只有在局部低成本时才值得保留。
decision: 结构化读取默认采用格式语义和受控序列化，原文由 full-read 拥有，自定义渲染只属于 readable-view，源码顺序按证据决定。
relations: []
---

## 目的
- 让不同格式 adapter 可以交付稳定、可继续的结构化读取，而不默认承担逐字节复刻源文件的复杂度。
- 保持 raw protocol、原文读取和面向人的展示分层，使各层可以独立演进而不互相改写契约。

## 背景
- JSON、代码和后续结构化格式都会把源码解析为格式语义；再次序列化时，空白、escape spelling 和其它词法细节可能被 parser 或 serializer 自然规范化。
- 把所有源码 spelling、member order 或 source span 都提升为 structured read 的通用义务，会迫使 adapter 引入额外 parser、全量源码映射或共享抽象，即使调用方只需要节点语义和可继续 ref。
- 某些信息仍可能影响正确性或可用性，例如会改变值的数值转换、无法唯一定位的重复成员，以及用户实际依赖的源码顺序；这些差异需要格式 owner 和证据判断，不能由 serializer 默认或全局偏好一概决定。
- Raw protocol 需要稳定、可校验的事实；面向人的信息密度、层级、标点和 preview 属于另一消费层，不能反向改变传输 shape。

## 决策
- 采用: Structured read 默认返回选中节点的格式语义表示，并可使用 workspace-pinned parser/serializer 的自然规范化结果；除格式 owner 明确声明的正确性例外外，不承担原始 whitespace、escape spelling 或其它源码 lexeme 的保真。
- 采用: Unstructured full-read 拥有原文读取和源码保真职责；需要完整 spelling、顺序或布局的调用方使用 full-read，而不是要求 structured read 同时充当源码切片接口。
- 采用: 源码成员顺序不是所有结构化 adapter 的无条件共享 invariant。格式可以在 adapter-private 表示能够自然承载、无需新增 parser 或 shared/public surface、且没有显著内存与复杂度牺牲时保留；否则使用并文档化确定性的语义模型顺序。
- 采用: 会改变节点值、身份或 ref 唯一性的格式事实不能以“接受 parser 默认”为由丢失；具体例外由对应格式 owner 定义并由测试证明，不提升为所有格式共享的词法保真义务。
- 采用: “自定义渲染”专指 `readable-view` 对既有 raw result facts 的面向人展示。它可以调整信息密度、层级、标点、preview 和分页显示，但不得改变 raw protocol result、ref、cost、page 或传输包装。
- 采用: 当 raw structured output 依赖 workspace-pinned parser/serializer 的可观察结果时，依赖升级必须经过对应 owner contract 与回归证据复核；不得把依赖版本变化静默当作无行为变化。
- 不采用: 为所有 structured read 统一要求 byte-for-byte source preservation、source spans、固定源码顺序或通用 ordered-tree abstraction。
- 不采用: 让格式专用自定义渲染向 raw protocol 增加 readable-only 字段或包装。
