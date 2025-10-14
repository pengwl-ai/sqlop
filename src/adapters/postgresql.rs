use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

pub struct PostgreSQLAdapter {
    common: CommonAdapter,
}

impl PostgreSQLAdapter {
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::PostgreSQL),
        }
    }
}

impl super::DatabaseAdapter for PostgreSQLAdapter {
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::PostgreSQL)
    }

    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }

    fn normalize_sql(&self, sql: &str) -> String {
        self.common.normalize_sql(sql)
    }

    fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        self.common.extract_metadata(sql)
    }
}

impl PostgreSQLAdapter {
    pub fn parse_postgresql_specific(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let processed_sql = self.process_postgresql_syntax(&normalized_sql);
        self.extract_metadata(&processed_sql)
    }

    fn process_postgresql_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理 PostgreSQL 特有的注释语法
        processed = self.process_postgresql_comments(&processed);
        
        // 处理 PostgreSQL 特有的函数
        processed = self.process_postgresql_functions(&processed);
        
        // 处理 PostgreSQL 特有的数据类型
        processed = self.process_postgresql_types(&processed);
        
        processed
    }

    fn process_postgresql_comments(&self, sql: &str) -> String {
        // PostgreSQL 支持 C 风格的注释
        let re = regex::Regex::new(r"/\*.*?\*/").unwrap();
        re.replace_all(sql, "").to_string()
    }

    fn process_postgresql_functions(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理 PostgreSQL 特有的函数
        let pg_functions = vec![
            ("NOW()", "CURRENT_TIMESTAMP"),
            ("CURRENT_TIMESTAMP", "CURRENT_TIMESTAMP"),
            ("CURRENT_DATE", "CURRENT_DATE"),
            ("CURRENT_TIME", "CURRENT_TIME"),
            ("EXTRACT", "EXTRACT"),
            ("TO_CHAR", "TO_CHAR"),
            ("TO_DATE", "TO_DATE"),
            ("TO_TIMESTAMP", "TO_TIMESTAMP"),
            ("ARRAY_AGG", "ARRAY_AGG"),
            ("STRING_AGG", "STRING_AGG"),
            ("JSONB_AGG", "JSONB_AGG"),
            ("ROW_NUMBER", "ROW_NUMBER"),
            ("RANK", "RANK"),
            ("DENSE_RANK", "DENSE_RANK"),
        ];
        
        for (pg_func, standard_func) in pg_functions {
            processed = processed.replace(pg_func, standard_func);
        }
        
        processed
    }

    fn process_postgresql_types(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理 PostgreSQL 特有的数据类型
        let pg_types = vec![
            ("SERIAL", "INTEGER"),
            ("BIGSERIAL", "BIGINT"),
            ("TEXT", "TEXT"),
            ("VARCHAR", "VARCHAR"),
            ("CHAR", "CHAR"),
            ("BOOLEAN", "BOOLEAN"),
            ("BOOL", "BOOLEAN"),
            ("JSON", "JSON"),
            ("JSONB", "JSONB"),
            ("UUID", "UUID"),
            ("BYTEA", "BYTEA"),
            ("INET", "INET"),
            ("CIDR", "CIDR"),
            ("MACADDR", "MACADDR"),
            ("POINT", "POINT"),
            ("LINE", "LINE"),
            ("LSEG", "LSEG"),
            ("BOX", "BOX"),
            ("PATH", "PATH"),
            ("POLYGON", "POLYGON"),
            ("CIRCLE", "CIRCLE"),
        ];
        
        for (pg_type, standard_type) in pg_types {
            processed = processed.replace(pg_type, standard_type);
        }
        
        processed
    }

    pub fn extract_postgresql_metadata(&self, sql: &str) -> Result<ParseResult> {
        let mut result = self.extract_metadata(sql)?;
        
        // 处理 PostgreSQL 特有的系统表
        if sql.contains("information_schema") || sql.contains("pg_catalog") {
            result.databases.insert("information_schema".to_string());
            result.databases.insert("pg_catalog".to_string());
        }
        
        // 处理 PostgreSQL 特有的系统函数
        if sql.contains("pg_") {
            let re = regex::Regex::new(r"pg_([a-zA-Z_]+)").unwrap();
            for cap in re.captures_iter(sql) {
                if let Some(func_name) = cap.get(1) {
                    log::debug!("PostgreSQL 系统函数引用: pg_{}", func_name.as_str());
                }
            }
        }
        
        // 处理 PostgreSQL 特有的操作符
        if sql.contains("::") {
            let re = regex::Regex::new(r"::([a-zA-Z_]+)").unwrap();
            for cap in re.captures_iter(sql) {
                if let Some(type_name) = cap.get(1) {
                    log::debug!("PostgreSQL 类型转换: ::{}", type_name.as_str());
                }
            }
        }
        
        Ok(result)
    }

    pub fn handle_postgresql_extensions(&self, sql: &str) -> Result<ParseResult> {
        let result = self.extract_metadata(sql)?;
        
        // 处理 PostgreSQL 扩展
        let extensions = vec![
            "postgis", "pgcrypto", "uuid-ossp", "pg_stat_statements", 
            "pg_buffercache", "pg_freespacemap", "pgrowlocks", "pg_trgm"
        ];
        
        for ext in extensions {
            if sql.contains(ext) {
                log::debug!("PostgreSQL 扩展使用: {}", ext);
            }
        }
        
        Ok(result)
    }
}