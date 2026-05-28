# Linux / GNOME 版 KeyStats 设计

- 日期：2026-05-25
- 作者：Codex
- 状态：Accepted defaults - 可进入 Phase 0 / Phase 1
- 范围：KeyStats Linux 版采集服务、GNOME Shell 顶栏扩展、后续跨桌面扩展基础

## 0. 已确认默认决策

2026-05-25 已确认以下默认决策，后续开发按这些边界推进：

- **目标平台**：优先支持 Ubuntu GNOME + Fedora GNOME，GNOME Shell 45+。
- **技术栈**：daemon 使用 Rust；GNOME 顶栏 UI 使用 GJS Shell extension；不复用 Swift / C# 代码，只复用产品逻辑和数据模型概念。
- **权限策略**：MVP 使用普通用户 daemon + `input` 组或专用 `keystats` 组 + udev rule；不做 root 常驻 daemon。
- **第一版功能边界**：先做按键、点击、鼠标距离、滚动、KPS/CPS、历史、D-Bus、GNOME 顶栏；不做按应用统计。
- **架构路线**：`keystats-daemon` 是核心，GNOME Shell extension 是 UI 外壳；纯 GNOME 扩展不作为主路径。

## 1. 背景

KeyStats 当前已有两个成熟方向：

- macOS：`KeyStats.app` + `KeyStatsHelper`。主 App 负责 UI、统计聚合、持久化；Helper 负责高权限输入事件采集。
- Windows：WPF tray app。输入采集、统计、UI 都在 Windows 端独立实现。

Linux 版的关键问题不是 UI，而是**全局输入事件采集权限**。GNOME Shell 扩展可以很好地做顶栏 UI，但不适合作为核心采集进程：

1. GNOME Shell 扩展运行在 `gnome-shell` 进程内，扩展崩溃或阻塞会影响桌面稳定性。
2. 读取 `/dev/input/event*` 通常需要 `root`、`input` 组、udev rule 或 polkit 授权，不适合塞进 Shell 扩展。
3. GNOME 扩展版本兼容成本高，每个 GNOME Shell 主版本都需要明确支持。
4. Wayland 下普通应用不能可靠监听全局键鼠事件；底层 evdev 是更稳定的采集入口。

因此 Linux 版应采用与 macOS 类似的职责拆分：**后台采集服务做核心，GNOME 扩展只做展示与控制入口**。

## 2. 目标 / 非目标

**目标**

- 实现 Linux 上的每日按键数、鼠标点击、鼠标移动距离、滚动距离统计。
- 支持 KPS / CPS 当前值与峰值。
- 支持本地历史统计、导入导出、重置今日数据。
- GNOME 顶栏显示核心数字，点击后展示弹出面板。
- 采集服务可在 GNOME Shell 重启、扩展禁用、UI 崩溃时继续稳定运行。
- 权限模型清晰：只统计 key code / button / relative movement，不记录输入文本、事件序列或窗口内容。
- 为后续 KDE / AppIndicator / GTK 独立 UI 留接口。

**非目标**

- 第一阶段不做按应用统计。Wayland 下前台应用归属不稳定，单独立项。
- 第一阶段不做 Flatpak / Snap 正式沙箱分发。全局输入权限与沙箱模型冲突较大。
- 第一阶段不追求所有桌面环境同等体验。先支持 GNOME 45+。
- 不记录原始输入事件日志，不提供回放能力。
- 不在 GNOME 扩展内直接读取 `/dev/input/event*`。
- 不使用 root 常驻进程作为主设计；优先使用受限用户服务 + 明确授权。

## 3. 推荐架构

```
┌──────────────────────────────────────┐
│ GNOME Shell extension                 │
│                                      │
│  - top bar indicator                 │
│  - popup stats panel                 │
│  - settings shortcut                 │
│  - calls daemon over D-Bus           │
└───────────────────▲──────────────────┘
                    │ session D-Bus
┌───────────────────▼──────────────────┐
│ keystats-daemon                       │
│                                      │
│  - evdev device discovery            │
│  - event loop and aggregation        │
│  - daily reset                       │
│  - SQLite persistence                │
│  - notification threshold logic      │
│  - D-Bus API                         │
│  - systemd --user service            │
└───────────────────▲──────────────────┘
                    │ read only
┌───────────────────▼──────────────────┐
│ /dev/input/event*                     │
│ Linux input subsystem / evdev         │
└──────────────────────────────────────┘
```

### 职责切分

**`keystats-daemon`**

- 发现键盘、鼠标、触控板等 evdev 设备。
- 读取 `EV_KEY`、`EV_REL`、`EV_ABS`、`EV_SYN` 等事件。
- 聚合统计数据并持久化。
- 暴露 D-Bus API 给 UI。
- 处理热插拔、睡眠恢复、日期切换。
- 负责权限诊断和状态上报。

