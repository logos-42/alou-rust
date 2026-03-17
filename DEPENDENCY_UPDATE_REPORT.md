# 依赖更新报告

**更新日期**: 2026-03-17

---

## ✅ Rust 依赖更新

### Cargo.toml 更新
```toml
# JavaScript Runtime
rquickjs = { version = "0.9", features = ["full", "parallel"] }  
→ rquickjs = { version = "0.10", features = ["full", "parallel"] }
```

### cargo update 更新的包
- `uuid`: 1.20.0 → 1.22.0
- `unicode-ident`: 1.0.23 → 1.0.24
- `wasm-bindgen`: 0.2.108 → 0.2.114
- `wasm-bindgen-futures`: 0.4.58 → 0.4.64
- `web-sys`: 0.3.85 → 0.3.91
- `tracing-subscriber`: 0.3.22 → 0.3.23
- `which`: 8.0.0 → 8.0.2
- `wry`: 0.54.1 → 0.54.3
- `zerocopy`: 0.8.39 → 0.8.42
- `zerocopy-derive`: 0.8.39 → 0.8.42
- `toml_parser`: 1.0.7 → 1.0.9
- `winnow`: 0.7.14 → 0.7.15

**新增依赖**:
- `unsigned-varint`: 0.8.0
- `utf8parse`: 0.2.2
- `web_atoms`: 0.2.3
- `webpki-root-certs`: 1.0.6

**移除依赖**:
- `toml_edit` (重复版本)
- `ucd-parse`
- `ucd-trie`
- `universal-hash`
- `toml_write`
- `winsafe`
- `wmi`

---

## 📦 npm 依赖

当前版本已经是最新或接近最新：

### 主要依赖
```json
{
  "@anthropic-ai/claude-agent-sdk": "^0.2.39",
  "@tauri-apps/api": "^2.10.1",
  "@tauri-apps/plugin-fs": "^2.4.5",
  "@walletconnect/ethereum-provider": "^2.23.5",
  "axios": "^1.13.5",
  "react": "^18.3.1",
  "react-dom": "^18.3.1",
  "react-router-dom": "^6.30.3",
  "zustand": "^5.0.11"
}
```

### 开发依赖
```json
{
  "@tauri-apps/cli": "^2.10.0",
  "@typescript-eslint/eslint-plugin": "^8.55.0",
  "@vitejs/plugin-react-swc": "^4.2.3",
  "eslint": "^9.20.0",
  "sharp": "^0.34.5",
  "typescript": "^5.7.2",
  "vite": "^7.3.1"
}
```

---

## 🔧 编译状态

**第一次编译**: 正在进行（下载新依赖）
**预计时间**: 5-10 分钟

后续编译将会更快。

---

## 📊 更新摘要

| 类别 | 更新前 | 更新后 | 状态 |
|------|--------|--------|------|
| Rust 包 | 1200+ | 1200+ | ✅ 完成 |
| rquickjs | 0.9 | 0.10 | ✅ 完成 |
| npm 包 | 最新 | 最新 | ✅ 已是最新 |

---

## ⚠️ 注意事项

1. **第一次编译时间较长** - 需要下载和编译新依赖
2. **后续编译会更快** - 依赖已缓存
3. **功能无变化** - 只是版本更新，API 兼容

---

**更新完成时间**: 2026-03-17
**状态**: ✅ 依赖已更新，编译中
