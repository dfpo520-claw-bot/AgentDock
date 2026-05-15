# AgentDock

AgentDock 是面向多引擎 AI Agent 运营的生产级桌面控制台，基于 Tauri v2、Vite 和 Rust 命令层构建。

## 当前阶段

本仓库处于 production fork baseline 阶段：

- 保留现有 OpenClaw 与 Hermes Agent 引擎能力。
- 优先保留 Tauri 桌面壳、Web 模式、服务管理、模型管理、Agent 管理、聊天、日志、诊断、备份与扩展模块。
- 先替换产品身份、图标、文档、应用元数据与发布入口。
- 后续逐步重构配置归属、模块边界与后端命令实现。

## 技术栈

- 前端：Vite + vanilla JavaScript
- 桌面端：Tauri v2
- 能力层：Rust Tauri commands
- 构建：Node.js、Cargo、Tauri CLI

## 本地开发

```bash
npm install
npm run dev
```

桌面开发：

```bash
npm run tauri dev
```

生产构建：

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## 项目状态

第一个里程碑是获得一个可运行、可打包的 AgentDock 基线，为后续重构提供稳定起点。
