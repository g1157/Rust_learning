//! 跨平台存储模块
//!
//! 提供统一的存储接口:
//! - 桌面版: 使用文件系统 (std::fs)
//! - WASM版: 暂时使用内存模式（不持久化）
//!   TODO: 使用 Macroquad 的 file API 或 quad-storage crate

/// 保存数据到存储
pub fn save(key: &str, data: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", key));
        fs::write(path, data).map_err(|e| format!("Failed to write file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: 暂不持久化，仅在内存中
        // 可以用 Macroquad 的文件 API 或添加 quad-storage crate
        eprintln!("WASM: save() called but not persisted (in-memory only)");
        Ok(())
    }
}

/// 从存储加载数据
pub fn load(key: &str) -> Result<String, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", key));
        fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        // WASM: 无法加载，返回错误（使用默认值）
        Err("WASM: No persistent storage available".to_string())
    }
}

/// 从存储删除数据
#[allow(dead_code)]
pub fn remove(key: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", key));
        fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        eprintln!("WASM: remove() called but not implemented");
        Ok(())
    }
}
