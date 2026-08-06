---
title: 路径提示路由先于文档 I/O
status: archived
alignment: null
createdAt: 2026-08-06T02:35:41Z
purpose: 让自动 adapter 选择只消费确定性 pathname 提示，并在选择完成后才访问真实文档。
background: 内容探测会把低成本选择与格式验证混合，并可能造成重复读取和顺序依赖。
decision: 自动路由先使用 manifest 派生的私有 pathname 事实，选中 adapter 再验证真实内容，失败不回退。
relations:
  - type: 修订
    target: adapter-selection/route-by-manifest-basename-hints.md
---

## 目的
- 在访问目标文档前完成确定性 adapter selection。
- 分离低成本 pathname route 与选中 adapter 对真实格式和内容的验证。

## 背景
- Selection 阶段执行 probe、content sniffing 或其它文档 I/O，会与 operation 重复工作并引入 registry 顺序依赖。
- Pathname hint 只能表达希望选择哪个 adapter，不能证明真实文档符合该格式。
- Manifest 字段、basename 匹配和冲突诊断的精确 Current 规则由 adapter、navigation、CLI 和 protocol owner 承接。

## 决策
- 采用: Automatic routing 只使用调用 pathname 和 registry manifest 派生的确定性提示；route 确定前不得对目标文档执行 metadata、open、canonicalize、read 或 parse。
- 采用: Derived lookup、matched hint 和 matched format identity 保持 invocation-private，不进入公共协议、ref、continuation、typed input 或 adapter operation contract。
- 采用: 选择完成后才进入真实路径处理，并由选中 adapter 按自身 grammar、安全和 operation contract 读取、解析和验证文档。
- 采用: 显式 adapter identity 直接选择对应 adapter；selected parse、semantic 或 operation failure 不重新路由或 fallback。
- 边界: Manifest 字段、hint grammar、匹配优先级和 registry 冲突规则由当前 owner 规范定义，不在本决策复制。
