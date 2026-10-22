# 应用源码

主线程拥有 UI / GPU / 可变模型；模型解析与文件选择器在工作线程运行，UDP 在每次逻辑更新中有界读取。

## 任务导航

| 任务 | 领域 | Target | Also Check |
| --- | --- | --- | --- |
| CLI、启动参数、退出状态 | 入口 | `main.rs` | `app.rs`, `verify.rs`, `control.rs` |
| UDP 字段、校准、平滑、选脸、失联处理 | 追踪 | `tracking.rs` | `model.rs`, `verify.rs` |
| 参数映射与配置迁移 / 保存 | 配置 | `config.rs` | `app.rs`, `app/ui/pages.rs` |
| 模型导入、动作、表情、参数驱动 | 模型 | `model.rs` | `render.rs`, `verify.rs` |
| 遮罩、纹理、变换、alpha 读回 | 渲染 | `render.rs` | `app.rs`, `verify.rs` |
| 界面流程、加载和渲染调度 | UI | `app.rs`, `app/CODEMAP.md` | `config.rs` |
| 全局快捷键命令及确认 | 控制 | `control.rs` | `app.rs`, `config.rs` |
| 验收真实模型 | 验证 | `verify.rs` | `model.rs`, `render.rs`, `tracking.rs` |

## 子目录

| 目录 | 领域 | 依赖 | 职责 |
| --- | --- | --- | --- |
| [app/](app/CODEMAP.md) | UI | 父级 `app.rs` 状态与动作 | 原生 UI 展示与交互 |

## 文件

| 文件 | 领域 | 职责 |
| --- | --- | --- |
| `main.rs` | 入口 | 解析 CLI，分流控制 / 检查 / 桌面模式，报告运行失败 |
| `app.rs` | UI | Studio 状态、工作线程结果、追踪 / 模型 / GPU 调度与自动保存 |
| `tracking.rs` | 追踪 | 严格解析 OpenSeeFace 多脸数据，转换 12 个通道并校准 / 平滑 |
| `config.rs` | 配置 | 映射、热键、模型档案、输出设置、旧品牌配置的一次性导入及原子 JSON 保存 |
| `model.rs` | 模型 | 查找模型清单，加载 Mocari，应用动作 / 追踪 / 表情 / 物理 / 手动参数 |
| `render.rs` | 渲染 | wgpu 网格与遮罩渲染、共享透明纹理、PNG 读回 |
| `control.rs` | 控制 | 本机 UDP 命令、按命令端口隔离实例和执行结果确认 |
| `verify.rs` | 验证 | 模型报告及无窗口 GPU + UDP 端到端验证 |

## 数据流与依赖

`main → app → tracking.values → config.Mapping → model.update → render.Renderer → 透明纹理 → app/ui`

`control → config.Action → app.execute` 与 UI 按钮 / F 键共用动作处理，避免两套直播操作语义。
