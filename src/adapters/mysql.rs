use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};
use super::DatabaseAdapter;

pub struct MySQLAdapter {
    common: CommonAdapter,
}

impl MySQLAdapter {
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::MySQL),
        }
    }
}

impl super::DatabaseAdapter for MySQLAdapter {
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::MySQL)
    }

    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::MySQL
    }

    fn normalize_sql(&self, sql: &str) -> String {
        self.common.normalize_sql(sql)
    }

    fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        self.common.extract_metadata(sql)
    }
}

impl MySQLAdapter {
    pub fn parse_mysql_specific(&self, sql: &str) -> Result<ParseResult> {
        // MySQL 特定的解析逻辑
        let normalized_sql = self.normalize_sql(sql);
        
        // 处理 MySQL 特有的语法
        let processed_sql = self.process_mysql_syntax(&normalized_sql);
        
        self.extract_metadata(&processed_sql)
    }

    fn process_mysql_syntax(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 处理 MySQL 特有的注释语法
        processed = self.process_mysql_comments(&processed);
        
        // 处理 MySQL 特有的函数
        processed = self.process_mysql_functions(&processed);
        
        // 处理 MySQL 特有的关键字
        processed = self.process_mysql_keywords(&processed);
        
        processed
    }

    fn process_mysql_comments(&self, sql: &str) -> String {
        // 移除 MySQL 特有的注释
        let re = regex::Regex::new(r"/\*![0-9]+ (.*?)\*/").unwrap();
        re.replace_all(sql, "").to_string()
    }

    fn process_mysql_functions(&self, sql: &str) -> String {
        // 处理 MySQL 特有的函数，如 NOW(), CURDATE() 等
        let mut processed = sql.to_string();
        
        // 将 MySQL 函数转换为标准 SQL
        let mysql_functions = vec![
            ("NOW()", "CURRENT_TIMESTAMP"),
            ("CURDATE()", "CURRENT_DATE"),
            ("CURTIME()", "CURRENT_TIME"),
            ("UNIX_TIMESTAMP()", "EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)"),
            ("FROM_UNIXTIME", "TO_TIMESTAMP"),
            ("DATE_FORMAT", "TO_CHAR"),
            ("IF", "CASE"),
        ];
        
        for (mysql_func, standard_func) in mysql_functions {
            processed = processed.replace(mysql_func, standard_func);
        }
        
        processed
    }

    fn process_mysql_keywords(&self, sql: &str) -> String {
        // 处理 MySQL 特有的关键字
        let mut processed = sql.to_string();
        
        // 处理 MySQL 特有的数据类型
        let mysql_types = vec![
            ("TINYINT", "SMALLINT"),
            ("MEDIUMINT", "INTEGER"),
            ("BIGINT", "BIGINT"),
            ("TEXT", "TEXT"),
            ("VARCHAR", "VARCHAR"),
            ("CHAR", "CHAR"),
            ("DATETIME", "TIMESTAMP"),
            ("TIMESTAMP", "TIMESTAMP"),
        ];
        
        for (mysql_type, standard_type) in mysql_types {
            processed = processed.replace(mysql_type, standard_type);
        }
        
        processed
    }

    pub fn extract_mysql_metadata(&self, sql: &str) -> Result<ParseResult> {
        // 提取 MySQL 特有的元数据
        let mut result = self.extract_metadata(sql)?;
        
        // 处理 MySQL 特有的系统表
        if sql.contains("information_schema") {
            result.databases.insert("information_schema".to_string());
        }
        
        // 处理 MySQL 特有的系统变量
        if sql.contains("@@") {
            // 提取系统变量引用
            let re = regex::Regex::new(r"@@([a-zA-Z_]+)").unwrap();
            for cap in re.captures_iter(sql) {
                if let Some(var_name) = cap.get(1) {
                    // 可以记录系统变量的使用
                    log::debug!("MySQL 系统变量引用: {}", var_name.as_str());
                }
            }
        }
        
        Ok(result)
    }
}