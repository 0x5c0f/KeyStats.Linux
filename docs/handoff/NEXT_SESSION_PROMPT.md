# Next Session Prompt

## Context

我们正在独立开发 KeyStats.Linux — 一个从 KeyStats fork 中拆分出来的独立项目。新仓库位于 `/home/cxd/Projects/aiediter/KeyStats.Linux`，已通过本地验证（`make build && make install` 成功）。

## Files to Read First

- `README.md` / `README.zh-CN.md` — 项目概览
- `docs/handoff/PROJECT_HANDOFF.md` — 架构和标识符
- `docs/handoff/STATUS_CHECKLIST.md` — 当前完成/待办状态

## Current Phase

- 核心 Linux 移植已完成（daemon + CLI + GNOME 扩展）
- i18n 已实现（英文 + 简体中文）
- 项目已从上游 fork 拆分为独立仓库
- 原仓库 `feat/linux-gnome-next` 分支仅保留 i18n 提交
- 后续所有开发在新仓库进行

## Strict Constraints

1. **GNOME 原生嵌合** — 使用 GNOME Shell 标准能力，不引入非标准依赖
2. **与上游低耦合** — 代码与 macOS KeyStats 完全独立
3. **每次功能更新后，用户确认功能有效前不要 commit**
4. **同类修改合并为一个 commit，避免过多琐碎提交**
5. **开发过程中遇到需要安装的外部依赖，先告知用户**

## Next Recommended Task

从待办清单中选取下一项（优先级从高到低）：

1. **Popup UI 优化**（高）— Key Breakdown pill badge 样式
2. **Dynamic Accent Color**（中）— GSettings 键和 prefs.js 开关已就位，缺少逻辑实现
3. **Fedora GNOME 冒烟测试**（中）

## Verification

```bash
cd /home/cxd/Projects/aiediter/KeyStats.Linux
make build    # 编译通过
make install  # 安装成功
keystatsctl status
```
