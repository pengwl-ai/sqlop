use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

/// Sybase数据库适配器
pub struct SybaseAdapter {
    common: CommonAdapter,
}

impl SybaseAdapter {
    /// 创建新的Sybase适配器实例
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::Sybase),
        }
    }
}

impl super::DatabaseAdapter for SybaseAdapter {
    /// 检查是否可以处理指定的数据库类型
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::Sybase)
    }

    /// 解析审计日志
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    /// 获取数据库类型
    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::Sybase
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

impl SybaseAdapter {
    /// 解析Sybase特定的SQL语法
    pub fn parse_sybase_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_sybase_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    /// 处理Sybase特有的SQL语法
    fn process_sybase_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理Sybase特有的语法元素
        // 1. 处理Sybase特有的标识符引用方式
        processed = self.handle_sybase_identifiers(&processed);
        
        // 2. 处理Sybase特有的函数
        processed = self.handle_sybase_functions(&processed);
        
        // 3. 处理Sybase特有的存储过程语法
        processed = self.handle_sybase_procedures(&processed);
        
        processed
    }

    /// 处理Sybase特有的标识符引用方式
    fn handle_sybase_identifiers(&self, sql: &str) -> String {
        // Sybase支持方括号作为标识符引用
        // 这里将方括号标识符转换为双引号标识符
        let processed = sql.replace(
            '[', 
            '"'.to_string().as_str()
        ).replace(
            ']', 
            '"'.to_string().as_str()
        );
        processed
    }

    /// 处理Sybase特有的函数
    fn handle_sybase_functions(&self, sql: &str) -> String {
        // 替换Sybase特有的函数为标准函数
        let processed = sql.replace("GETDATE()", "NOW()");
        processed.replace("NEWID()", "UUID()")
    }

    /// 处理Sybase特有的存储过程语法
    fn handle_sybase_procedures(&self, sql: &str) -> String {
        // Sybase有特定的存储过程语法
        // 这里可以添加对这种语法的特殊处理
        sql.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OperationType;

    #[test]
    fn test_sybase_adapter_can_handle() {
        let adapter = SybaseAdapter::new();
        assert!(adapter.can_handle(&DatabaseType::Sybase));
        assert!(!adapter.can_handle(&DatabaseType::MySQL));
    }

    #[test]
    fn test_sybase_adapter_get_database_type() {
        let adapter = SybaseAdapter::new();
        assert_eq!(adapter.get_database_type(), DatabaseType::Sybase);
    }

    #[test]
    fn test_sybase_adapter_parse_simple_sql() {
        let adapter = SybaseAdapter::new();
        let sql = "SELECT id, name FROM users WHERE status = 'active'";
        
        let result = adapter.extract_metadata(sql).expect("解析失败");
        assert_eq!(result.database_type, DatabaseType::Sybase);
        assert_eq!(result.operation_type, OperationType::SELECT);
        assert!(result.tables_contains("users"));
        assert!(result.columns_contains("id"));
        assert!(result.columns_contains("name"));
    }

    #[test]
    fn test_sybase_adapter_handle_identifiers() {
        let adapter = SybaseAdapter::new();
        let sql = "SELECT [id], [name] FROM [users]";
        
        let processed = adapter.handle_sybase_identifiers(sql);
        assert!(processed.contains('"'));
        assert!(!processed.contains('['));
        assert!(!processed.contains(']'));
    }
}