# 演示数据

项目 `demo/demo-data.json` 包含用于产品演示的示例数据集快照。

## 首次启动

**首次安装**（本地尚无 `todos.db`）时，应用会在数据库初始化完成后自动导入演示数据，无需手动操作。

## 内容概览

- **3 个分类**：工作、生活、学习
- **4 个标签**：紧急、本周、会议、生日
- **3 个看板列**：待办、进行中、待验收
- **10 条任务**：含置顶、子任务、富文本、农历周期提醒等场景

## 重置为演示数据

请先**关闭正在运行的 Todo List**，然后在项目根目录执行：

```bash
cd src-tauri
cargo run --bin seed-demo
```

也可指定数据库路径：

```bash
cargo run --bin seed-demo -- "C:\Users\你\AppData\Roaming\com.tx.todo-list\todos.db"
```

`demo/demo-data.json` 为导出的快照，可通过应用内「设置 → 数据 → 导入 JSON」手动导入（需先清空或在新环境中使用）。
