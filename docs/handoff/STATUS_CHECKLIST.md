# Status Checklist

## Completed

- [x] Rust workspace (keystats-core + keystats-daemon + keystatsctl)
- [x] evdev 输入管线（键盘 + 鼠标 + 滚动）
- [x] SQLite 持久化（按天存储）
- [x] D-Bus API（GetTodayStats, GetHistory, GetTopKeys, GetSettings 等）
- [x] GNOME Shell 扩展 — 面板指示器 + 弹出面板 + 偏好设置
- [x] KPS/CPS 实时速率 + 峰值追踪
- [x] 7 天历史柱状图
- [x] 按键详情（前 15 键，3 列布局）
- [x] 系统主题自适应（深色/浅色）
- [x] i18n gettext 国际化（英文 + 简体中文）
- [x] systemd 用户服务 + udev 规则
- [x] USB/蓝牙设备热插拔重扫描
- [x] 根 Makefile 统一构建/安装/打包
- [x] 项目独立拆分（新仓库 KeyStats.Linux）
- [x] D-Bus 命名符合规范（x0x5c0f 前缀）
- [x] 隐私描述文案优化
- [x] 本地构建安装验证通过

## Partially Complete

- [ ] **GNOME 扩展 UI 风格** — Key Breakdown pill badge 样式可优化

## Not Started

- [ ] **Dynamic Accent Color**（中优先级）— GSettings `dynamic-color` 键和 prefs.js 开关已有，逻辑未实现
- [ ] **Fedora GNOME 冒烟测试**（中优先级）
- [ ] **deb/rpm 打包验证**（低优先级）— cargo-deb/cargo-rpm 命令已记录，未实测
- [ ] **Per-app 统计**（研究）— Wayland 限制
- [ ] **KDE/其他 DE 支持**（研究）
- [ ] **extensions.gnome.org 提交审核**（低优先级）
- [ ] **GitHub 仓库创建** — `https://github.com/0x5c0f/KeyStats.Linux`
- [ ] **键盘热力图** — 明确不做（与隐私设计冲突）

## Priority Order

1. Popup UI 优化（高）→ 2. Dynamic Accent Color（中）→ 3. Fedora 冒烟测试（中）→ 4. deb/rpm 打包（低）→ 5. Per-app 统计（研究）→ 6. KDE 支持（研究）
