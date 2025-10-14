use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

pub struct SQLiteAdapter {
    common: CommonAdapter,
}

impl SQLiteAdapter {
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::SQLite),
        }
    }
}

impl super::DatabaseAdapter for SQLiteAdapter {
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::SQLite)
    }

    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::SQLite
    }

    fn normalize_sql(&self, sql: &str) -> String {
        self.common.normalize_sql(sql)
    }

    fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        self.common.extract_metadata(sql)
    }
}

impl SQLiteAdapter {
    pub fn parse_sqlite_specific(&self, sql: &str) -> Result<ParseResult> {
        // SQLite 特定的解析逻辑
        let normalized_sql = self.normalize_sql(sql);
        
        // 处理 SQLite 特有的语法
        let processed_sql = self.process_sqlite_syntax(&normalized_sql);
        
        self.extract_metadata(&processed_sql)
    }
    
    fn process_sqlite_syntax(&self, sql: &str) -> String {
        // 处理 SQLite 特有的语法特性
        let mut processed = sql.to_string();
        
        // 处理 SQLite 特有的数据类型
        let sqlite_types = vec![
            ("INTEGER", "INTEGER"),
            ("TEXT", "TEXT"),
            ("REAL", "DOUBLE PRECISION"),
            ("BLOB", "BLOB"),
            ("NULL", "NULL"),
        ];
        
        for (sqlite_type, standard_type) in sqlite_types {
            processed = processed.replace(sqlite_type, standard_type);
        }
        
        // 处理 SQLite 特有的 AUTOINCREMENT 关键字
        if processed.contains("AUTOINCREMENT") {
            processed = processed.replace("AUTOINCREMENT", "IDENTITY");
        }
        
        processed
    }
}