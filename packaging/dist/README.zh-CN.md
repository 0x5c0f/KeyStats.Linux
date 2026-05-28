# KeyStats.Linux

隐私优先的 Linux 键盘鼠标统计工具，集成 GNOME Shell 面板。

## 前置条件

- GNOME Shell 45+
- 用户需要加入 `input` 组才能读取输入设备：

```bash
sudo usermod -aG input $USER
# 需要重新登录才能生效
```

## 安装

```bash
make install
```

安装内容：
- `keystats-daemon` 和 `keystatsctl` → `~/.local/bin/`
- systemd 用户服务 → `~/.config/systemd/user/`

## 启动守护进程

```bash
make enable
```

## 安装 GNOME 扩展

```bash
gnome-extensions install keystats@0x5c0f.github.io.zip
```

重新加载 GNOME Shell（X11 下 `Alt+F2` → `r`，Wayland 下需重新登录），然后：

```bash
gnome-extensions enable keystats@0x5c0f.github.io
```

## 验证

```bash
keystatsctl status
```

## 卸载

```bash
make uninstall
gnome-extensions uninstall keystats@0x5c0f.github.io
```
