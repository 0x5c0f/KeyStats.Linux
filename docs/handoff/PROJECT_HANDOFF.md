# KeyStats.Linux — Project Handoff

## Purpose

KeyStats.Linux 是隐私优先的 Linux 键盘鼠标统计工具，追踪输入统计数据（按键频次、点击次数、移动距离），从原始 evdev 事件中仅提取聚合计数和距离，丢弃原始事件数据。

受 [KeyStats](https://github.com/debugtheworldbot/KeyStats)（macOS & Windows）启发，这是一个独立的 Linux 实现。

## Repository

- 本地路径：`/home/cxd/Projects/aiediter/KeyStats.Linux`
- 远程：`https://github.com/0x5c0f/KeyStats.Linux`
- v0.1.0 已发布（tag 驱动发布流程）

## Architecture

```
┌──────────────────────┐     D-Bus (会话总线)     ┌─────────────────────┐
│  GNOME Shell 扩展     │◄──────────────────────►│   keystats-daemon   │
│  (GJS / St / GTK)    │                         │   (Rust / evdev)    │
└──────────────────────┘                         └─────────┬───────────┘
                                                           │
                                                 ┌─────────▼───────────┐
                                                 │    keystatsctl       │
                                                 │    (CLI)             │
                                                 └──────────────────────┘
```

- **keystats-daemon** — 读取 `/dev/input/event*` via evdev，聚合统计数据，持久化到 SQLite，通过 D-Bus 暴露 API
- **GNOME Shell 扩展** — 面板指示器 + 弹出面板 + 偏好设置（通过 D-Bus 消费数据）
- **keystatsctl** — CLI 诊断工具（status / doctor / device list）

## Tech Stack

- **Rust** 1.70+ — 3 workspace crates (keystats-core, keystats-daemon, keystatsctl)
- **evdev** — Linux input event 读取
- **zbus** — D-Bus 会话总线通信
- **rusqlite** — SQLite 持久化
- **GJS** (JavaScript) — GNOME Shell 扩展（GNOME 45+）
- **GNU gettext** — 国际化（pot/po/mo）
- **Make** — 构建/安装/打包

## Key Identifiers

| 项目 | 值 |
|------|-----|
| GNOME 扩展 UUID | `keystats@0x5c0f.github.io` |
| D-Bus 总线名 | `io.github.x0x5c0f.KeyStats` |
| D-Bus 对象路径 | `/io/github/0x5c0f/KeyStats` |
| D-Bus 接口 | `io.github.x0x5c0f.KeyStats1` |
| GSettings schema | `org.gnome.shell.extensions.keystats` |
| gettext domain | `keystats` |

## Directory Layout

```
KeyStats.Linux/
├── Makefile                    ← 根构建系统
├── Cargo.toml / Cargo.lock     ← Rust workspace
├── crates/
│   ├── keystats-core/          ← 共享数据模型、格式化
│   ├── keystats-daemon/        ← evdev 管线、SQLite、D-Bus
│   └── keystatsctl/            ← CLI
├── gnome-extension/
│   ├── extension.js            ← 面板 + 弹出 UI
│   ├── prefs.js                ← 偏好设置窗口
│   ├── stylesheet.css          ← 双主题 CSS
│   ├── schemas/                ← GSettings XML
│   ├── po/                     ← gettext 源文件
│   ├── locale/                 ← 编译后的 .mo 文件
│   └── Makefile                ← 扩展构建/安装
├── packaging/
│   ├── systemd/keystats.service ← 用户 systemd 单元
│   ├── udev/60-keystats-input.rules ← 输入设备权限
│   └── dist/                   ← 二进制分发包（Makefile + README）
├── docs/
│   ├── images/                 ← 扩展截图
│   ├── superpowers/specs/      ← 设计规格
│   ├── superpowers/plans/      ← 实现计划
│   └── handoff/                ← 交接文档
└── .github/workflows/
    └── release.yml             ← tag 驱动发布 CI
```

## Build & Install

```bash
make build    # cargo build --release + locale compile
make install  # daemon + systemd + extension (glib-compile-schemas included)
make zip      # extension zip for gnome-extensions install
make dist     # 打包二进制分发包（tarball）
make test     # cargo test
make check    # cargo check + fmt + clippy
make clean    # clean all artifacts
```

安装路径：`~/.local/bin/`（用户级，无需 sudo）

## Branch & Release Strategy

- **release** — 开发分支，所有功能从 release 创建 feat/fix 子分支
- **main** — 发布基线，最新代码不等于最新发布
- **发布流程**：手动推送 tag 触发
  - `v*-rc.*` 标签 → 预发布（prerelease）
  - `v*` 标签 → 正式发布
- release 分支推送不触发任何发布

## Design Principles

1. **GNOME 原生嵌合** — 使用 GNOME Shell 标准能力（gettext、GJS、Adw/GTK4、GSettings）
2. **与上游低耦合** — 代码与 macOS KeyStats 完全独立，不共享任何代码
3. **最小侵入** — 新功能通过新文件/模块扩展，不重写现有结构

## Completed Capabilities

- evdev 输入管线（键盘 + 鼠标 + 滚动）
- SQLite 持久化（按天存储）
- D-Bus API（GetTodayStats, GetHistory, GetTopKeys, GetSettings 等）
- GNOME 面板指示器 + 弹出面板 + 偏好设置
- KPS/CPS 实时速率 + 峰值追踪
- 7 天历史柱状图
- 按键详情（前 15 键，3 列布局）
- 系统主题自适应（深色/浅色）
- i18n 国际化（英文 + 简体中文，gettext）
- systemd 用户服务 + udev 输入权限规则
- USB/蓝牙设备热插拔重扫描
- 根 Makefile 统一构建/安装
- 二进制分发包（tarball + extension zip）
- GitHub Actions CI/CD（tag 驱动发布）
- extensions.gnome.org 提交审核

## Known Constraints

- 无 per-app 统计（Wayland 限制）
- 仅 GNOME Shell 45+
- 不记录按键内容或鼠标路径（隐私设计）

## Verification

```bash
make build && make install
systemctl --user enable --now keystats
gnome-extensions enable keystats@0x5c0f.github.io
keystatsctl status
```
