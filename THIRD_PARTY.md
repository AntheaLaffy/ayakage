# 第三方组件

| 组件 | 使用方式 | 许可 / 来源 |
| --- | --- | --- |
| Mocari 0.4.0 | `.moc3` 解析、参数 / 动作 / 表情 / 物理和 wgpu 渲染器 | MIT；[Eatgrapes/Mocari](https://github.com/Eatgrapes/Mocari)，Copyright (c) 2026 Eatgrapes |
| egui / eframe 0.37 | 原生 UI、多窗口、输入和纹理显示 | MIT / Apache-2.0；[emilk/egui](https://github.com/emilk/egui) |
| wgpu 30 | 图形设备与 GPU 渲染 | MIT / Apache-2.0；[gfx-rs/wgpu](https://github.com/gfx-rs/wgpu) |
| rfd | 系统文件选择器 | MIT；[PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| Noto Sans CJK SC | 内嵌中文界面字体子集 | SIL OFL 1.1；[notofonts/noto-cjk](https://github.com/notofonts/noto-cjk)，字体记录 Copyright © 2014–2021 Adobe；见 `assets/fonts/OFL.txt` |
| OpenSeeFace | 独立的可选面捕进程；本项目实现兼容数据接收 | BSD-2-Clause；[emilianavt/OpenSeeFace](https://github.com/emilianavt/OpenSeeFace) |

所有 Rust 依赖的确切版本见 `Cargo.lock`。`tools/package.py` 会收集依赖包自带的许可证 / NOTICE，放入分发目录的 `licenses/`。

本项目没有分发官方 Cubism Core、官方 SDK 或任何 Live2D 模型。验证过程中使用的外部样例保留其自身许可，不随本项目发布。Live2D / Cubism / VTube Studio 等名称仅用于说明格式与目标兼容性。
