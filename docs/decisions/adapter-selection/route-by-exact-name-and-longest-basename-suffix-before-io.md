---
title: 按精确文件名和最长完整 basename suffix 先行路由
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:38Z
purpose: 以 manifest 的 exact filename 和完整 basename suffix 在文档 I/O 前确定 adapter，并保留无回退验证边界。
background: 复合 suffix、dotfile 和项目专用名称需要比 terminal extension 更精确的确定性规则，但不需要内容探测或通用 pattern 系统。
decision: 先匹配 exact filename，再按最长完整 basename suffix 匹配；命中事实保持私有，选中后验证真实文档且失败不回退。
relations:
  - type: 修订
    target: adapter-selection/route-by-pathname-before-document-io.md
---

## 目的
- 在任何 document metadata、open、canonicalize、read 或 parse 前，仅凭调用 pathname 与 manifest 提示完成 automatic routing。
- 让普通 extension、复合 suffix 和少量 exact filename 共享一套精确、轻量且无需额外 routing dependency 的规则。
- 保持 pathname selection、真实格式验证和 selected-failure 处理三层边界可独立审计。

## 背景
- Terminal-extension-only extraction 会把 `.schema.json` 与 `.json` 压成同一粒度，也不能自然覆盖 dotfile 和项目专用 filename。
- MIME、content sniffing、regex、glob、directory pattern、第二份 alias store 或 detector framework 会扩大 routing owner，并可能在 selection 阶段引入 document I/O 或顺序依赖。
- Pathname hint 只决定选择哪个 adapter，不能证明真实文档符合该格式；命中事实也不应渗入 operation contract。

## 决策
- 采用: Manifest format descriptor 是 automatic routing metadata 的唯一 owner，继续声明 `extensions[]` 并声明可为空的 `filenames[]`；不增加 MIME、content inference、regex、glob、directory-aware pattern、第二 alias registry 或新的 routing dependency。
- 采用: Automatic routing 只从调用 pathname 词法派生完整 basename；route 确定前不得对目标文档执行 metadata、open、canonicalize、read 或 parse。
- 采用: 先以大小写敏感的完整 basename 精确匹配 `filenames[]`；未命中时，把 `extensions[]` 解释为带前导点且可包含多个点的完整 basename suffix，并在 ASCII 大小写归一化后比较。
- 采用: 多个 suffix 同时命中时选择最长 suffix；ASCII 归一化后相同的 suffix，或 spelling 完全相同的 exact filename，继续作为 manifest registry 冲突，不向 manifest 暴露更通用的 pattern language。
- 采用: Derived basename、matched hint 和 matched format identity 保持 invocation-private，不进入 public protocol、ref、continuation、typed input、日志或 adapter operation contract。
- 采用: Route 命中后才处理真实路径，并由选中 adapter 按自身 grammar、安全和 operation contract 读取、解析和验证文档；显式 adapter identity 直接选择对应 adapter，selected parse、semantic 或 operation failure 不重新路由或 fallback。
- 不采用: Terminal-extension-only extraction、route 前 document I/O、content probe/sniffing，以及 selected failure 后 fallback。
