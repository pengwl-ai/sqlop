use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

/// GaussDB数据库适配器
pub struct GaussDBAdapter {
    common: CommonAdapter,
}

impl GaussDBAdapter {
    /// 创建新的GaussDB适配器实例
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::GaussDB),
        }
    }
}

impl super::DatabaseAdapter for GaussDBAdapter {
    /// 检查是否可以处理指定的数据库类型
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::GaussDB)
    }

    /// 解析审计日志
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    /// 获取数据库类型
    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::GaussDB
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

impl GaussDBAdapter {
    /// 解析GaussDB特定的SQL语法
    pub fn parse_gaussdb_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_gaussdb_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    /// 处理GaussDB特有的SQL语法
    fn process_gaussdb_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理GaussDB特有的语法元素
        // 例如：GaussDB的一些特定函数或扩展
        
        // 1. 处理GaussDB特有的CREATE TABLE语法
        processed = self.handle_gaussdb_create_table(&processed);
        
        // 2. 处理GaussDB特有的函数
        processed = self.handle_gaussdb_functions(&processed);
        
        processed
    }

    /// 处理GaussDB特有的CREATE TABLE语法
    fn handle_gaussdb_create_table(&self, sql: &str) -> String {
        // GaussDB在PostgreSQL基础上增加了一些存储参数
        let processed = sql.replace(
            "USING HEAP", 
            "WITH (orientation=row, compression=no)"
        );
        processed.replace(
            "USING COLUMN", 
            "WITH (orientation=column, compression=low)"
        )
    }

    /// 处理GaussDB特有的函数
    fn handle_gaussdb_functions(&self, sql: &str) -> String {
        // 替换GaussDB特有的函数为标准函数
        let processed = sql.replace("GAUSSDBNOW()", "NOW()");
        processed.replace("GAUSSDBCURRENT_TIMESTAMP()", "CURRENT_TIMESTAMP")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OperationType;

    #[test]
    fn test_gaussdb_adapter_can_handle() {
        let adapter = GaussDBAdapter::new();
        assert!(adapter.can_handle(&DatabaseType::GaussDB));
        assert!(!adapter.can_handle(&DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_gaussdb_adapter_get_database_type() {
        let adapter = GaussDBAdapter::new();
        assert_eq!(adapter.get_database_type(), DatabaseType::GaussDB);
    }

    #[test]
    fn test_gaussdb_adapter_parse_simple_sql() {
        let adapter = GaussDBAdapter::new();
        let sql = "SELECT id, name FROM users WHERE status = 'active'";
        
        let result = adapter.extract_metadata(sql).expect("解析失败");
        assert_eq!(result.database_type, DatabaseType::GaussDB);
        assert_eq!(result.operation_type, OperationType::SELECT);
        assert!(result.tables.contains("users"));
        assert!(result.columns.contains("id"));
        assert!(result.columns.contains("name"));
    }

    #[test]
    fn test_gaussdb_adapter_process_specific_syntax() {
        let adapter = GaussDBAdapter::new();
        let sql = "CREATE TABLE test_table (id INT) USING HEAP";
        
        let processed = adapter.process_gaussdb_syntax(sql);
        assert!(processed.contains("WITH (orientation=row, compression=no)"));
    }
}