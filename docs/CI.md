# CI 与发布制度

主分支通过 PR 合入；发布只从主分支已有提交创建 `vMAJOR.MINOR.PATCH` 标签。CI 使用锁文件，不自动格式化、修改源码或合并依赖更新。

## 必需检查

PR 与 main 推送先比较实际 Git 差异。只修改根目录使用 / 开发 / 许可说明、更新记录、`talk.md`、各级 `CODEMAP.md`、`docs/**/*.md`、PR 模板或字体 README 时，仅运行轻量 `scope` 与 `required`，跳过 Rust、GPU、窗口和打包任务。文档与代码混改、依赖、工作流、构建配置及未知路径均运行完整 CI；删除或把代码重命名为文档也不会获得豁免。

保留轻量工作流是为了让受保护分支获得必需的检查状态，避免整个工作流被路径过滤跳过后一直等待。分类或 Git 历史读取失败时不会放行豁免。周一 03:23 UTC 定时、手动运行及版本发布始终运行完整 CI，发布显式强制全套检查。

| 检查 | 环境 | 通过条件 |
| --- | --- | --- |
| `scope` | Ubuntu 24.04、Python / Git | 验证文档分类的回归用例，再判断本次差异 |
| `quality` | Ubuntu 24.04、固定 Rust 工具链 | `cargo fmt --check`、Python 语法检查、固定版本 actionlint |
| `test (1.95.0)` | Ubuntu 24.04、最低支持 Rust | `cargo test --locked`，包括真实 localhost UDP 测试 |
| `test (1.98.1)` | Ubuntu 24.04、发布工具链 | 同上，以及 Clippy 全 targets、警告作为错误 |
| `linux` | Ubuntu 22.04、Mesa 软件 Vulkan、Xvfb | release 打包；解压后检查许可、源码、版本、校验和与 glibc ≤ 2.35；真实模型 GPU / UDP 验证；透明 / 绿幕双窗口截图及重启恢复 |
| `required` | 汇总 | 分类成功；完整检查要求每项成功，文档模式要求重任务明确跳过；失败或取消不放行 |

模型固定为 Mocari 仓库提交 `13c7ed3c1e6471bcc1aacf5e7d3fdb1d038ed759` 的 Haru，独立下载到忽略目录，仅用于测试。测试使用软件 GPU 和合成面捕输入；摄像头质量、原生 Wayland / PipeWire 捕获及 OBS 实录仍按 [验证记录](VERIFICATION.md) 在实际桌面验收。

CI 工件保留 7 天：便携压缩包、校验文件与数值 JSON 报告。样例模型、模型截图、用户配置与本地录像不上传。发布包包含可重建源码、依赖许可证与 `build-info.json`；公开发布二进制从干净的 Ubuntu runner 构建，不沿用开发机的构建缓存。

## 合并门禁

GitHub main 分支要求 PR、最新基线上的 GitHub Actions `required` 检查、对话已解决及线性历史，禁止强推与删除，管理员也受约束。当前单维护者模式要求 0 个人工批准，维护者仍需检查改动动机与验证证据后合并。后续增加维护者时可提高批准人数。

仓库只启用 squash / rebase 合并，合并后删除开发分支。调整必需检查名称时，工作流和分支保护配置必须同步，确认新的检查已实际运行后再更新门禁。

策略文件不会自行修改 GitHub 设置。维护者应用 / 审计策略：

```sh
gh api --method PUT repos/AntheaLaffy/vtb-in-linux/branches/main/protection \
  --input .github/branch-protection.json
gh api repos/AntheaLaffy/vtb-in-linux/branches/main/protection
gh api repos/AntheaLaffy/vtb-in-linux/rulesets
```

首次建立标签保护使用 `gh api --method POST repos/AntheaLaffy/vtb-in-linux/rulesets --input .github/tag-rules.json`。已有规则需按其 ID 更新，避免重复创建。`v*` 标签允许新建，禁止改写或删除。

Dependabot 每周为 Cargo 与 GitHub Actions 创建更新 PR，统一经过完整门禁，不自动合并。Actions 固定完整提交 SHA。每月检查 Rust 工具链、actionlint 版本与测试样例固定提交；升级时同步工具链文件、矩阵、文档及校验摘要，重新运行全部检查。定时 CI 发现的失败先修复再发布。

## 版本发布

1. 在开发分支更新 `Cargo.toml`、`Cargo.lock` 与 `CHANGELOG.md`，完成对应产品行为检查，经 PR 合入 main。
2. 确认 main CI 通过，并按改动范围完成真实 Wayland / OBS 验收。首次 0.1.0 的记录见 [VERIFICATION.md](VERIFICATION.md)。
3. 在该提交创建并推送不可改写的版本标签：

   ```sh
   git switch main
   git pull --ff-only
   git tag -a v0.1.0 -m 'Kage Studio 0.1.0'
   git push origin v0.1.0
   ```

4. `Release` 校验标签格式、Cargo 版本、更新记录和 main 祖先关系，重新运行完整 CI。全部通过后，下载同一次运行中已经解压验证的压缩包，复核 SHA-256、源码提交与干净状态，再发布 GitHub Release。`0.x` 标为实验性预发布。
5. 发布资产为 `kage-studio-VERSION-linux-x86_64.tar.gz` 与 `SHA256SUMS`。从 Release 下载后校验、解压并运行 `./kage-studio-linux-x86_64/kage-studio --version`。Ubuntu 22.04 / glibc 2.35 是二进制基线，仍需要系统图形驱动和桌面 Portal。

发布工作流只有最终上传 job 获得 `contents: write`，其余检查保持只读权限；不会接收 PR 代码的写权限事件。失败时修复原因，再针对同一标签手动重跑 `Release`；已经发布的版本不覆盖资产，后续修复使用新版本号。

在本机重现打包与 CI 冒烟检查（需要 Pillow、Xvfb、xauth、readelf 和可用 Vulkan 驱动）：

```sh
python3 tools/package.py --archive
python3 tools/ci_smoke.py --package dist/kage-studio-0.1.0-linux-x86_64.tar.gz \
  --model /path/to/Haru.model3.json --output-dir .local/ci-local
```

本机检查不传 `--max-glibc 2.35`，因为本机 glibc 可能较新；`--require-clean` 专供提供 `GITHUB_SHA` 的 CI。源码验收与二进制可移植性是独立检查。
