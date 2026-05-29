# KeyStats Linux 打包文档

[English](README.md) | 简体中文

所有命令假设你在 **仓库根目录**。除特别说明外，路径均相对于仓库根目录。

```
KeyStats.Linux/                    ← 仓库根目录（你在这里）
├── Cargo.toml                     ← Rust workspace
├── crates/ (keystats-core, keystats-daemon, keystatsctl)
├── gnome-extension/               ← GNOME 扩展源码
├── packaging/ (systemd, udev)
└── ...
```

---

## 前提条件

- **Rust** 1.70+（安装：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）
- **GNOME Shell 45+**
- **input 组** 权限

```bash
sudo usermod -aG input $USER
# 注销重新登录，或执行：newgrp input
```

---

## 开发构建与运行

所有 cargo 命令在 `KeyStats.Linux/` 目录下执行：

```bash
cd KeyStats.Linux

# 构建
cargo build -p keystats-daemon -p keystatsctl

# 运行守护进程（前台，Ctrl+C 停止）
cargo run -p keystats-daemon

# 检查设备权限
cargo run -p keystatsctl -- doctor

# 检查守护进程统计
cargo run -p keystatsctl -- status
```

---

## 安装

### 1. 构建并安装守护进程 + CLI

```bash
cd KeyStats.Linux

# Release 构建
cargo build --release -p keystats-daemon -p keystatsctl

# 安装到 ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/keystats-daemon ~/.local/bin/
cp target/release/keystatsctl ~/.local/bin/

# 确保 ~/.local/bin 在 PATH 中
export PATH="$HOME/.local/bin:$PATH"
```

### 2. 安装 systemd 用户服务

```bash
mkdir -p ~/.config/systemd/user
cp KeyStats.Linux/packaging/systemd/keystats.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now keystats.service

# 验证
systemctl --user status keystats.service
```

### 3. 安装 GNOME Shell 扩展

扩展源码位于 `gnome-extension/`。

**方式 A：打包为 zip 安装**（推荐用于分发）

```bash
cd gnome-extension

# 构建语言包并创建 zip 包
make zip

# 安装
gnome-extensions install keystats@0x5c0f.github.io.zip
```

**方式 B：直接安装**（快速开发迭代）

```bash
cd gnome-extension

# 构建语言包并安装到 ~/.local/share/gnome-shell/extensions/
make install
```

两种方式完成后：

```bash
# 重启 GNOME Shell：按 Alt+F2，输入 r，按回车

# 启用扩展
gnome-extensions enable keystats@0x5c0f.github.io
```

### 4. 验证

```bash
keystatsctl doctor       # 检查设备权限
keystatsctl status       # 检查守护进程统计
# 点击顶部栏 "K... C..." 指示器 → 弹窗应正常显示
```

---

## 分发包构建

所有分发包命令在 **仓库根目录** 下执行。

### tarball

```bash
cd KeyStats.Linux

# 构建 release 二进制
cargo build --release -p keystats-daemon -p keystatsctl

# 组装 tarball
mkdir -p dist
cp target/release/keystats-daemon dist/
cp target/release/keystatsctl dist/
cp packaging/systemd/keystats.service dist/
cp packaging/udev/60-keystats-input.rules dist/
cd dist && tar -czf ../keystats-linux-x86_64.tar.gz *
# tarball 位置：KeyStats.Linux/keystats-linux-x86_64.tar.gz
```

### GNOME 扩展 zip

```bash
cd gnome-extension
zip -r keystats@0x5c0f.github.io.zip \
    metadata.json extension.js prefs.js stylesheet.css schemas/
# zip 位置：gnome-extension/keystats@0x5c0f.github.io.zip
```

将 zip 移动到仓库根目录用于发布上传：

```bash
cp gnome-extension/keystats@0x5c0f.github.io.zip .
```

### .deb

```bash
cargo install cargo-deb
cd KeyStats.Linux && cargo deb -p keystats-daemon
```

### .rpm

```bash
cargo install cargo-rpm
cd KeyStats.Linux && cargo rpm build -p keystats-daemon
```

---

## 权限配置

运行 `keystatsctl doctor` 检查。如有设备被阻止：

```bash
# 方案 A：input 组（推荐）
sudo usermod -aG input $USER
# 重新登录

# 方案 B：专用 keystats 组 + udev 规则
sudo cp KeyStats.Linux/packaging/udev/60-keystats-input.rules /etc/udev/rules.d/
sudo groupadd --system keystats
sudo usermod -aG keystats $USER
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## 故障排除

| 症状 | 检查 |
|------|------|
| 面板显示 "K--" "C--" | 守护进程是否运行？`systemctl --user status keystats.service` |
| `keystatsctl doctor` 显示设备被阻止 | 未加入 `input` 组。参见权限配置 |
| `gnome-extensions list` 中找不到扩展 | 重启 GNOME Shell：**Alt+F2 → r → Enter** |
| 扩展加载但缺少方法 | 守护进程需重新构建。`cargo build -p keystats-daemon` 后重启 |
| Preferences 中 Reset/Clear 失败 | 守护进程未运行或未在 schema 变更后重新构建 |

---

## 卸载

```bash
# 守护进程
systemctl --user disable --now keystats.service
rm ~/.config/systemd/user/keystats.service
rm ~/.local/bin/keystats-daemon ~/.local/bin/keystatsctl

# GNOME 扩展
gnome-extensions uninstall keystats@0x5c0f.github.io

# 数据
rm -rf ~/.local/state/keystats/

# 可选：移除 udev 规则
sudo rm /etc/udev/rules.d/60-keystats-input.rules
```

---

## 文件位置

| 内容 | 位置 |
|------|------|
| 守护进程二进制 | `~/.local/bin/keystats-daemon` |
| CLI 二进制 | `~/.local/bin/keystatsctl` |
| systemd 服务 | `~/.config/systemd/user/keystats.service` |
| GNOME 扩展 | `~/.local/share/gnome-shell/extensions/keystats@0x5c0f.github.io/` |
| 统计数据库 | `~/.local/state/keystats/stats.sqlite3` |
| tarball（构建后） | `KeyStats.Linux/keystats-linux-x86_64.tar.gz` |
| 扩展 zip（构建后） | `gnome-extension/keystats@0x5c0f.github.io.zip` |
| 日志 | `journalctl --user -u keystats.service` |
