一句话核心：实现 `docnav` 核心 CLI 的路由和输出层，让用户命令稳定调用 adapter。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. CLI 参数与配置

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）建立 `docnav` 核心 CLI 命令结构，覆盖 outline/read/find/info/init/doctor/version/config。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）实现未知参数、缺失值和多余参数校验。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）实现显式参数、项目配置、用户配置、内置默认值的解析优先级。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）确保从 `docnav` 入口省略 page 时显式转换为 `1`，limit_chars 为有限正整数；adapter 直接 CLI 的 page 省略支持由 adapter 契约覆盖。

## 2. Path 与 Adapter 选择

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）实现项目根发现和 path 规范化，允许项目根外可访问文件。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）实现 adapter registry 读取接口，先支持简单相对命令路径记录，后续可接正式管理记录。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）实现 `--adapter <adapter-id>` 预选 adapter 校验。
- [ ] 2.4 （阻塞：等待 0.1 用户审计确认）实现无 `--adapter` 时的 core 简易推断预选 adapter。
- [ ] 2.5 （阻塞：等待 0.1 用户审计确认）实现预选失败后的 registry 遍历函数和 FORMAT_UNKNOWN。

## 3. Invoke 与输出映射

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）实现 adapter invoke 子进程启动，cwd 设置为项目根。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）将最终 page、limit_chars、ref、query 和 options 写入显式 invoke 请求。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）实现 protocol-json 输出，保留完整 protocol envelope。
- [ ] 3.4 （阻塞：等待 0.1 用户审计确认）实现默认 text 和 readable-json 输出，不包含 protocol envelope。
- [ ] 3.5 （阻塞：等待 0.1 用户审计确认）实现错误 code、details、guidance 和退出码映射。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）覆盖 `docnav outline -> ref -> read` 端到端测试。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）覆盖 `--adapter` 预选、core 推断预选、预选失败回退遍历和全失败测试。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）验证 protocol-json 与 readable-json 业务语义一致但包装不同。
- [ ] 4.4 （阻塞：等待 0.1 用户审计确认）用局部 diff 确认只修改核心 CLI 和相关测试范围。
