# Key Breakdown Pill Badge — 微边框视觉增强

## 目标

为 Key Breakdown 区域的 pill badge（按键名标签）添加微边框，使其与 popup 中其他卡片组件（Hero cards、Click tiles、Distance cards）的视觉语言保持一致。

## 当前状态

`ks-key-badge` 使用半透明背景色，无边框：
- Dark: `background-color: rgba(255, 255, 255, 0.1)`
- Light: `background-color: rgba(0, 0, 0, 0.05)`

其他卡片组件已有边框：
- Dark: `border: 1px solid rgba(255, 255, 255, 0.06)`
- Light: `border: 1px solid rgba(0, 0, 0, 0.08)`

## 方案

直接复用 Hero cards 的边框参数，为 `.ks-key-badge` 的 dark/light 主题规则各添加一行 `border` 属性。

## 改动

文件：`gnome-extension/stylesheet.css`

```css
/* Dark theme — line 146 附近 */
.ks-dark .ks-key-badge {
    background-color: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    border: 1px solid rgba(255, 255, 255, 0.06);  /* 新增 */
}

/* Light theme — line 218 附近 */
.ks-light .ks-key-badge {
    background-color: rgba(0, 0, 0, 0.05);
    color: #1a1a1a;
    border: 1px solid rgba(0, 0, 0, 0.08);  /* 新增 */
}
```

## 影响范围

- 仅影响 Key Breakdown 区域的最多 15 个 pill badge
- 不改变布局、间距或尺寸（border-box 模型下 padding 不变）
- 与 Hero cards、Click tiles、Distance cards 的边框风格完全一致