**GNOME Shell extension**

- 展示顶栏数字和图标。
- 每 0.5-2 秒拉取或订阅 daemon snapshot。
- 展示今日统计、近期历史、权限错误提示。
- 提供重置、打开设置、打开日志、退出 daemon 等动作入口。
- 不做输入监听，不持有高权限，不落盘核心统计数据。

## 4. 技术选型

### 4.1 daemon 语言

推荐：Rust。

理由：

- Linux evdev / udev / zbus 生态成熟。
- 单二进制易分发。
- 长期运行服务更容易控制内存安全。
- 与现有 Swift/AppKit 代码耦合度低，重写成本可控。

候选依赖：

| 领域 | 推荐 crate | 说明 |
|---|---|---|
| 输入事件 | `evdev` | 读取 `/dev/input/event*` |
| 设备发现 | `udev` | 发现 input 设备、监听热插拔 |
| D-Bus | `zbus` | session bus API |
| 存储 | `rusqlite` | 本地 SQLite |
| 时间 | `time` 或 `chrono` | 日期 reset、历史查询 |
| 日志 | `tracing` + `tracing-subscriber` | systemd journal 友好 |
| 配置 | `serde` + `toml` | 设置文件 |

### 4.2 UI

推荐：GNOME Shell extension，GJS 编写。

理由：

- 产品形态最接近 macOS menu bar。
- GNOME 官方支持 Shell extension 作为顶栏扩展方式。
- 可以通过 GJS 使用 D-Bus client 与 daemon 通信。

约束：

- `extension.js` 运行在 `gnome-shell` 进程内，必须轻量、异步、可清理。
- 不在 extension 内 spawn 特权进程。
- 不在 extension 内执行高频采集逻辑。
- 支持的 GNOME 版本要显式列在 `metadata.json`。

## 5. 目录结构

建议新增：

```
KeyStats.Linux/
├── Cargo.toml
├── crates/
│   ├── keystats-core/
│   │   ├── src/lib.rs
│   │   ├── src/model.rs
│   │   ├── src/format.rs
│   │   └── src/import_export.rs
│   ├── keystats-daemon/
│   │   ├── src/main.rs
│   │   ├── src/input/
│   │   │   ├── mod.rs
│   │   │   ├── device.rs
│   │   │   ├── event_loop.rs
│   │   │   └── keymap.rs
│   │   ├── src/stats/
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs
│   │   │   └── rates.rs
│   │   ├── src/db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs
│   │   │   └── migrations.rs
│   │   ├── src/dbus/
│   │   │   ├── mod.rs
│   │   │   └── service.rs
│   │   └── src/permissions.rs
│   └── keystatsctl/
│       ├── src/main.rs
│       └── src/commands.rs
├── packaging/
│   ├── systemd/keystats.service
│   ├── udev/60-keystats-input.rules
│   ├── debian/
│   └── rpm/
└── README.md

KeyStats.GNOME/
└── keystats@debugtheworldbot.github.io/
    ├── metadata.json
    ├── extension.js
    ├── prefs.js
    ├── stylesheet.css
    ├── schemas/
    │   └── org.gnome.shell.extensions.keystats.gschema.xml
    └── locale/
```

## 6. 输入事件采集设计

### 6.1 evdev 事件范围

daemon 读取 Linux input subsystem 的 evdev 事件：

- `EV_KEY`
  - `KEY_*`：键盘按下 / 释放 / 自动重复。
  - `BTN_LEFT`、`BTN_RIGHT`、`BTN_MIDDLE`、`BTN_SIDE`、`BTN_EXTRA` 等鼠标按钮。
- `EV_REL`
  - `REL_X` / `REL_Y`：鼠标相对移动。
  - `REL_WHEEL` / `REL_HWHEEL` / high resolution wheel code：滚轮。
- `EV_ABS`
  - 触控板和部分设备可能以绝对坐标上报，需要先作为后续增强。
- `EV_SYN`
  - 事件包边界，用于合并一次物理动作产生的多个 event。

### 6.2 计数规则

| 指标 | 规则 |
|---|---|
| keyPresses | `EV_KEY` 且 code 为 `KEY_*`，value == 1 时 +1 |
| auto repeat | value == 2 默认忽略，与 macOS 当前行为保持一致 |
| leftClicks | `BTN_LEFT` value == 1 |
| rightClicks | `BTN_RIGHT` value == 1 |
| sideBackClicks | `BTN_SIDE` value == 1 |
| sideForwardClicks | `BTN_EXTRA` value == 1 |
| mouseDistance | `sqrt(dx^2 + dy^2)`，从 `REL_X/REL_Y` 聚合 |
| scrollDistance | `abs(REL_WHEEL)` / `abs(REL_HWHEEL)` 归一化为 px-like 单位 |
| KPS / CPS | trailing 1-second sliding window |
| peakKPS / peakCPS | 今日窗口最大值 |

