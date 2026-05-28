# Next Session Prompt

## Context

KeyStats.Linux — 隐私优先的 Linux 键盘鼠标统计工具，v0.1.0 已发布。仓库位于 `/home/cxd/Projects/aiediter/KeyStats.Linux`，远程 `https://github.com/0x5c0f/KeyStats.Linux`。

## Files to Read First

- `README.md` / `README.zh-CN.md` — 项目概览
- `docs/handoff/PROJECT_HANDOFF.md` — 架构和标识符
- `docs/handoff/STATUS_CHECKLIST.md` — 当前完成/待办状态

## Current Phase

- 核心功能已完成，v0.1.0 已发布
- extensions.gnome.org 已提交审核
- CI/CD 为 tag 驱动发布（手动推送标签触发）
- 分支策略：release（开发）→ main（发布基线）

## Strict Constraints

1. **GNOME 原生嵌合** — 使用 GNOME Shell 标准能力，不引入非标准依赖
2. **与上游低耦合** — 代码与 macOS KeyStats 完全独立
3. **每次功能更新后，用户确认功能有效前不要 commit**
4. **同类修改合并为一个 commit，避免过多琐碎提交**
5. **开发过程中遇到需要安装的外部依赖，先告知用户**
6. **任何改动先从 release 创建子分支（feat/ 或 fix/），不要在 release 上直接修改**
7. **CLAUDE.md 等工具配置文件不纳入提交**

## Next Recommended Task

剩余研究类任务（按优先级）：

1. **Per-app 统计**（研究）— Wayland 下无法获取窗口信息，需调研方案
2. **KDE/其他 DE 支持**（研究）— 需调研 KDE 面板集成方式

## Verification

```bash
cd /home/cxd/Projects/aiediter/KeyStats.Linux
make build    # 编译通过
make install  # 安装成功
keystatsctl status
```
