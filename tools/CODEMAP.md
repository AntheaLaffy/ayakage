# 开发工具

工具服务可重复检查与交付，不是应用运行的必要依赖。

## 任务导航

| 任务 | 领域 | Target | Also Check |
| --- | --- | --- | --- |
| 生成当前系统分发目录 | 打包 | `package.py` | `../Cargo.toml`, `../THIRD_PARTY.md` |
| 检查实际发布压缩包 | 验证 | `ci_smoke.py` | `package.py`, `../.github/workflows/ci.yml` |
| 修改纯文档 CI 豁免 | CI | `ci_scope.py`, `test_ci_scope.py` | `../.github/workflows/ci.yml`, `../docs/CI.md` |
| 无摄像头检查追踪连接 | 验证 | `send_tracking.py` | `../src/tracking.rs` |
| 更新内嵌界面字体 | 字体 | `subset_font.py` | `../src/app/ui/`, `../assets/fonts/README.md` |

## 文件

| 文件 | 领域 | 职责 |
| --- | --- | --- |
| `package.py` | 打包 | Cargo release 构建、收集二进制 / 可重建源码 / 依赖许可及校验信息 |
| `ci_smoke.py` | 验证 | 解压校验、glibc 基线、实际模型 GPU / UDP 及 Xvfb 双窗口 / 重启检查 |
| `ci_scope.py` | CI | 按实际 Git 差异选择完整检查；未知历史、标签和定时任务保守运行全套 |
| `test_ci_scope.py` | CI | 用真实临时 Git 历史验证文档豁免、代码混改、重命名及强制检查 |
| `send_tracking.py` | 验证 | 明确标注的合成 OpenSeeFace UDP 发送器 |
| `subset_font.py` | 字体 | 从 Noto CJK 字体提取源代码实际使用的字符 |
