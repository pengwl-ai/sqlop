// SQL转换模块
// 实现不同数据库方言间的SQL转换功能

use sqlparser::dialect::Dialect;
use crate::SqlopEngine;
use crate::core::types::{DatabaseType, ParseResult, ParserConfig, AuditLog};
use regex::Regex;
use std::collections::HashMap;

/// SQL转换器，负责在不同数据库方言间转换SQL
pub struct SqlTranspiler {
    source_dialect: Box<dyn Dialect>,
    target_dialect: Box<dyn Dialect>,
    source_type: DatabaseType,
    target_type: DatabaseType,
    
    // 函数映射
    function_mappings: HashMap<String, String>,
    
    // 数据类型映射
    type_mappings: HashMap<String, String>,
    
    // 特殊语法处理器
    special_syntax_handlers: Vec<Box<dyn SpecialSyntaxHandler>>,
}

impl SqlTranspiler {
    pub fn new(source_type: DatabaseType, target_type: DatabaseType) -> Self {
        let source_dialect = create_dialect(&source_type);
        let target_dialect = create_dialect(&target_type);
        
        let mut transpiler = Self {
            source_dialect,
            target_dialect,
            source_type,
            target_type,
            function_mappings: HashMap::new(),
            type_mappings: HashMap::new(),
            special_syntax_handlers: Vec::new(),
        };
        
        // 初始化映射关系
        transpiler.init_mappings();
        
        // 注册特殊语法处理器
        transpiler.register_special_handlers();
        
        transpiler
    }
    
    /// 初始化函数和类型映射
    fn init_mappings(&mut self) {
        // 根据源数据库和目标数据库类型设置不同的映射
        match (self.source_type.clone(), self.target_type.clone()) {
            (DatabaseType::MySQL, DatabaseType::PostgreSQL) => {
                // MySQL -> PostgreSQL映射
                self.function_mappings.insert("DATE_FORMAT".to_string(), "TO_CHAR".to_string());
                self.function_mappings.insert("NOW()".to_string(), "CURRENT_TIMESTAMP".to_string());
                self.function_mappings.insert("LIMIT".to_string(), "LIMIT".to_string()); // 相同
                
                self.type_mappings.insert("INT".to_string(), "INTEGER".to_string());
                self.type_mappings.insert("DATETIME".to_string(), "TIMESTAMP".to_string());
                self.type_mappings.insert("VARCHAR".to_string(), "TEXT".to_string());
            },
            (DatabaseType::PostgreSQL, DatabaseType::MySQL) => {
                // PostgreSQL -> MySQL映射
                self.function_mappings.insert("TO_CHAR".to_string(), "DATE_FORMAT".to_string());
                self.function_mappings.insert("CURRENT_TIMESTAMP".to_string(), "NOW()".to_string());
                self.function_mappings.insert("LIMIT".to_string(), "LIMIT".to_string()); // 相同
                
                self.type_mappings.insert("INTEGER".to_string(), "INT".to_string());
                self.type_mappings.insert("TIMESTAMP".to_string(), "DATETIME".to_string());
                self.type_mappings.insert("TEXT".to_string(), "VARCHAR(65535)".to_string());
            },
            // 其他数据库组合的映射
            _ => {
                // 默认映射
            }
        }
    }
    
    /// 注册特殊语法处理器
    fn register_special_handlers(&mut self) {
        // 根据源数据库和目标数据库类型注册处理器
        if self.source_type == DatabaseType::MySQL && self.target_type == DatabaseType::PostgreSQL {
            self.special_syntax_handlers.push(Box::new(MySQLToPostgreSQLHandler));
        } else if self.source_type == DatabaseType::PostgreSQL && self.target_type == DatabaseType::MySQL {
            self.special_syntax_handlers.push(Box::new(PostgreSQLToMySQLHandler));
        }
        
        // 注册通用处理器
        self.special_syntax_handlers.push(Box::new(GeneralSqlHandler));
    }
    
    /// 转换SQL语句
    pub fn transpile(&self, sql: &str) -> Result<String, String> {
        // 1. 解析SQL为AST
        let config = ParserConfig::default();
        let mut engine = SqlopEngine::new(Some(config)).map_err(|e| format!("Failed to create engine: {}", e))?;
        
        // 直接使用engine.parse_sql解析SQL
        let parse_result = engine.parse_sql(sql, self.source_type.clone())
            .map_err(|e| format!("解析源SQL失败: {}", e))?;
        
        // 2. 转换AST
        let transpiled_sql = self.transpile_ast(&parse_result)?;
        
        // 3. 应用特殊语法处理
        let mut final_sql = transpiled_sql;
        for handler in &self.special_syntax_handlers {
            final_sql = handler.handle(&final_sql, self.source_type.clone(), self.target_type.clone())?;
        }
        
        Ok(final_sql)
    }
    
