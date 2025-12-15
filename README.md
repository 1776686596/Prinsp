# PrinSp - Linux 截图工具

在 Windows 上一直使用 PixPin 截图，它的截图和 OCR 功能非常好用。但切换到 Linux 后发现没有类似的好用工具，于是自己动手做了这个尝试。目前测试下来基本功能都能正常使用。

## 功能特性

- 🖼️ **区域截图** - 框选任意屏幕区域
- ✏️ **标注工具** - 矩形、箭头、文字、马赛克
- 🎨 **颜色选择** - 7种预设颜色
- ↩️ **撤销/重做** - 支持操作历史
- 📋 **剪贴板** - 一键复制到剪贴板
- 📝 **OCR** - 调用 Tesseract 进行文字识别（需本机安装）
- 🔔 **系统托盘** - 后台运行，随时调用

## 使用方法

### 启动应用

```bash
# 开发模式
npm run tauri dev

# 构建发布版本
npm run tauri build
```

### 截图流程

1. **启动截图**
   - 点击系统托盘图标开始截图
   - 托盘菜单选择「截图」

2. **选择区域**
   - 屏幕会显示全屏截图
   - 按住鼠标左键拖动选择区域
   - 松开鼠标确认选区
   - 按 `Esc` 取消截图

3. **添加标注**
   - 选择工具栏中的标注工具：
     - `▢` 矩形框
     - `→` 箭头
     - `T` 文字
     - `▦` 马赛克
   - 选择颜色
   - 在截图上绘制标注
   - 使用 `↩` 撤销，`↪` 重做

4. **完成截图**
   - 点击 `✓` 确认，图片自动复制到剪贴板
   - 点击 `✕` 取消截图

### 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Esc` | 取消截图 |
| `Ctrl+Z` | 撤销标注 |
| `Ctrl+Y` | 重做标注 |

## 系统要求

- Linux (X11/Wayland)
- 需要安装以下依赖：
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libpipewire-0.3-dev libgbm-dev librsvg2-dev
  # OCR 功能（可选）
  sudo apt install tesseract-ocr tesseract-ocr-chi-sim
  ```

## 技术栈

- **后端**: Rust + Tauri v2
- **前端**: Vue 3 + TypeScript
- **截图**: xcap
- **剪贴板**: arboard

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## License

MIT
