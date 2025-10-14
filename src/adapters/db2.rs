use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

/// DB2数据库适配器
pub struct DB2Adapter {
    common: CommonAdapter,
}

impl DB2Adapter {
    /// 创建新的DB2适配器实例
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::DB2),
        }
    }
}

impl super::DatabaseAdapter for DB2Adapter {
    /// 检查是否可以处理指定的数据库类型
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::DB2)
    }

    /// 解析审计日志
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    /// 获取数据库类型
    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::DB2
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

impl DB2Adapter {
    /// 解析DB2特定的SQL语法
    pub fn parse_db2_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_db2_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    /// 处理DB2特有的SQL语法
    fn process_db2_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理DB2特有的语法元素
        // 1. 处理DB2特有的表空间语法
        processed = self.handle_db2_tablespaces(&processed);
        
        // 2. 处理DB2特有的函数
        processed = self.handle_db2_functions(&processed);
        
        // 3. 处理DB2特有的数据类型
        processed = self.handle_db2_data_types(&processed);
        
        processed
    }

    /// 处理DB2特有的表空间语法
    fn handle_db2_tablespaces(&self, sql: &str) -> String {
        // DB2有特定的表空间语法
        // 这里可以添加对表空间语法的特殊处理
        let processed = sql.replace("IN USERSPACE1", "");
        processed.replace("IN DMS_TS", "")
    }

    /// 处理DB2特有的函数
    fn handle_db2_functions(&self, sql: &str) -> String {
        // 替换DB2特有的函数为标准函数
        let processed = sql.replace("CURRENT TIMESTAMP", "NOW()");
        processed.replace("GENERATE_UNIQUE()", "UUID()")
    }

    /// 处理DB2特有的数据类型
    fn handle_db2_data_types(&self, sql: &str) -> String {
        // DB2有一些特定的数据类型
        let processed = sql.replace("DB2SECURITYLABEL", "VARCHAR(255)");
        processed.replace("DB2XML", "XMLTYPE")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::OperationType;

    #[test]
    fn test_db2_adapter_can_handle() {
        let adapter = DB2Adapter::new();
        assert!(adapter.can_handle(&DatabaseType::DB2));
        assert!(!adapter.can_handle(&DatabaseType::Oracle));
    }

    #[test]
    fn test_db2_adapter_get_database_type() {
        let adapter = DB2Adapter::new();
        assert_eq!(adapter.get_database_type(), DatabaseType::DB2);
    }

    #[test]
    fn test_db2_adapter_parse_simple_sql() {
        let adapter = DB2Adapter::new();
        let sql = "SELECT id, name FROM users WHERE status = 'active'";
        
        let result = adapter.extract_metadata(sql).expect("解析失败");
        assert_eq!(result.database_type, DatabaseType::DB2);
        assert_eq!(result.operation_type, OperationType::SELECT);
        assert!(result.tables.contains("users"));
        assert!(result.columns.contains("id"));
        assert!(result.columns.contains("name"));
    }

    #[test]
    fn test_db2_adapter_handle_functions() {
        let adapter = DB2Adapter::new();
        let sql = "SELECT CURRENT TIMESTAMP, GENERATE_UNIQUE() FROM SYSIBM.SYSDUMMY1";
        
        let processed = adapter.handle_db2_functions(sql);
        assert!(processed.contains("NOW()"));
        assert!(processed.contains("UUID()"));
    }
}