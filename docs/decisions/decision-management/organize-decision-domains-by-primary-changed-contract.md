---
title: 按主要被改变契约组织决策领域
status: active
alignment: aligned
createdAt: 2026-08-06T04:07:25Z
purpose: 让每个决策领域成为无需隐藏分支即可恢复相关长期判断的有界入口，而不是宽泛主题标签。
background: 旧领域曾把 adapter 选择、文档生命周期、边界证据和产品排序混在一起，也把机器读取语义与人类展示放在同一查询边界。
decision: 决策按主要被改变契约、稳定 owner 和共同消费者归域；独立演进的责任拆域，同一格式 owner 的成功语义保持聚合。
relations: []
---

## 目的
- 让 `list --domain` 返回由共同 owner 和消费者直接使用的长期判断集合，不要求调用方再按隐藏责任手工分流。
- 让领域数量由真实变化隔离决定，既不把不同契约压进宽泛主题，也不为每个局部名词创建标签式小领域。

## 背景
- Adapter selection、selected-document lifecycle、共享边界证据和格式扩展排序具有不同 owner、消费者与变化原因，不能仅因都涉及 adapter 就共享一个领域。
- Structured/full-read 机器语义与 `readable-view` presentation 属于独立输出层；前者变化不应迫使人类展示决策一起演进。
- JSON 的 input、ref、find、安全和 value representation 虽然表面不同，但共同由 JSON adapter 的成功语义拥有；继续聚合比拆成多个单记录领域更容易恢复完整格式约束。
- `<domain>/<slug>.md` 是稳定身份；领域纠正是结构迁移，不是决策语义演进。

## 决策
- 采用: 一条记录归入拥有其主要被改变契约的领域；正文出现的模块名、执行阶段和全部受影响对象不形成自由 tag 或次级领域。
- 采用: 当一组判断能够独立修订、由不同 owner 验收、面向不同主要消费者，且合并查询会要求隐藏分支时，拆成独立领域。
- 采用: 当判断共享同一格式 owner、成功语义、生命周期和主要消费入口时，即使涉及多个 operation 或实现表面，也保持在同一领域。
- 采用: Adapter selection、adapter document lifecycle 和 adapter boundary evidence 使用独立领域；格式扩展先后仍归产品方向。
- 采用: Structured read semantics 与 readable presentation 使用独立领域；raw/readable 分层不通过同域重新混合。
- 采用: 领域或 slug 的结构迁移必须使用完整 old-to-new 映射，保持正文、`createdAt`、生命周期、alignment 和关系图语义，并在同一受控迁移批次中更新关系目标、直接链接与派生索引；不伪造演进关系，也不保留会被扫描成第二条决策的 Markdown redirect。
- 边界: 只有检索、owner 或变化隔离具有长期收益时才新增或拆分领域；单条决策、名称相似或目录整齐本身都不是充分理由。精确领域集合和描述由 `decision-domains.json` 拥有。
