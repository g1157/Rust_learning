//! 跨平台存储模块
//!
//! 提供统一的存储接口:
//! - 桌面版: 使用文件系统 (std::fs)
//! - WASM版: 使用 LocalStorage (quad-storage)

/// 保存数据到存储
pub fn save(_key: &str, _data: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", _key));
        fs::write(path, _data).map_err(|e| format!("Failed to write file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        quad_storage::STORAGE
            .lock()
            .unwrap()
            .set(_key, _data);
        Ok(())
    }
}

/// 从存储加载数据
pub fn load(_key: &str) -> Result<String, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", _key));
        fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        quad_storage::STORAGE
            .lock()
            .unwrap()
            .get(_key)
            .ok_or_else(|| format!("Key '{}' not found in storage", _key))
    }
}

/// 从存储删除数据
#[allow(dead_code)]
pub fn remove(_key: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let mut path = std::env::current_dir().unwrap_or_default();
        path.push(format!("{}.json", _key));
        fs::remove_file(path).map_err(|e| format!("Failed to delete file: {}", e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        quad_storage::STORAGE
            .lock()
            .unwrap()
            .remove(_key);
        Ok(())
    }
}
