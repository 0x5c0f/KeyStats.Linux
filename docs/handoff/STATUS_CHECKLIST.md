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
- [x] Popup UI 优化 — Key Breakdown pill badge 样式改进
- [x] Dynamic Accent Color 开关已禁用（功能搁置）
- [x] keystatsctl status D-Bus 修复
- [x] 二进制分发包（tarball + extension zip）
- [x] GitHub Actions CI/CD（tag 驱动发布）
- [x] GitHub 仓库创建 — `https://github.com/0x5c0f/KeyStats.Linux`
- [x] Fedora GNOME 冒烟测试通过
- [x] extensions.gnome.org 提交审核（等待审核中）
- [x] README 截图嵌入（中英文）

## Not Started

- [ ] **Per-app 统计**（研究）— Wayland 限制
- [ ] **KDE/其他 DE 支持**（研究）

## Deferred

- **Dynamic Accent Color** — GSettings 键和 prefs.js 开关已有，逻辑暂不实现
- **键盘热力图** — 明确不做（与隐私设计冲突）
- **deb/rpm 打包** — 保持现有 tarball + zip 安装模式

## Priority Order

1. Per-app 统计（研究）→ 2. KDE 支持（研究）
