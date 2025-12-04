//! 跨平台存储模块
//!
//! 提供统一的存储接口:
//! - 桌面版: 使用文件系统 (std::fs)
//! - WASM版: 使用 LocalStorage (quad-storage)

/// 获取存储目录路径（桌面版）
#[cfg(not(target_arch = "wasm32"))]
fn get_storage_path(key: &str) -> Result<std::path::PathBuf, String> {
    // 优先使用当前目录，如果失败则使用临时目录
    let base_dir = std::env::current_dir()
        .or_else(|_| std::env::temp_dir().canonicalize())
        .map_err(|e| format!("无法获取存储目录: {}", e))?;

    let mut path = base_dir;
    path.push(format!("{}.json", key));
    Ok(path)
}

/// 保存数据到存储
pub fn save(key: &str, data: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let path = get_storage_path(key)?;
        fs::write(&path, data).map_err(|e| format!("写入文件失败 '{}': {}", path.display(), e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let storage = quad_storage::STORAGE
            .lock()
            .map_err(|e| format!("获取存储锁失败: {}", e))?;
        storage.set(key, data);
        Ok(())
    }
}

/// 从存储加载数据
pub fn load(key: &str) -> Result<String, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let path = get_storage_path(key)?;
        fs::read_to_string(&path).map_err(|e| format!("读取文件失败 '{}': {}", path.display(), e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let storage = quad_storage::STORAGE
            .lock()
            .map_err(|e| format!("获取存储锁失败: {}", e))?;
        storage
            .get(key)
            .ok_or_else(|| format!("存储中未找到键 '{}'", key))
    }
}

/// 从存储删除数据
#[allow(dead_code)]
pub fn remove(key: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let path = get_storage_path(key)?;
        fs::remove_file(&path).map_err(|e| format!("删除文件失败 '{}': {}", path.display(), e))
    }

    #[cfg(target_arch = "wasm32")]
    {
        let storage = quad_storage::STORAGE
            .lock()
            .map_err(|e| format!("获取存储锁失败: {}", e))?;
        storage.remove(key);
        Ok(())
    }
}
