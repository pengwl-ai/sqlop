use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

/// Highgo数据库适配器
pub struct HighgoAdapter {
    common: CommonAdapter,
}

impl HighgoAdapter {
    /// 创建新的Highgo适配器实例
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::Highgo),
        }
    }
}

impl super::DatabaseAdapter for HighgoAdapter {
    /// 检查是否可以处理指定的数据库类型
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::Highgo)
    }

    /// 解析审计日志
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    /// 获取数据库类型
    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::Highgo
    }

    /// 标准化SQL语句
    fn normalize_sql(&self, sql: &str) -> String {
        self.common.normalize_sql(sql)
    }

    /// 从SQL语句中提取元数据
    fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        self.common.extract_metadata(sql)
    }
}

impl HighgoAdapter {
    /// 解析Highgo特定的SQL语法
    pub fn parse_highgo_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_highgo_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    /// 处理Highgo特有的SQL语法
    fn process_highgo_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理Highgo特有的语法元素
        // 1. 处理Highgo特有的数据类型
        processed = self.handle_highgo_data_types(&processed);
        
        // 2. 处理Highgo特有的函数
        processed = self.handle_highgo_functions(&processed);
        
        // 3. 处理Highgo特有的扩展
        processed = self.handle_highgo_extensions(&processed);
        
        processed
    }

    /// 处理Highgo特有的数据类型
    fn handle_highgo_data_types(&self, sql: &str) -> String {
        // Highgo支持一些特定的数据类型
        let processed = sql.replace("HG_UUID", "UUID");
        processed.replace("HG_JSONB", "JSONB")
    }

    /// 处理Highgo特有的函数
    fn handle_highgo_functions(&self, sql: &str) -> String {
        // 替换Highgo特有的函数为标准函数
        let processed = sql.replace("HG_ENCRYPT()", "PGP_SYM_ENCRYPT()");
        processed.replace("HG_DECRYPT()", "PGP_SYM_DECRYPT()")
    }

    /// 处理Highgo特有的扩展
    fn handle_highgo_extensions(&self, sql: &str) -> String {
        // Highgo有一些特有的扩展
        // 这里可以添加对这些扩展的特殊处理
        sql.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OperationType;

    #[test]
    fn test_highgo_adapter_can_handle() {
        let adapter = HighgoAdapter::new();
        assert!(adapter.can_handle(&DatabaseType::Highgo));
        assert!(!adapter.can_handle(&DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_highgo_adapter_get_database_type() {
        let adapter = HighgoAdapter::new();
        assert_eq!(adapter.get_database_type(), DatabaseType::Highgo);
    }

    #[test]
    fn test_highgo_adapter_parse_simple_sql() {
        let adapter = HighgoAdapter::new();
        let sql = "SELECT id, name FROM users WHERE status = 'active'";
        
        let result = adapter.extract_metadata(sql).expect("解析失败");
        assert_eq!(result.database_type, DatabaseType::Highgo);
        assert_eq!(result.operation_type, OperationType::SELECT);
        assert!(result.tables_contains("users"));
        assert!(result.columns_contains("id"));
        assert!(result.columns_contains("name"));
    }

    #[test]
    fn test_highgo_adapter_handle_data_types() {
        let adapter = HighgoAdapter::new();
        let sql = "CREATE TABLE test (id HG_UUID, data HG_JSONB)";
        
        let processed = adapter.handle_highgo_data_types(sql);
        assert!(processed.contains("UUID"));
        assert!(processed.contains("JSONB"));
    }
}