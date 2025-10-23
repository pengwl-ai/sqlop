use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

/// Dameng数据库适配器
pub struct DamengAdapter {
    common: CommonAdapter,
}

impl DamengAdapter {
    /// 创建新的Dameng适配器实例
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::Dameng),
        }
    }
}

impl super::DatabaseAdapter for DamengAdapter {
    /// 检查是否可以处理指定的数据库类型
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::Dameng)
    }

    /// 解析审计日志
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    /// 获取数据库类型
    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::Dameng
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

impl DamengAdapter {
    /// 解析Dameng特定的SQL语法
    pub fn parse_dameng_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_dameng_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    /// 处理Dameng特有的SQL语法
    fn process_dameng_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理Dameng特有的语法元素
        // 1. 处理Dameng特有的表空间语法
        processed = self.handle_dameng_tablespaces(&processed);
        
        // 2. 处理Dameng特有的函数
        processed = self.handle_dameng_functions(&processed);
        
        // 3. 处理Dameng特有的数据类型
        processed = self.handle_dameng_data_types(&processed);
        
        processed
    }

    /// 处理Dameng特有的表空间语法
    fn handle_dameng_tablespaces(&self, sql: &str) -> String {
        // Dameng有特定的表空间语法
        // 这里可以添加对表空间语法的特殊处理
        let processed = sql.replace("IN TABLESPACE DATA", "");
        processed.replace("IN INDEXSPACE INDEX", "")
    }

    /// 处理Dameng特有的函数
    fn handle_dameng_functions(&self, sql: &str) -> String {
        // 替换Dameng特有的函数为标准函数
        let processed = sql.replace("SYSDATE", "NOW()");
        processed.replace("SYS_GUID()", "UUID()")
    }

    /// 处理Dameng特有的数据类型
    fn handle_dameng_data_types(&self, sql: &str) -> String {
        // Dameng有一些特定的数据类型
        let processed = sql.replace("DMTEXT", "VARCHAR(2000)");
        processed.replace("DMBLOB", "BLOB")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OperationType;

    #[test]
    fn test_dameng_adapter_can_handle() {
        let adapter = DamengAdapter::new();
        assert!(adapter.can_handle(&DatabaseType::Dameng));
        assert!(!adapter.can_handle(&DatabaseType::Oracle));
    }

    #[test]
    fn test_dameng_adapter_get_database_type() {
        let adapter = DamengAdapter::new();
        assert_eq!(adapter.get_database_type(), DatabaseType::Dameng);
    }

    #[test]
    fn test_dameng_adapter_parse_simple_sql() {
        let adapter = DamengAdapter::new();
        let sql = "SELECT id, name FROM users WHERE status = 'active'";
        
        let result = adapter.extract_metadata(sql).expect("解析失败");
        assert_eq!(result.database_type, DatabaseType::Dameng);
        assert_eq!(result.operation_type, OperationType::SELECT);
        assert!(result.tables_contains("users"));
        assert!(result.columns_contains("id"));
        assert!(result.columns_contains("name"));
    }

    #[test]
    fn test_dameng_adapter_handle_functions() {
        let adapter = DamengAdapter::new();
        let sql = "SELECT SYSDATE, SYS_GUID() FROM DUAL";
        
        let processed = adapter.handle_dameng_functions(sql);
        assert!(processed.contains("NOW()"));
        assert!(processed.contains("UUID()"));
    }
}