### 6.3 隐私边界

daemon **允许**持久化：

- 日期聚合统计。
- 按 key code / 标准键名聚合的次数。
- 鼠标按钮聚合次数。
- 鼠标移动和滚动累计距离。

daemon **禁止**持久化：

- 原始事件序列。
- 精确时间戳序列。
- 输入文本。
- 窗口标题、网页标题、输入框内容。
- 鼠标绝对位置轨迹。

如果未来做按应用统计，只允许存储 bundle/app id 或 desktop file id + 聚合计数，不存窗口标题。

## 7. 权限模型

### 7.1 基础事实

大多数发行版上 `/dev/input/event*` 属于 `root:input` 或通过 `uaccess` ACL 分配给本地登录用户。不同发行版差异较大，必须在设计中承认并检测。

### 7.2 MVP 授权方案

优先级：

1. 如果当前用户已经可读目标 event devices，直接运行。
2. 如果用户属于 `input` 组，提示重新登录后生效。
3. 如果没有权限，安装器提供可选 udev rule / group 配置。
4. 不默认要求 daemon 以 root 常驻。

建议安装器创建：

```
groupadd --system keystats
usermod -aG keystats "$USER"
```

udev rule 示例：

```
SUBSYSTEM=="input", KERNEL=="event*", GROUP="keystats", MODE="0640"
```

注意：这个 rule 较宽，必须在安装时明确提示风险。更精细的 rule 可以按设备 capability 过滤键盘/鼠标，但实现和兼容性会复杂。

### 7.3 权限诊断 API

daemon 启动时输出结构化状态：

```json
{
  "can_read_any_input": true,
  "readable_devices": 3,
  "blocked_devices": 2,
  "recommended_action": "none | add_group | install_udev_rule | run_installer",
  "message": "..."
}
```

GNOME 扩展只展示诊断结果，不执行提权。

## 8. D-Bus API

使用 session bus，service name：

```
io.github.debugtheworldbot.KeyStats
```

object path：

```
/io/github/debugtheworldbot/KeyStats
```

interface：

```
io.github.debugtheworldbot.KeyStats1
```

### 8.1 Methods

```
GetTodayStats() -> a{sv}
GetRates() -> a{sv}
GetHistory(days: u) -> aa{sv}
GetSettings() -> a{sv}
SetSettings(settings: a{sv}) -> b
ResetToday() -> b
ExportData() -> s
ImportData(json: s, mode: s) -> b
GetPermissionStatus() -> a{sv}
```

### 8.2 Signals

```
StatsChanged(a{sv} snapshot)
PermissionChanged(a{sv} status)
SettingsChanged(a{sv} settings)
```

### 8.3 Snapshot 字段

```
date: string
keyPresses: uint64
leftClicks: uint64
rightClicks: uint64
sideBackClicks: uint64
sideForwardClicks: uint64
totalClicks: uint64
mouseDistance: double
scrollDistance: double
currentKPS: uint32
currentCPS: uint32
peakKPS: uint32
peakCPS: uint32
updatedAt: string
```

## 9. 数据存储

推荐 SQLite，路径遵循 XDG：

```
$XDG_STATE_HOME/keystats/stats.sqlite3
```

默认：

```
~/.local/state/keystats/stats.sqlite3
```

配置文件：

```
$XDG_CONFIG_HOME/keystats/config.toml
```

日志：

```
systemd journal
```

### 9.1 Schema 草案

