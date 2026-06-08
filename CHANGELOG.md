# Changelog

本文件记录项目的 notable 变更，格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- 首次启动自动导入演示数据（新建 `todos.db` 时写入示例分类、标签与任务）
- 启动自检：检测数据目录可写与数据库读写，失败时提示用户
- 全局 invoke 错误提示与 `app.log` 日志
- 弹窗 `append-to-body` / 层级修复，改善 WebView2 下设置、新建任务不可用的问题

## [0.1.0] - 2026-06-07

首个公开版本，提供完整的本地桌面待办 MVP。

### Added

- 任务管理：创建、编辑、完成、置顶、软删除、回收站、拖拽排序、自动保存
- 组织方式：分类、标签、优先级、开始 / 截止日期、负责人
- 子任务：增删改、勾选完成、进度展示
- 视图：列表视图、看板视图（列内 / 跨列拖拽）、极简边缘吸附模式
- 周期提醒：公历重复（日 / 周 / 月 / 季 / 年）、农历年度重复与节日预设
- 富文本与附件：TipTap 编辑器、图片 / PDF / Office 等附件管理
- 搜索：SQLite FTS5 + jieba 中文分词，组合筛选
- 桌面集成：系统托盘、单实例、全局快捷键、快速捕获、独立详情窗口、窗口透明度
- 通知：Windows Toast、SMTP 邮件、应用内提醒、提前提醒与重复频率
- 数据：Zip 备份 / 恢复、JSON 导出 / 导入、演示数据 CLI（`seed-demo`）
- 个性化：浅色 / 深色 / 跟随系统、中英文、可自定义快捷键
- 多窗口：主界面、极简、快速捕获、独立详情（Hash 路由 + Tauri Event 同步）

### Documentation

- README、架构文档、功能清单、贡献指南
- MIT License

[Unreleased]: https://github.com/yuxiaoyong/Todo-List/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yuxiaoyong/Todo-List/releases/tag/v0.1.0
