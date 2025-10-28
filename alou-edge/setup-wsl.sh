#!/bin/bash
set -e

echo "=== Setting up WSL environment for Rust Cloudflare Workers ==="

# Update package list
echo "Updating package list..."
sudo apt-get update

# Install build essentials
echo "Installing build essentials..."
sudo apt-get install -y build-essential pkg-config libssl-dev

# Install Rust
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "Rust already installed: $(rustc --version)"
fi

# Add wasm32 target
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Install worker-build
echo "Installing worker-build..."
cargo install worker-build --force

# Install Node.js and npm if not present
if ! command -v node &> /dev/null; then
    echo "Installing Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
else
    echo "Node.js already installed: $(node --version)"
fi

# Install wrangler
if ! command -v wrangler &> /dev/null; then
    echo "Installing wrangler..."
    npm install -g wrangler
else
    echo "Wrangler already installed: $(wrangler --version)"
fi

echo ""
echo "=== Setup complete! ==="
echo ""
echo "To build the project, run:"
echo "  cd /mnt/d/AI/alou-pay/aloupay/alou-edge"
echo "  worker-build --release"
echo ""
echo "To deploy, run:"
echo "  wrangler deploy"
