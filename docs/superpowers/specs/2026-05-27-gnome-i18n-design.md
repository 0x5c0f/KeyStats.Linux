# GNOME Shell Extension Localization (i18n)

## Goal

为 GNOME Shell 扩展添加 gettext 国际化支持，初始支持英文（源语言）和简体中文。

## Design Principles

1. **GNOME 原生嵌合**：使用 GNOME Shell 扩展标准的 gettext 工具链和目录结构。
2. **与 macOS 低耦合**：翻译文件独立维护，不复用 macOS 侧的 `.strings` 文件。

## Current State

- `metadata.json` 已声明 `"gettext-domain": "keystats"`
- `extension.js` 和 `prefs.js` 已导入 `gettext as _` 并包裹了大部分字符串
- 缺少：`.pot` 模板、`.po` 翻译文件、`.mo` 编译产物、构建脚本
- 有 4-5 个硬编码字符串未包裹 `_()`

## Scope

### In Scope

1. **修复硬编码字符串**：将 `prefs.js` 中的 `'Error'`、`'Could not reach...'` 和 `extension.js` 中的 `'offline'`、`'K'`、`'C'`、`'/s'` 包裹 `_()`
2. **创建 `po/` 目录**：包含 `keystats.pot` 模板和 `zh_CN.po` 中文翻译
3. **创建 Makefile**：标准 gettext 构建流程（`xgettext` 提取 → `msgfmt` 编译）
4. **输出 `locale/` 目录**：`locale/zh_CN/LC_MESSAGES/keystats.mo`
5. **安装脚本**：Makefile 的 `install` target 将 `.mo` 文件安装到扩展目录

### Out of Scope

- meson 构建系统（对扩展过重）
- 复用或同步 macOS 侧的翻译文件
- 其他语言（后续可增量添加）

## Architecture

```
KeyStats.GNOME/keystats@debugtheworldbot.github.io/
├── po/
│   ├── keystats.pot          # 模板（由 xgettext 从 JS 源文件提取）
│   └── zh_CN.po              # 简体中文翻译
├── locale/
│   └── zh_CN/
│       └── LC_MESSAGES/
│           └── keystats.mo   # 编译产物（由 msgfmt 生成）
├── Makefile                  # i18n 构建 + 安装
├── extension.js              # 源语言（英文）已包裹 _()
├── prefs.js                  # 源语言（英文）已包裹 _()
└── ...
```

### Gettext 工作流

1. `make pot` — 从 `extension.js` + `prefs.js` 提取所有 `_()` 调用，生成 `po/keystats.pot`
2. `make po` — 更新 `po/zh_CN.po`（合并新字符串，保留已有翻译）
3. `make mo` — 编译 `zh_CN.po` → `locale/zh_CN/LC_MESSAGES/keystats.mo`
4. `make install` — 将 `locale/` 复制到 `~/.local/share/gnome-shell/extensions/keystats@debugtheworldbot.github.io/locale/`
5. `make` — 执行 pot + po + mo 全流程

### 语言回退策略

- **默认跟随系统语言**：GNOME Shell 加载扩展时自动读取系统 locale（`LANG` / `LANGUAGE`），gettext 根据该 locale 查找对应的 `.mo` 文件。
- **不支持时回退英文**：若系统语言不在已有的语言包中（如日语、法语），gettext 找不到对应 `.mo`，自动使用源语言（英文）显示。无需代码干预。
- **无语言切换 UI**：不提供扩展内的语言选择器，完全由系统控制。

### GNOME Shell 45+ gettext 自动绑定

GNOME Shell 45+ 的 `Extension` 类在加载时自动根据 `metadata.json` 的 `gettext-domain` 调用 `bindtextdomain`。无需在代码中手动绑定。

## Hardcoded Strings to Fix

| File | Line | String | Fix |
|------|------|--------|-----|
| prefs.js | ~26 | `'Error'` | `_('Error')` |
| prefs.js | ~27 | `'Could not reach the KeyStats daemon...'` | `_('Could not reach...')` |
| extension.js | ~248 | `'offline'` | `_('offline')` |
| extension.js | ~218-220 | `'K'`, `'C'`, `'/s'` | `_('K')`, `_('C')`, `_('/s')` |

注：`'K'`、`'C'`、`'/s'` 是单位后缀，在中文环境下通常不需要翻译（保持原样即可），但包裹 `_()` 保持一致性。

## Verification

1. `make pot` 生成 `.pot` 文件，检查所有字符串是否被提取
2. 翻译 `.po` 文件后 `make mo` 编译成功
3. 安装扩展后切换系统语言到中文，验证 UI 显示中文
4. 切换回英文，验证 UI 回退到英文（源语言）
