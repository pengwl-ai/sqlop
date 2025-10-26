pub mod core;
pub mod adapters;
pub mod utils;

pub use core::*;
pub use adapters::*;

// 重新导出主要类型和函数
pub use core::SqlopEngine;
pub use core::types::{
    AuditLog, DatabaseConfig, DatabaseType, OperationType, ParseResult, ParserConfig,
    SensitivePattern, SqlObject,
};
pub use adapters::AdapterManager;
pub use adapters::DatabaseAdapter;

// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 初始化日志系统
pub fn init_logger() {
    env_logger::init();
}

/// 创建默认引擎实例
pub fn create_engine() -> SqlopEngine {
    SqlopEngine::default()
}

/// 从配置文件创建引擎实例
pub fn create_engine_from_config(config_path: &std::path::Path) -> std::result::Result<SqlopEngine, Box<dyn std::error::Error>> {
    Ok(SqlopEngine::from_config_file(config_path)?)
}

/// 获取支持的数据库类型列表
pub fn supported_databases() -> Vec<DatabaseType> {
    let manager = AdapterManager::new();
    manager.list_supported_databases()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_engine() {
        let engine = create_engine();
        assert!(engine.get_performance_stats().cache_size == 0);
    }

    #[test]
    fn test_supported_databases() {
        let databases = supported_databases();
        assert!(!databases.is_empty());
        assert!(databases.contains(&DatabaseType::MySQL));
        assert!(databases.contains(&DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "sqlop");
    }
}