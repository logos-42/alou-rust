#!/bin/bash
# 下载并设置 Kubo (IPFS) 二进制文件

set -e

KUBO_VERSION="v0.39.0"
KUBO_DIR="src-tauri/kubo"
OS=""
ARCH="amd64"

# 检测操作系统
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    KUBO_FILE="kubo_${KUBO_VERSION}_linux-${ARCH}.tar.gz"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="darwin"
    KUBO_FILE="kubo_${KUBO_VERSION}_darwin-${ARCH}.tar.gz"
else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi

# 创建目录
mkdir -p "$KUBO_DIR"

# 下载 Kubo
echo "Downloading Kubo ${KUBO_VERSION} for ${OS}..."
KUBO_URL="https://dist.ipfs.tech/kubo/${KUBO_VERSION}/${KUBO_FILE}"
curl -L "$KUBO_URL" -o "/tmp/${KUBO_FILE}"

# 解压
echo "Extracting Kubo..."
tar -xzf "/tmp/${KUBO_FILE}" -C "/tmp"

# 复制二进制文件
echo "Copying binary to ${KUBO_DIR}..."
cp "/tmp/kubo/ipfs" "${KUBO_DIR}/ipfs"

# 设置执行权限
chmod +x "${KUBO_DIR}/ipfs"

# 清理
rm -rf "/tmp/kubo" "/tmp/${KUBO_FILE}"

echo "Kubo setup complete!"
echo "Binary location: ${KUBO_DIR}/ipfs"

