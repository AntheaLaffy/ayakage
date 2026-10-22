---
mode: maintenance
generated_at: 2026-10-22
ignore: 见范围与排除项
---

# 项目导航

Ayakage 是原生 Rust 应用，从 OpenSeeFace 的 UDP 输入驱动模型参数，生成透明 wgpu 纹理供预览及独立直播窗口使用。

## 范围与排除项

| 路径 / 模式 | 处理 | 原因 |
| --- | --- | --- |
| `target/`, `.local/`, `dist/`, `.project-maintenance/`, `*.tmp` | Excluded | `.gitignore` 中的编译、验证、分发产物、维护指纹及临时配置 |
| VCS / IDE 缓存、`Cargo.lock` | Excluded | 元数据 / 依赖锁文件；依赖改动仍需验证，不以排除作为正确性证据 |
| `assets/` | Boundary only | 字体和许可证静态资源，运行时以编译期内嵌方式消费 |
| `CODEMAP.md` / 子目录地图 | Excluded | 导航工件不重复纳入源文件清单 |

## 任务导航

| 任务 | 领域 | Target | Also Check |
| --- | --- | --- | --- |
| 修改应用入口 / CLI | 应用 | `src/CODEMAP.md` → `main.rs` | 需要同步选项时读 `README.md` |
| 修改追踪 / 映射 | 追踪 | `src/CODEMAP.md` → `tracking.rs`, `config.rs` | `model.rs`, UI 映射页 |
| 修改模型加载 / 动作 / 渲染 | 模型 | `src/CODEMAP.md` → `model.rs`, `render.rs` | `verify.rs`, MVP 验收 |
| 修改原生界面 / 输出窗口 | UI | `src/app/CODEMAP.md` | 配置或调度变化时读 `src/app.rs` |
| 构建、打包、检查 | 工具 | `tools/CODEMAP.md`, `CONTRIBUTING.md` | `docs/VERIFICATION.md` |
| 修改 CI、合并门禁与版本发布 | 发布 | `.github/CODEMAP.md`, `docs/CI.md` | `tools/package.py`, `CHANGELOG.md` |
| 查看范围、使用步骤与后续方向 | 产品 | `README.md`, `docs/CODEMAP.md` | `talk.md` 为原始背景 |

## 子目录

| 目录 | 领域 | 依赖 | 职责 |
| --- | --- | --- | --- |
| [src/](src/CODEMAP.md) | 应用 | egui / Mocari / wgpu | 可执行应用及内联测试 |
| [docs/](docs/CODEMAP.md) | 产品 | 实际功能及验证产物 | MVP 范围、路线图和验收记录 |
| [tools/](tools/CODEMAP.md) | 工具 | Cargo / Python | 打包、合成追踪发送、字体子集更新 |
| [.github/](.github/CODEMAP.md) | 发布 | GitHub Actions / 仓库设置 | 自动检查、分支和标签门禁、版本发布 |
| `assets/` | 静态资源 | 编译期字体内嵌 | **Internals not indexed**；字体及许可 |

## 文件

| 文件 | 领域 | 职责 |
| --- | --- | --- |
| `Cargo.toml` | 构建 | 包、依赖、Linux UI features 及 release 配置 |
| `.gitignore` | 构建 | 排除构建、验收、分发产物和临时配置 |
| `rust-toolchain.toml` | 构建 | 固定开发 / 发布 Rust 版本与质量检查组件 |
| `README.md` | 使用 | 安装、启动、面捕与 OBS / niri 接入 |
| `CONTRIBUTING.md` | 开发 | 可执行验证入口及数据格式约定 |
| `CHANGELOG.md` | 发布 | 按版本记录产品行为与发布边界 |
| `AGENTS.md` | 导航 | 当前源码与地图维护协议 |
| `LICENSE` | 许可 | 应用代码 MIT 许可 |
| `THIRD_PARTY.md` | 许可 | 运行时、UI、字体及外部面捕来源 |
| `talk.md` | 背景 | 开发前对 Linux 开源 Live2D 生态的讨论原文 |
