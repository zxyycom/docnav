---
title: 保留 JSON 数字的原始 token
status: archived
alignment: null
createdAt: 2026-07-28T11:57:52Z
purpose: 让合法 JSON number 在解析、导航和结构化读取之间保持同一文本身份。
background: JSON 导航观察 number token，常见整数或浮点模型无法无损承载全部合法 token。
decision: JSON adapter 保存并原样序列化 number token；arithmetic equivalence 位于导航能力之外。
relations: []
---

## 目的
- 保证 read、find preview 和分页使用的数字文本与已验证 source token 一致。
- 让 JSON navigation 以 token identity 工作，数值计算由其它能力拥有。

## 背景
- 合法 JSON number 可以超出常见整数范围，也可以使用小数或指数 spelling；转换为 binary float 可能舍入，固定整数模型也可能溢出。
- `2.9999999`、`3`、`3.0` 和 `3e0` 在 navigation 中分别保留各自 token；数学关系属于另一语义层。
- Parser/serializer 对普通 lexeme 的规范化不覆盖 number token identity。

## 决策
- 采用: JSON decode model 必须保存每个 number 的原始、语法已验证 token，并在 structured read 中原样写回。
- 采用: Navigation、search preview 和 serialization 都以 raw token 为 number 的可观察身份。
- 采用: 保存机制保持 adapter-private，并使用 pinned JSON parser 的 raw-value 能力或等价私有 mechanics。
