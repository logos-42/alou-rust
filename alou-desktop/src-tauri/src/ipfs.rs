// IPFS/Kubo integration module
// This module provides Rust-side utilities for interacting with the embedded Kubo binary

use std::path::PathBuf;
use std::process::Command;

pub struct KuboManager {
    binary_path: PathBuf,
    data_dir: PathBuf,
}

impl KuboManager {
    pub fn new(binary_path: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            binary_path,
            data_dir,
        }
    }

    pub fn init(&self) -> Result<(), String> {
        let output = Command::new(&self.binary_path)
            .arg("init")
            .arg("--profile=server")
            .env("IPFS_PATH", &self.data_dir)
            .output()
            .map_err(|e| format!("Failed to initialize IPFS: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("already") {
                return Err(format!("IPFS init failed: {}", stderr));
            }
        }

        Ok(())
    }

    pub fn get_peer_id(&self) -> Result<String, String> {
        let output = Command::new(&self.binary_path)
            .arg("id")
            .arg("--format=<id>")
            .env("IPFS_PATH", &self.data_dir)
            .output()
            .map_err(|e| format!("Failed to get peer ID: {}", e))?;

        if !output.status.success() {
            return Err("IPFS node is not running".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn add_file(&self, file_path: &str) -> Result<String, String> {
        let output = Command::new(&self.binary_path)
            .arg("add")
            .arg("-Q")
            .arg(file_path)
            .env("IPFS_PATH", &self.data_dir)
            .output()
            .map_err(|e| format!("Failed to add file: {}", e))?;

        if !output.status.success() {
            return Err("Failed to add file to IPFS".to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn pin_cid(&self, cid: &str) -> Result<(), String> {
        let output = Command::new(&self.binary_path)
            .arg("pin")
            .arg("add")
            .arg(cid)
            .env("IPFS_PATH", &self.data_dir)
            .output()
            .map_err(|e| format!("Failed to pin CID: {}", e))?;

        if !output.status.success() {
            return Err("Failed to pin CID".to_string());
        }

        Ok(())
    }
}

