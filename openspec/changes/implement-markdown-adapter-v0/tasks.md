一句话核心：实现独立 Markdown adapter，让真实 Markdown 文档完成 outline、ref、read、find 和 info。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. Adapter 基础命令

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）建立 `docnav-markdown` adapter 模块和可执行入口。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）实现 `manifest --output protocol-json`，声明 Markdown 格式、协议范围、推荐参数和四项 capability。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）实现 `probe`，只返回格式识别证据，不执行导航。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）接入 `docnav-adapter-sdk` 的 invoke 分发。

## 2. Markdown 导航能力

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）接入成熟 Markdown parser，并保留 heading 源码位置。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）实现扁平 outline，默认 H1-H3，entry 包含唯一 ref 和紧凑 display。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）实现 ref 生成和解析，覆盖重复标题与重复完整路径。
- [ ] 2.4 （阻塞：等待 0.1 用户审计确认）实现 read，通过 ref 唯一读取章节并返回 `content_type: "text/markdown"`。
- [ ] 2.5 （阻塞：等待 0.1 用户审计确认）实现 find，返回有限 matches、ref、display 和 page。
- [ ] 2.6 （阻塞：等待 0.1 用户审计确认）实现 info，返回 Markdown 紧凑摘要和 capability 信息。

## 3. 分页与输出

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）实现 Unicode 字符预算分页，覆盖 outline/read/find。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）实现超长 display 和超长 ref 的分页前进规则。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）实现 adapter 直接 CLI 的 text、readable-json 和 protocol-json 输出。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）覆盖无 heading、仅深层 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节和非 UTF-8 fixture。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）运行 Markdown adapter 单元、invoke 和直接 CLI 测试。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）用局部 diff 确认只修改 Markdown adapter 和相关测试范围。
