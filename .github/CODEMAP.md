# GitHub 自动化

| 任务 | Target | Also Check |
| --- | --- | --- |
| 修改合并检查 | `workflows/ci.yml` | `../docs/CI.md`, `../tools/ci_smoke.py` |
| 修改版本发布 | `workflows/release.yml` | `../tools/package.py`, `../CHANGELOG.md` |
| 修改主分支门禁 | `branch-protection.json` | `../docs/CI.md`；JSON 变更后需由维护者应用到 GitHub |
| 保护已发布版本标签 | `tag-rules.json` | `workflows/release.yml`, `../docs/CI.md` |

| 文件 | 职责 |
| --- | --- |
| `workflows/ci.yml` | PR、主分支、定时与发布复用的检查及 Linux 便携包验证 |
| `workflows/release.yml` | 校验版本和主分支来源，复用 CI，上传校验后的发布包 |
| `branch-protection.json` | 可重放的 main 分支保护策略 |
| `tag-rules.json` | 禁止更新或删除 `v*` 标签的仓库规则集 |
| `dependabot.yml` | 每周依赖与 Actions 更新 PR，不自动合并 |
| `pull_request_template.md` | 变更动机、实际验证及地图维护要求 |
