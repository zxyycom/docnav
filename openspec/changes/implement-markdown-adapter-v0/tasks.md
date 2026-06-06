一句话核心：实现独立 Markdown adapter，让真实 Markdown 文档完成 outline、ref、read、find 和 info。

## 0. 审计门禁

- [x] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现。

## 1. Adapter 基础命令

- [x] 1.1 建立 `docnav-markdown` adapter 模块和可执行入口。
- [x] 1.2 实现 `manifest --output protocol-json`，声明 Markdown 格式、协议范围、推荐参数和四项 capability。
- [x] 1.3 实现 `probe`，只返回格式识别证据，不执行导航。
- [x] 1.4 接入 `docnav-adapter-sdk` 的 invoke 分发。

## 2. Markdown 导航能力

- [x] 2.1 接入成熟 Markdown parser，并保留 heading 源码位置。
- [x] 2.2 实现扁平 outline，默认 H1-H3，entry 包含唯一 ref 和紧凑 display；当前 outline 为空时返回全文 ref。
- [x] 2.3 实现 ref 生成和解析，覆盖重复标题与重复完整路径。
- [x] 2.4 实现 read，通过 ref 唯一读取章节并返回 `content_type: "text/markdown"`。
- [x] 2.5 实现 find，返回有限 matches、ref、display 和 page；match ref 指向最近 outline，outline 为空时使用全文 ref。
- [x] 2.6 实现 info，返回 Markdown 紧凑摘要和 capability 信息。

## 3. 分页与输出

- [x] 3.1 实现 Unicode 字符预算分页，覆盖 outline/read/find。
- [x] 3.2 实现超长 display 和超长 ref 的分页前进规则。
- [x] 3.3 实现 adapter 直接 CLI 的 text、readable-json 和 protocol-json 输出。

## 4. 验证与审计

- [x] 4.1 覆盖无 heading、仅深层 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节、find 最近 outline ref、全文 ref fallback 和非 UTF-8 fixture。
- [x] 4.2 运行 Markdown adapter 单元、invoke 和直接 CLI 测试。
- [x] 4.3 用局部 diff 确认只修改 Markdown adapter 和相关测试范围。
