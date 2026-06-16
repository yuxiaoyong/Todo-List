# Agent Notes — Todo List (Tauri + Vue)

本文件记录在本仓库开发、调试、打包时容易踩坑的事项。**修改前端 UI、i18n 或 Tauri 配置后，务必验证 release 构建，不要只测 `tauri dev`。**

## 1. Release 与 Dev 行为不一致

| 场景 | Dev (`tauri dev`) | Release (`tauri build`) |
|------|-------------------|-------------------------|
| 页面来源 | `http://localhost:1420` | `http://tauri.localhost`（打包资源） |
| CSP  enforcement | 相对宽松 | 严格生效 |
| 典型现象 | 弹窗、设置、插值文案正常 | 主界面可能正常，但设置/新建任务等弹窗空白或只剩遮罩 |

**结论：** 涉及弹窗、i18n 插值、Tauri `invoke` 的改动，完成后应执行 `npm run tauri build` 并运行 `src-tauri/target/release/todo-list.exe` 做冒烟测试。

## 2. CSP（Content Security Policy）— 必读

CSP 配置位于 `src-tauri/tauri.conf.json` → `app.security.csp`。

### 2.1 Tauri 2 必需的 CSP 项

生产环境除 `'self'` 外，还需允许 Tauri 专用协议，否则 IPC、静态资源会失败：

- `default-src` / 资源：`asset:`
- `connect-src`：`ipc: http://ipc.localhost`（`invoke` 通信）
- `img-src`：`asset: http://asset.localhost blob: data:`
- `font-src`：`'self' data:`（如 `hevue-img-preview` 的 iconfont）
- `style-src`：`'unsafe-inline'`（Element Plus / Vue 内联样式）

