# 🔧 DMG 打包问题诊断与解决

## 📋 问题描述

在执行 `npm run build:tauri:macos` 打包时，DMG 生成脚本出错。

---

## 🔍 当前状态

**构建阶段**:
- ✅ 前端构建 (Vite)
- 🔄 Rust 编译 (进行中)
- ⏳ DMG 生成 (等待中)

**已生成文件**:
```
/Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/dmg/
├── bundle_dmg.sh (19 KB) - DMG 打包脚本
└── icon.icns (558 KB) - 图标文件
```

**缺失文件**:
```
❌ Alou_0.1.11_x64.dmg - 最终的 DMG 安装包
```

---

## 🐛 可能的问题原因

### 1. Rust 编译时间过长
**现象**: 构建进程长时间运行  
**原因**: 
- 首次打包或代码变动大
- 依赖项需要重新编译
- 系统性能限制

**解决方案**: 等待编译完成（正常需要 5-15 分钟）

---

### 2. DMG 打包脚本错误
**常见错误**:
```bash
# 错误 1: 找不到支持文件
Cannot find support/ directory

# 错误 2: AppleScript 执行失败
Failed running AppleScript

# 错误 3: 磁盘空间不足
No space left on device

# 错误 4: 权限问题
Permission denied
```

**解决方案**:

#### 方案 1: 清理后重新打包
```bash
cd /Users/apple/Downloads/alou/alou-desktop

# 清理旧的构建产物
rm -rf src-tauri/target/release/bundle
rm -rf dist

# 重新打包
npm run build:tauri:macos
```

#### 方案 2: 跳过 DMG，只生成 APP
```bash
cd /Users/apple/Downloads/alou/alou-desktop
npm run tauri build -- --bundles app
```

#### 方案 3: 手动修复 DMG 脚本
```bash
# 编辑 tauri.conf.json
# 修改 bundle 配置
```

---

### 3. macOS 签名问题
**现象**: DMG 创建成功但无法打开  
**错误信息**: "无法验证开发者"

**解决方案**:
```bash
# 方法 1: 使用开发证书签名
codesign --sign - --force --deep \
  /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/Alou.app

# 方法 2: 移除隔离属性
xattr -rd com.apple.quarantine \
  /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/Alou.app

# 方法 3: 系统设置
# 系统偏好设置 → 安全性与隐私 → 仍要打开
```

---

### 4. 资源文件缺失
**现象**: 打包时找不到图标或背景图片  
**错误信息**: "Cannot find icon file"

**解决方案**:
```bash
# 检查图标文件
ls -la /Users/apple/Downloads/alou/alou-desktop/src-tauri/icons/

# 重新生成图标
cd /Users/apple/Downloads/alou/alou-desktop
npm run icons:regenerate
```

---

## 🛠️ 推荐的打包流程

### 完整打包（推荐）
```bash
cd /Users/apple/Downloads/alou/alou-desktop

# 1. 清理旧构建
npm run clean 2>/dev/null || rm -rf dist src-tauri/target/release/bundle

# 2. 安装依赖
npm install

# 3. 设置 Kubo
npm run setup:kubo:unix

# 4. 打包
npm run build:tauri:macos
```

### 快速打包（跳过 Kubo）
```bash
cd /Users/apple/Downloads/alou/alou-desktop

# 直接打包（假设 Kubo 已存在）
npm run build
npm run tauri build -- --bundles dmg
```

### 仅生成 APP（开发测试）
```bash
cd /Users/apple/Downloads/alou/alou-desktop
npm run tauri build -- --bundles app
```

---

## 📊 构建时间参考

| 阶段 | 首次构建 | 后续构建 |
|------|----------|----------|
| 前端构建 | 1-2 分钟 | 30 秒 |
| Rust 编译 | 5-10 分钟 | 1-3 分钟 |
| DMG 生成 | 1-2 分钟 | 1 分钟 |
| **总计** | **8-14 分钟** | **3-5 分钟** |

---

## 🔍 实时监控构建进度

### 查看进程
```bash
# 查看 Rust 编译进程
ps aux | grep "rustc.*alou_desktop"

# 查看构建日志
tail -f /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/build.log
```

### 检查构建产物
```bash
# 检查 APP 是否生成
ls -lh /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/

# 检查 DMG 是否生成
ls -lh /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/dmg/
```

---

## 🎯 验证构建产物

### 验证 APP
```bash
# 检查 APP 结构
ls -la /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/Alou.app/Contents/

# 检查可执行文件
file /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/Alou.app/Contents/MacOS/Alou

# 尝试运行
open /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/macos/Alou.app
```

### 验证 DMG
```bash
# 验证 DMG 完整性
hdiutil verify /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/dmg/Alou_0.1.11_x64.dmg

# 挂载 DMG
hdiutil attach /Users/apple/Downloads/alou/alou-desktop/src-tauri/target/release/bundle/dmg/Alou_0.1.11_x64.dmg
```

---

## 📝 常见错误及解决方案

### 错误 1: `command not found: cargo`
**解决**:
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 错误 2: `npm: command not found`
**解决**:
```bash
# 安装 Node.js
brew install node
```

### 错误 3: `Tauri CLI not found`
**解决**:
```bash
cd /Users/apple/Downloads/alou/alou-desktop
npm install -g @tauri-apps/cli
```

### 错误 4: `Signing failed`
**解决**:
```bash
# 使用开发证书
export APPLE_SIGNING_IDENTITY="-"
npm run build:tauri:macos
```

### 错误 5: `Disk image not found`
**解决**:
```bash
# 检查构建产物
ls -la src-tauri/target/release/bundle/

# 如果只有 APP 没有 DMG，手动创建
cd src-tauri/target/release/bundle
npx create-dmg \
  --volname "Alou" \
  --volicon "icons/icon.icns" \
  --background "background.png" \
  --window-pos 200 120 \
  --window-size 600 400 \
  --icon-size 100 \
  --app-drop-link 450 200 \
  "Alou_0.1.11_x64.dmg" \
  "macos/Alou.app"
```

---

## 🎉 成功标志

打包成功的输出应该包含：
```
✅ Compressing disk image...
✅ Disk image done
✅ Done
```

生成的文件：
```
src-tauri/target/release/bundle/dmg/Alou_0.1.11_x64.dmg (约 89 MB)
src-tauri/target/release/bundle/macos/Alou.app (约 220 MB)
```

---

## 📞 需要帮助？

如果以上方案都无法解决问题，请提供：
1. 完整的错误日志
2. 系统版本 (`sw_vers`)
3. Node.js 版本 (`node -v`)
4. Rust 版本 (`rustc -V`)
5. 构建产物列表 (`ls -la src-tauri/target/release/bundle/`)

---

**最后更新**: 2026 年 3 月 6 日  
**版本**: 0.1.11  
**平台**: macOS