    /// 转换AST
    fn transpile_ast(&self, parse_result: &ParseResult) -> Result<String, String> {
        // 这里简化处理，实际实现应该遍历AST并转换每个节点
        // 目前我们基于字符串进行转换
        let mut transpiled = parse_result.original_sql.clone();
        
        // 转换函数名
        for (source_func, target_func) in &self.function_mappings {
            let pattern = Regex::new(&format!(r"\b{}\b", source_func))
                .map_err(|e| format!("创建正则表达式失败: {}", e))?;
            transpiled = pattern.replace_all(&transpiled, target_func).to_string();
        }
        
        // 转换数据类型
        for (source_type, target_type) in &self.type_mappings {
            let pattern = Regex::new(&format!(r"\b{}\b", source_type))
                .map_err(|e| format!("创建正则表达式失败: {}", e))?;
            transpiled = pattern.replace_all(&transpiled, target_type).to_string();
        }
        
        Ok(transpiled)
    }
    
    /// 获取支持的目标数据库类型
    pub fn get_supported_target_types(source_type: DatabaseType) -> Vec<DatabaseType> {
        // 返回源数据库类型支持转换到的所有目标数据库类型
        match source_type {
            DatabaseType::MySQL => vec![DatabaseType::PostgreSQL, DatabaseType::SQLServer, DatabaseType::Oracle],
            DatabaseType::PostgreSQL => vec![DatabaseType::MySQL, DatabaseType::SQLServer, DatabaseType::Oracle],
            DatabaseType::SQLServer => vec![DatabaseType::MySQL, DatabaseType::PostgreSQL, DatabaseType::Oracle],
            DatabaseType::Oracle => vec![DatabaseType::MySQL, DatabaseType::PostgreSQL, DatabaseType::SQLServer],
            _ => vec![DatabaseType::MySQL, DatabaseType::PostgreSQL, DatabaseType::SQLServer],
        }
    }
}

/// 特殊语法处理器接口
pub trait SpecialSyntaxHandler {
    fn handle(&self, sql: &str, source_type: DatabaseType, target_type: DatabaseType) -> Result<String, String>;
}

/// MySQL到PostgreSQL的特殊处理器
struct MySQLToPostgreSQLHandler;

impl SpecialSyntaxHandler for MySQLToPostgreSQLHandler {
    fn handle(&self, sql: &str, source_type: DatabaseType, target_type: DatabaseType) -> Result<String, String> {
        if source_type != DatabaseType::MySQL || target_type != DatabaseType::PostgreSQL {
            return Ok(sql.to_string());
        }
        
        let mut handled = sql.to_string();
        
        // 处理LIMIT/OFFSET语法（两者相同，无需转换）
        
        // 处理字符串连接运算符
        handled = handled.replace("CONCAT(", "(");
        handled = handled.replace("||", "||"); // PostgreSQL原生支持||连接字符串
        
        // 处理IFNULL函数 -> COALESCE
        handled = Regex::new(r"IFNULL\s*\(\s*(.+)\s*,\s*(.+)\s*\)")
            .map_err(|e| format!("创建正则表达式失败: {}", e))?
            .replace_all(&handled, "COALESCE($1, $2)").to_string();
        
        // 处理日期函数
        handled = Regex::new(r"DATE_FORMAT\s*\(\s*(.+)\s*,\s*'(.+)'\s*\)")
            .map_err(|e| format!("创建正则表达式失败: {}", e))?
            .replace_all(&handled, "TO_CHAR($1, '$2')").to_string();
        
        Ok(handled)
    }
}

/// PostgreSQL到MySQL的特殊处理器
struct PostgreSQLToMySQLHandler;

