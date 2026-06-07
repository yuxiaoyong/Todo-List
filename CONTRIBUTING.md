# 参与贡献

感谢你对 Todo List 的关注！本项目欢迎 Issue 与 Pull Request。

## 开始之前

1. 阅读 [README.md](./README.md) 了解项目定位与构建方式
2. 查看 [Feature.md](./docs/Feature.md) 确认功能是否已实现
3. 查看 [Plan.md](./docs/Plan.md) 了解路线图，避免重复劳动

## 报告问题

在 [GitHub Issues](https://github.com/yuxiaoyong/Todo-List/issues) 提交时请尽量包含：

- **环境**：Windows 版本、应用版本（或 commit hash）
- **复现步骤**：从启动到出问题的完整操作
- **期望行为** vs **实际行为**
- **截图 / 日志**（如有）

## 提交代码

### 环境准备

```bash
npm install --legacy-peer-deps
npm run tauri dev
```

需要 Node.js 18+、Rust 工具链；Windows 上需 [WebView2 运行时](https://developer.microsoft.com/microsoft-edge/webview2/)。

### 分支与 PR

1. Fork 本仓库并创建功能分支（如 `fix/notification-toast`、`feat/search-highlight`）
2. 保持 PR **聚焦**：一个 PR 解决一类问题，避免大范围无关改动
3. 在 PR 描述中说明：**改了什么、为什么、如何自测**
4. 确保本地能正常 `npm run tauri dev` 启动

### 代码约定

- **前端**：遵循现有 Vue 3 + TypeScript + Pinia 风格；UI 文案需同步 `src/i18n/locales/zh-CN.ts` 与 `en.ts`
- **Rust**：业务逻辑放在 `repositories/`，对外接口放在 `commands/`；数据库变更需新增版本化迁移 SQL
- **范围控制**：不做与 Issue 无关的重构；不引入新依赖除非必要

### 数据库迁移

若修改表结构，在 `src-tauri/src/db/migrations/` 新增递增编号 SQL，并在 `src-tauri/src/db/mod.rs` 注册。详见 [ARCHITECTURE.md](./docs/ARCHITECTURE.md#数据库迁移)。

## 欢迎的贡献方向

- 搜索关键词高亮、分类 / 标签侧边栏编辑
- Rust 层通知 / 托盘文案 i18n
- macOS / Linux 平台适配与测试
- 文档完善（含英文 README）
- Bug 修复与 UI 打磨

## 许可证

提交的代码将按项目 [MIT License](./LICENSE) 授权。