```sql
CREATE TABLE daily_stats (
    date TEXT PRIMARY KEY,
    key_presses INTEGER NOT NULL DEFAULT 0,
    left_clicks INTEGER NOT NULL DEFAULT 0,
    right_clicks INTEGER NOT NULL DEFAULT 0,
    side_back_clicks INTEGER NOT NULL DEFAULT 0,
    side_forward_clicks INTEGER NOT NULL DEFAULT 0,
    mouse_distance REAL NOT NULL DEFAULT 0,
    scroll_distance REAL NOT NULL DEFAULT 0,
    peak_kps INTEGER NOT NULL DEFAULT 0,
    peak_cps INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE key_counts (
    date TEXT NOT NULL,
    key_name TEXT NOT NULL,
    count INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, key_name)
);

CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### 9.2 写入策略

- 内存中实时累计。
- 每 2 秒或事件 burst 结束后 flush。
- 退出前 flush。
- 日期切换时先保存旧日期，再创建新日期。
- SQLite 使用 WAL，单 writer。

## 10. GNOME 扩展设计

### 10.1 顶栏

显示策略：

- 图标 + 两行数字，类似 macOS：上方 keys，下方 clicks。
- 用户可配置只显示 keys、只显示 clicks、显示图标颜色。
- D-Bus 不可用时显示 warning icon，点击弹出权限/服务状态。

### 10.2 弹出面板

MVP 内容：

- 今日按键数。
- 今日点击数，拆分左右键与侧键。
- 鼠标移动距离。
- 滚动距离。
- 当前 KPS/CPS 与峰值。
- 最近 7 天简易历史。
- Reset 按钮。
- Settings 按钮。

### 10.3 设置

扩展设置只保存 UI 偏好：

- 是否显示 keys。
- 是否显示 clicks。
- 刷新频率。
- 是否使用动态颜色。

核心设置存到 daemon config，并通过 D-Bus 设置。

## 11. 安装与发布

### 11.1 开发期

```
cargo build -p keystats-daemon
cargo build -p keystatsctl
systemctl --user link ./KeyStats.Linux/packaging/systemd/keystats.service
systemctl --user enable --now keystats.service
gnome-extensions install KeyStats.GNOME/keystats@debugtheworldbot.github.io.zip
```

### 11.2 发行包

第一阶段建议：

- `.deb`：Ubuntu / Debian / Pop!_OS。
- `.rpm`：Fedora / openSUSE。
- tarball：高级用户手动安装。

包内容：

- `/usr/bin/keystats-daemon`
- `/usr/bin/keystatsctl`
- `/usr/lib/systemd/user/keystats.service`
- `/usr/lib/udev/rules.d/60-keystats-input.rules` 或安装后脚本可选写入
- `/usr/share/gnome-shell/extensions/keystats@debugtheworldbot.github.io/`

## 12. 风险

| 风险 | 严重度 | 说明 | 缓解 |
|---|---:|---|---|
| evdev 权限安装体验差 | 高 | 用户可能不理解 input 权限 | 安装器诊断 + 明确提示 + CLI doctor |
| GNOME Shell 版本兼容 | 中 | 扩展 API 随版本变化 | 明确支持 GNOME 45+，每版 smoke test |
| Wayland 按应用统计受限 | 中 | 无法稳定获得活跃应用 | 第一阶段不做 |
| 多设备重复计数 | 中 | 某些设备同时暴露多个 event node | 基于 capability 和物理路径去重 |
| 触控板滚动单位不一致 | 中 | 不同设备上 REL_WHEEL 粒度不同 | 先定义 px-like 归一化，后续校准 |
| 用户隐私疑虑 | 高 | 读取底层输入事件敏感 | 不存原始事件，文案明确，代码审计点固定 |
| 扩展阻塞 Shell | 高 | GJS 同进程运行 | 所有 D-Bus 调用异步，刷新节流 |

## 13. Spike 清单

实现前先做以下验证：

1. 在 Ubuntu GNOME 24.04 / Fedora GNOME 上读取 evdev 事件。
2. 验证普通用户、`input` 组、udev rule 三种权限路径。
3. 验证 systemd user service 自动启动和崩溃恢复。
4. 验证 GJS 通过 D-Bus 读取 daemon snapshot。
5. 验证 GNOME 45 / 46 / 47 / 48 扩展 API 差异。
6. 验证多键盘、多鼠标、蓝牙设备热插拔。

## 14. 第一版验收标准

- GNOME 顶栏能显示按键数和点击数。
- daemon 重启后数据不丢。
- 键盘输入、鼠标点击、移动、滚动能实时更新。
- 无权限时 UI 能明确提示原因和修复路径。
- 不记录输入文本或事件序列。
- `keystatsctl status` 能输出 daemon、权限、设备、今日统计。
- Ubuntu GNOME 和 Fedora GNOME 各完成一次手工 smoke test。

## 15. 参考

- Linux input subsystem userspace API: https://kernel.org/doc/html/latest/input/input_uapi.html
- Linux evdev introduction: https://docs.kernel.org/input/input.html
- Linux input event codes: https://docs.kernel.org/input/event-codes.html
- GNOME Shell extension anatomy: https://gjs.guide/extensions/overview/anatomy.html
- GNOME Shell extension architecture: https://gjs.guide/extensions/overview/architecture.html
- GNOME Shell extension review guidelines: https://gjs.guide/extensions/review-guidelines/review-guidelines.html
- GJS Gio.Subprocess guide: https://gjs.guide/guides/gio/subprocesses.html
