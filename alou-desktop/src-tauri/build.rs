fn main() {
    // 在 Windows 平台上，只复制 Windows 需要的 kubo 文件
    #[cfg(target_os = "windows")]
    {
        use std::fs;
        use std::path::PathBuf;
        
        let kubo_source = PathBuf::from("kubo");
        let kubo_target = PathBuf::from("target").join("release").join("kubo");
        
        // 创建目标目录
        if let Err(e) = fs::create_dir_all(&kubo_target) {
            eprintln!("警告: 无法创建 kubo 目标目录: {}", e);
        } else {
            // 只复制 Windows 需要的文件
            let windows_files = vec!["ipfs.exe", "README.md"];
            
            for file in windows_files {
                let source = kubo_source.join(file);
                let target = kubo_target.join(file);
                
                if source.exists() {
                    if let Err(e) = fs::copy(&source, &target) {
                        eprintln!("警告: 无法复制 {}: {}", file, e);
                    }
                }
            }
        }
    }
    
    tauri_build::build()
}