impl SpecialSyntaxHandler for PostgreSQLToMySQLHandler {
    fn handle(&self, sql: &str, source_type: DatabaseType, target_type: DatabaseType) -> Result<String, String> {
        if source_type != DatabaseType::PostgreSQL || target_type != DatabaseType::MySQL {
            return Ok(sql.to_string());
        }
        
        let mut handled = sql.to_string();
        
        // 处理字符串连接运算符 || -> CONCAT
        handled = Regex::new(r"([^\s']+)\s*\|\|\s*([^\s']+)").map_err(|e| e.to_string())?
            .replace_all(&handled, "CONCAT($1, $2)").to_string();
        
        // 处理COALESCE函数 -> IFNULL
        handled = Regex::new(r"COALESCE\s*\(\s*(.+)\s*,\s*(.+)\s*\)")
            .map_err(|e| format!("创建正则表达式失败: {}", e))?
            .replace_all(&handled, "IFNULL($1, $2)").to_string();
        
        // 处理TO_CHAR函数 -> DATE_FORMAT
        handled = Regex::new(r"TO_CHAR\s*\(\s*(.+)\s*,\s*'(.+)'\s*\)")
            .map_err(|e| format!("创建正则表达式失败: {}", e))?
            .replace_all(&handled, "DATE_FORMAT($1, '$2')").to_string();
        
        Ok(handled)
    }
}

/// 通用SQL处理器
struct GeneralSqlHandler;

impl SpecialSyntaxHandler for GeneralSqlHandler {
    fn handle(&self, sql: &str, _source_type: DatabaseType, _target_type: DatabaseType) -> Result<String, String> {
        // 处理通用SQL语法问题
        let mut handled = sql.to_string();
        
        // 移除多余的空白字符
        handled = Regex::new(r"\s+")
            .map_err(|e| format!("创建正则表达式失败: {}", e))?
            .replace_all(&handled, " ").to_string();
        
        // 标准化关键字大小写
        handled = handled.to_uppercase();
        
        Ok(handled)
    }
}

/// 根据数据库类型创建对应的方言解析器
fn create_dialect(database_type: &DatabaseType) -> Box<dyn Dialect> {
     use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, MsSqlDialect};
    use crate::adapters::dialects::{EnhancedMySqlDialect, SQLiteDialect, HiveDialect, DB2Dialect, DamengDialect, GaussDBDialect, KingbaseDialect, HighgoDialect, GreenplumDialect, VastbaseDialect};
    
    match database_type {
        DatabaseType::MySQL => {
            // 默认使用增强的MySQL方言
            Box::new(EnhancedMySqlDialect::new())
        },
        DatabaseType::PostgreSQL => {
            Box::new(PostgreSqlDialect {})
        },
        DatabaseType::SQLServer => {
            Box::new(MsSqlDialect {})
        },
        DatabaseType::Oracle => {
            // Oracle 使用 PostgreSQL 方言作为基础，后续可以优化
            Box::new(PostgreSqlDialect {})
        },
        DatabaseType::SQLite => {
            Box::new(SQLiteDialect)
        },
        DatabaseType::Hive => {
            Box::new(HiveDialect)
        },
        DatabaseType::DB2 => {
            Box::new(DB2Dialect)
        },
        DatabaseType::Dameng => {
            Box::new(DamengDialect)
        },
        DatabaseType::GaussDB => {
            Box::new(GaussDBDialect)
        },
        DatabaseType::Kingbase => {
            Box::new(KingbaseDialect)
        },
        DatabaseType::Highgo => {
            Box::new(HighgoDialect)
        },
        DatabaseType::Greenplum => {
            Box::new(GreenplumDialect)
        },
        DatabaseType::Vastbase => {
            Box::new(VastbaseDialect)
        },
        // 对于未知的数据库类型，使用通用方言
        _ => {
            Box::new(GenericDialect {})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mysql_to_postgresql_transpile() {
        let transpiler = SqlTranspiler::new(DatabaseType::MySQL, DatabaseType::PostgreSQL);
        let mysql_sql = "SELECT id, name, DATE_FORMAT(created_at, '%Y-%m-%d') AS create_date FROM users LIMIT 10";
        let result = transpiler.transpile(mysql_sql).unwrap();
        
        assert!(result.contains("SELECT"));
        assert!(result.contains("TO_CHAR"));
        assert!(result.contains("LIMIT"));
    }
    
    #[test]
    fn test_postgresql_to_mysql_transpile() {
        let transpiler = SqlTranspiler::new(DatabaseType::PostgreSQL, DatabaseType::MySQL);
        let pg_sql = "SELECT id, name, TO_CHAR(created_at, 'YYYY-MM-DD') AS create_date FROM users LIMIT 10";
        let result = transpiler.transpile(pg_sql).unwrap();
        
        assert!(result.contains("SELECT"));
        assert!(result.contains("DATE_FORMAT"));
        assert!(result.contains("LIMIT"));
    }
}