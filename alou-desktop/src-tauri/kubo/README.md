# Kubo (IPFS) 二进制文件

此目录应包含对应平台的 Kubo (IPFS) 二进制文件。

## 下载 Kubo

### Windows
```bash
# 下载 Windows 版本
# https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_windows-amd64.zip
# 解压后将 ipfs.exe 放到此目录
```

### macOS
```bash
# 下载 macOS 版本
# https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_darwin-amd64.tar.gz
# 解压后将 ipfs 二进制放到此目录
```

### Linux
```bash
# 下载 Linux 版本
# https://dist.ipfs.tech/kubo/v0.24.0/kubo_v0.24.0_linux-amd64.tar.gz
# 解压后将 ipfs 二进制放到此目录
```

## 目录结构

```
kubo/
├── ipfs.exe (Windows)
├── ipfs (macOS/Linux)
└── README.md
```

## 注意事项

1. 确保二进制文件有执行权限（macOS/Linux）
2. 二进制文件会被打包到应用的 resources 目录
3. 首次运行时，IPFS 会在应用数据目录初始化

