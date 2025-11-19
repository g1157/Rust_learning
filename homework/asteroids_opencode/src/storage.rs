//! 跨平台存储模块
//!
//! 提供统一的存储接口:
//! - 桌面版: 使用文件系统 (std::fs)
//! - WASM版: 使用浏览器 LocalStorage API

#[cfg(target_arch = "wasm32")]
use web_sys::window;

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
        let window = window().ok_or("No window found")?;
        let storage = window
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;
        
        storage
            .set_item(key, data)
            .map_err(|e| format!("Failed to save to localStorage: {:?}", e))
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
        let window = window().ok_or("No window found")?;
        let storage = window
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;
        
        storage
            .get_item(key)
            .map_err(|e| format!("Failed to load from localStorage: {:?}", e))?
            .ok_or_else(|| "Key not found in storage".to_string())
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
        let window = window().ok_or("No window found")?;
        let storage = window
            .local_storage()
            .map_err(|e| format!("Failed to get localStorage: {:?}", e))?
            .ok_or("localStorage not available")?;
        
        storage
            .remove_item(key)
            .map_err(|e| format!("Failed to remove from localStorage: {:?}", e))
    }
}