参考：[Tauri CSP 文档](https://v2.tauri.app/security/csp/)

### 2.2 不要随意收紧 `script-src`

当前项目使用 **vue-i18n 9**，带占位符的文案（如 `settings.opacityValue` 的 `{value}`、`errors.invokeFailed` 的 `{detail}`）在**运行时**通过 `new Function()` 编译。

若 `script-src` 只有 `'self'` 且不含 `'unsafe-eval'`，会出现：

- 控制台：`Evaluating a string as JavaScript violates ... Content Security Policy`
- UI：设置弹窗等只有遮罩、内容为空；部分依赖插值翻译的组件渲染失败
- 纯字符串 key（如 `sidebar.totalTasks`）仍可能正常，**容易误判为「只有某个页面坏了」**

**当前策略：** `script-src 'self' 'unsafe-eval'`（见 `tauri.conf.json`）。

**更安全的长期方案（未启用）：** 接入 `@intlify/unplugin-vue-i18n` 构建期预编译全部 locale，然后移除 `'unsafe-eval'`。接入时需同时配置 `vue-i18n` runtime alias，并处理 locale 中 `@` 等特殊字符；改动后必须全量回归 release 构建。

### 2.3 修改 CSP 时的检查清单

- [ ] 是否保留 `connect-src ipc: http://ipc.localhost`
- [ ] 是否保留 `asset:` / `font-src`（字体与打包资源）
- [ ] 若移除 `'unsafe-eval'`，是否已启用 i18n 构建期预编译且 release 弹窗/插值文案均正常
- [ ] 是否运行 release exe 验证设置、新建任务、添加分类/标签

## 3. i18n 文案规范

- 文案文件：`src/i18n/locales/zh-CN.ts`、`en.ts`；配置入口：`src/i18n/index.ts`
- 新增/修改用户可见文案时，**中英文同步**
- **含 `@` 的 literal 文本**（如邮箱占位 `noreply@example.com`）须转义，否则 vue-i18n 会当作 linked message 解析，触发 `Invalid linked format`：

  ```ts
  fromAddressPlaceholder: "noreply{'@'}example.com"
  ```

- 优先使用简单字符串；插值使用 `{name}` 等占位符时，记住 release 环境依赖 CSP 中的 `'unsafe-eval'`（除非已预编译）

## 4. Element Plus 弹窗（WebView2）

- 设置、新建任务等使用 `el-dialog` + `append-to-body`（Teleport 到 `body`）
- 全局样式：`src/styles/main.css` 中已为 WebView2 设置 `.el-overlay` / `.app-dialog` 的 `z-index`，避免遮罩挡点击
- **任务详情抽屉**（`.task-drawer`）内的 `el-select` / `el-popover` / `el-date-picker` 默认 `teleported` 到 `body`，popper 原始 z-index (~2000) 会低于 drawer overlay (3999)，表现为下拉/日期面板打不开或无法点击。已在 `main.css` 用 `body:has(.task-drawer) .el-popper { z-index: 4100 }` 修复
- 弹窗相关样式若需作用于 Teleport 内容，应写在**全局 CSS**，不要依赖 scoped 样式

## 5. 推荐验证步骤

```bash
npm run tauri build
# 运行 release 可执行文件
./src-tauri/target/release/todo-list.exe   # Windows
```

至少手动或自动化确认：

1. 主界面：侧边栏、任务列表、统计数据
2. 设置弹窗：各 Tab 有内容（尤其含 `{value}` 的透明度 slider）
3. 新建任务弹窗：表单字段完整
4. 添加分类 / 添加标签：内联表单可展开
5. 控制台无 CSP / `unsafe-eval` / IPC 相关错误

## 6. 常见误判

| 现象 | 常见误判 | 实际原因 |
|------|----------|----------|
| Release 弹窗空白 | Element Plus 或 Teleport bug | CSP 阻止 vue-i18n 运行时编译 |
| Release 部分 API 失败 | Rust 命令未注册 | CSP 缺少 `connect-src ipc:` |
| Dev 正常、Release 异常 | 前端打包路径错误 | 多为 CSP / i18n，而非资源未打包 |
| 主界面正常、设置异常 | 设置页单独 bug | 设置页大量使用 i18n 插值，触发 eval 限制 |

## 7. 相关文件

- CSP：`src-tauri/tauri.conf.json`
- i18n：`src/i18n/`、`src/stores/locale.ts`
- 弹窗样式：`src/styles/main.css`
- 设置 / 新建任务：`src/components/settings/SettingsDialog.vue`、`src/components/todo/NewTaskDialog.vue`

## 8. Release 远程 Debug 排查流程

Release 包没有 Vite 热更新，也默认打不开 DevTools。排查「dev 正常、build 异常」时，可通过 **WebView2 远程调试端口 + CDP 脚本** 直接读控制台与 DOM。

### 8.1 启用 WebView2 远程调试

在启动 release exe **之前**设置环境变量（仅对当前进程生效）：

**PowerShell（Windows）：**

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
Start-Process ".\src-tauri\target\release\todo-list.exe"
```

确认调试端口已监听：

```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:9222/json/list"
```

应返回类似条目：`url: http://tauri.localhost/#/`、`title: Todo List`。

> **注意：** 修改 exe 后需先结束旧进程再 rebuild，否则 Windows 可能报「拒绝访问」无法覆盖 exe：
>
> ```powershell
> Stop-Process -Name "todo-list" -Force -ErrorAction SilentlyContinue
> ```

### 8.2 用 CDP 连接（puppeteer-core）

DevTools 协议走 WebSocket，适合用 `puppeteer-core` 连接已有 WebView2，无需再装 Chromium。

```powershell
# 一次性准备（任意临时目录即可）
mkdir $env:TEMP\tauri-cdp-test -Force
cd $env:TEMP\tauri-cdp-test
npm init -y
npm install puppeteer-core@24.2.0
```

最小连接示例：

```javascript
const puppeteer = require("puppeteer-core");

(async () => {
  const browser = await puppeteer.connect({ browserURL: "http://127.0.0.1:9222" });
  const page = (await browser.pages()).find((p) => p.url().includes("tauri.localhost"));
  console.log(await page.title());
  await browser.disconnect();
})();
```

### 8.3 本次问题的排查步骤（可复用）

按以下顺序由浅入深，避免一上来改 CSP 或 i18n：

**Step 1 — 确认 release 能否构建、能否启动**

```bash
npm run tauri build
# 运行 exe，肉眼看主界面是否正常
```

**Step 2 — 挂 Vue 错误钩子 + 监听控制台**

在 CDP 脚本里于操作前注入：

```javascript
page.on("pageerror", (err) => errors.push({ message: err.message, stack: err.stack }));
page.on("console", (msg) => {
  if (msg.type() === "error") errors.push(msg.text());
});

await page.evaluate(() => {
  const app = document.querySelector("#app")?.__vue_app__;
  window.__vueCaptured = [];
  app.config.errorHandler = (err, _i, info) => {
    window.__vueCaptured.push({ message: String(err?.message || err), info });
  };
});
```

**Step 3 — 复现问题并检查 DOM**

打开设置按钮后检查 overlay / dialog 结构：

```javascript
const state = await page.evaluate(() => {
  const overlay = [...document.querySelectorAll(".el-overlay.el-modal-dialog")]
    .find((el) => getComputedStyle(el).display !== "none");
  return {
    dialogCount: document.querySelectorAll(".el-dialog").length,
    overlayHTML: overlay?.innerHTML?.slice(0, 500) ?? null,
  };
});
```

| 观察到的 DOM | 含义 |
|--------------|------|
| `overlayHTML` 为 `<div class="el-overlay-dialog"> <!----> </div>` | 遮罩已打开，**dialog 内容 vnode 渲染失败** |
| 有 `.el-dialog__header` 但 `__body` 为空 | 部分组件渲染失败（多为子树内报错） |
| 控制台 CSP 报错 + `IPC custom protocol failed` | `connect-src` 缺少 `ipc:` |

**Step 4 — 对照控制台关键字**

| 控制台信息 | 指向 |
|------------|------|
| `violates ... Content Security Policy ... 'unsafe-eval'` | vue-i18n 运行时编译插值文案被 CSP 拦截 → 见 §2.2 |
| `connect-src ... default-src 'self'` + `ipc.localhost` | Tauri invoke 被 CSP 拦截 → 补 `connect-src ipc:` |
| `IPC custom protocol failed, ... postMessage` | IPC 协议失败后的降级，常伴随 CSP 问题 |
| `Invalid linked format` | locale 中未转义的 `@` 等字符 → 见 §3 |
| Vue `UNEXPECTED_RETURN_TYPE`（code 24） | 误配 `@intlify/unplugin-vue-i18n` 且未配 runtime alias 时，整页可能白屏 |

**Step 5 — 区分「真坏」与「测试方式问题」**

- 用 `page.click(".add-category-btn")` 可能点到错误坐标；改用 `page.evaluate(() => document.querySelector(...).click())` 更可靠。
- 侧边栏有两个 `.add-category-btn`（添加分类 / 添加标签），应用 `innerText` 区分。
- 设置弹窗打开后内联表单可能被 `v-if/v-else` 替换按钮，此时 `btnVisible: false` 是预期行为。

**Step 6 — 修复后自动化冒烟**

修复 CSP 或 i18n 后，用 CDP 批量断言（本次验证通过的检查项）：

1. `.settings-btn` 存在，`bodyTextLen > 100`
2. 点击设置 → `.el-dialog` 存在且 `innerText.length > 50`
3. 点击「添加分类」→ `.add-category-form` 出现
4. 点击「新建任务」→ `.el-dialog .el-form` 出现
5. `fatalErrorCount === 0`（可忽略邮件 Tab 的 `Invalid linked format`，修复 `@` 转义后应消失）

### 8.4 本次排查结论摘要

1. **根因：** release CSP 过严 — 缺 `unsafe-eval`（vue-i18n 插值）+ 缺 Tauri 专用 `connect-src` / `asset:`。
2. **表象：** 主界面纯字符串 i18n 正常；设置弹窗（大量 `{value}` 插值）打开后 overlay 有、内容空。
3. **修复：** 更新 `tauri.conf.json` CSP；locale 邮箱占位符使用 `{'@'}` 转义。
4. **验证：** `npm run tauri build` + CDP 冒烟，`ok: true`（主界面、设置、添加分类/标签、新建任务均正常）。

### 8.5 其他可选手段

- **人工 DevTools：** 启动带 `--remote-debugging-port=9222` 的 exe 后，Chrome/Edge 打开 `chrome://inspect` → 选择对应 page inspect（与 CDP 同源）。
- **Rust 侧日志：** `src-tauri/src/infra/log.rs`、`invoke` 失败时前端 `tauriInvoke.ts` 会写 app log；DOM/控制台无法访问时再看。
- **不要依赖 `vite preview` 代替 release：** 浏览器里没有 Tauri CSP 与 `ipc.localhost`，无法复现同类问题。

## 9. 列表视图开发备忘

主界面任务列表为自定义实现（`DraggableTaskList.vue`），**不是** `el-table`。架构说明见 [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md#列表视图实现)。

### 9.1 关键约束

| 场景 | 注意点 |
| ---- | ------ |
| 表头 / 表体对齐 | 表头与表体必须在同一横向滚动容器内；列设置按钮叠在表头，不要放进表体行 |
| 纵向 sticky 表头 | `TaskListPanel` 的 `.table-wrap` 用 `overflow: hidden`；纵向滚动只在 `.task-list-scroll` |
| 固定列重叠 | 左 / 中 / 右三区各自设不透明背景，避免 sticky 重叠时透视 |
| 子组件样式 | 行 / 列单元格在独立 `.vue` 中，父组件需 `:deep()` 或子组件自带 scoped 样式 |
| 列宽 | 固定像素宽定义在 `taskListColumns.ts`；标题列用 `minmax(Npx, 1fr)` 占满剩余宽度 |

### 9.2 相关文件

- 布局：`DraggableTaskList.vue`、`TaskListPanel.vue`
- 单元格：`TaskListColumnCells.vue`（表头）、`TaskListRowCells.vue`（表体）
- 列配置：`TaskListColumnSettings.vue`、`stores/taskListColumns.ts`、`utils/taskListColumns.ts`
- 上下文：`taskListCellContext.ts`（`provide` / `inject`）

### 9.3 修改后验证

1. 开启多列 → 横向滚动，左 / 右列保持固定
2. 纵向滚动 → 表头 sticky 不随内容滚走
3. 列设置 Popover → 显隐 / 排序生效且重启后保留
4. 行内编辑（标题、分类、标签等）→ 不破坏列对齐
5. 涉及 i18n 插值时按 §1 执行 release 构建冒烟
