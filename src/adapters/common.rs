use crate::core::error::{ParseError, Result};
use crate::core::types::{AuditLog, DatabaseType, OperationType, ParseResult, SqlObject};
use sqlparser::ast::{Statement, TableFactor, ObjectName};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, MsSqlDialect};
use sqlparser::parser::Parser;
use std::collections::HashSet;
// use std::collections::HashMap;

// 导入自定义方言
use super::dialects::{SQLiteDialect, HiveDialect, DB2Dialect, DamengDialect, OracleDialect, GaussDBDialect, KingbaseDialect, HighgoDialect, GreenplumDialect, VastbaseDialect};

pub struct CommonAdapter {
    db_type: DatabaseType,
    sql_keywords: HashSet<String>,
}

impl CommonAdapter {
    pub fn new(db_type: DatabaseType) -> Self {
        let sql_keywords = Self::initialize_sql_keywords();
        Self { 
            db_type, 
            sql_keywords 
        }
    }
    
    fn initialize_sql_keywords() -> HashSet<String> {
        let keywords = vec![
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP",
            "ALTER", "TABLE", "INDEX", "VIEW", "JOIN", "INNER", "OUTER", "LEFT", "RIGHT",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "UNION", "AND",
            "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL", "COUNT", "SUM",
            "AVG", "MIN", "MAX", "DISTINCT", "AS", "CASE", "WHEN", "THEN", "ELSE", "END",
            "TRUE", "FALSE", "INT", "VARCHAR", "TEXT", "FLOAT", "DOUBLE", "DECIMAL", "DATE",
            "TIME", "TIMESTAMP", "INTERVAL", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
            "ALL", "ANY", "SOME", "DISTINCT", "UNIQUE", "PRIMARY", "KEY", "FOREIGN", "REFERENCES",
            "CHECK", "DEFAULT", "CONSTRAINT", "AUTO_INCREMENT", "UNSIGNED", "NULLS", "FIRST", "LAST",
            "ASC", "DESC", "PARTITION", "DISTRIBUTE", "SORT", "BUCKET", "OVER", "PARTITION BY", "ORDER BY",
            "ROWS", "RANGE", "PRECEDING", "FOLLOWING", "CURRENT ROW", "UNBOUNDED PRECEDING", "UNBOUNDED FOLLOWING",
            "WITH", "AS", "TEMPORARY", "GLOBAL", "LOCAL", "IF", "EXISTS", "NOT EXISTS", "ONLY", "CASCADE",
            "RESTRICT", "SET", "VALUES", "INTO", "ON", "DUPLICATE", "KEY", "UPDATE", "IGNORE", "MERGE",
            "EXPLAIN", "ANALYZE", "DESCRIBE", "SHOW", "USE", "DATABASE", "SCHEMA", "TRANSACTION", "COMMIT",
            "ROLLBACK", "SAVEPOINT", "RELEASE", "LOCK", "UNLOCK", "BEGIN", "START", "END", "ISOLATION", "LEVEL",
            "SERIALIZABLE", "REPEATABLE READ", "READ COMMITTED", "READ UNCOMMITTED", "COMMITTED", "UNCOMMITTED",
            "FOR", "UPDATE", "SHARE", "NOWAIT", "KEY SHARE", "NO KEY UPDATE", "OF", "BY", "TO", "INNER",
            "LEFT", "RIGHT", "FULL", "OUTER", "CROSS", "NATURAL", "SELF", "ANTI", "SEMI", "LATERAL",
            "UNION", "INTERSECT", "EXCEPT", "ALL", "DISTINCT", "GROUP BY", "ORDER BY", "HAVING", "LIMIT",
            "OFFSET", "FETCH", "FIRST", "NEXT", "ROW", "ROWS", "ONLY", "PERCENT", "TIES", "SAMPLE",
            "TABLESAMPLE", "BUCKET", "ON", "RAND", "RANDOM", "SEED", "OUTPUT", "INTO", "VARIABLE", "VIEW",
            "MATERIALIZED", "OR REPLACE", "IF NOT EXISTS", "DROP", "CASCADE", "RESTRICT", "TRUNCATE", "CONTINUE IDENTITY",
            "RESTART IDENTITY", "PRESERVE IDENTITY", "OWNED BY", "ALL TABLES", "ALL SEQUENCES", "ALL INDEXES",
            "ALL VIEWS", "ALL MATERIALIZED VIEWS", "ALL FUNCTIONS", "ALL PROCEDURES", "ALL TRIGGERS", "ALL RULES",
            "ALL TYPES", "ALL DOMAINS", "ALL COLLATIONS", "ALL CONVERSIONS", "ALL SCHEMAS", "ALL EXTENSIONS",
            "ALL LANGUAGES", "ALL DATA TYPES", "ALL OPERATORS", "ALL OPERATOR CLASSES", "ALL OPERATOR FAMILIES",
            "ALL AGGREGATES", "ALL CASTS", "ALL USERS", "ALL ROLES", "ALL GROUPS", "ALL RESOURCE GROUPS",
            "ALL TABLESPACES", "ALL OBJECTS", "DATABASES", "SCHEMAS", "TABLES", "VIEW", "MATERIALIZED VIEW",
            "INDEX", "SEQUENCE", "FUNCTION", "PROCEDURE", "TRIGGER", "RULE", "TYPE", "DOMAIN", "COLLATION",
            "CONVERSION", "EXTENSION", "LANGUAGE", "DATA TYPE", "OPERATOR", "OPERATOR CLASS", "OPERATOR FAMILY",
            "AGGREGATE", "CAST", "USER", "ROLE", "GROUP", "RESOURCE GROUP", "TABLESPACE", "OBJECT", "DATABASE",
            "SCHEMA", "DEFAULT", "PUBLIC", "CURRENT_USER", "SESSION_USER", "SYSTEM_USER", "USER", "CURRENT_SCHEMA",
            "CURRENT_DATABASE", "CURRENT_CATALOG", "CURRENT_SCHEMA", "CURRENT_PATH", "CURRENT_DATE", "CURRENT_TIME",
            "CURRENT_TIMESTAMP", "LOCALTIME", "LOCALTIMESTAMP", "NOW", "EXTRACT", "CAST", "CONVERT", "TRIM",
            "LTRIM", "RTRIM", "BTRIM", "UPPER", "LOWER", "INITCAP", "POSITION", "SUBSTRING", "OVERLAY",
            "CONCAT", "||", "LENGTH", "CHAR_LENGTH", "CHARACTER_LENGTH", "BIT_LENGTH", "OCTET_LENGTH",
            "COLLATE", "ASCII", "CHR", "TO_CHAR", "TO_DATE", "TO_NUMBER", "TO_TIMESTAMP", "TO_BINARY",
            "TO_HEX", "FROM_HEX", "ENCODE", "DECODE", "MD5", "SHA1", "SHA256", "SHA512", "CRC32",
            "RAND", "RANDOM", "FLOOR", "CEIL", "CEILING", "ROUND", "TRUNC", "ABS", "SIGN", "MOD",
            "POWER", "SQRT", "EXP", "LOG", "LN", "LOG10", "LOG2", "SIN", "COS", "TAN", "ASIN",
            "ACOS", "ATAN", "ATAN2", "SINH", "COSH", "TANH", "COT", "DEGREES", "RADIANS", "PI",
            "E", "CURRENT_ROLE", "CURRENT_TRANSACTION", "CURRENT_CATALOG", "CURRENT_SCHEMA", "CURRENT_PATH",
            "CURRENT_USER", "SESSION_USER", "SYSTEM_USER", "USER", "CURRENT_DATE", "CURRENT_TIME",
            "CURRENT_TIMESTAMP", "LOCALTIME", "LOCALTIMESTAMP", "NOW", "EXTRACT", "CAST", "CONVERT",
            "TRIM", "LTRIM", "RTRIM", "BTRIM", "UPPER", "LOWER", "INITCAP", "POSITION", "SUBSTRING",
            "OVERLAY", "CONCAT", "||", "LENGTH", "CHAR_LENGTH", "CHARACTER_LENGTH", "BIT_LENGTH",
            "OCTET_LENGTH", "COLLATE", "ASCII", "CHR", "TO_CHAR", "TO_DATE", "TO_NUMBER", "TO_TIMESTAMP",
            "TO_BINARY", "TO_HEX", "FROM_HEX", "ENCODE", "DECODE", "MD5", "SHA1", "SHA256", "SHA512",
            "CRC32", "RAND", "RANDOM", "FLOOR", "CEIL", "CEILING", "ROUND", "TRUNC", "ABS", "SIGN",
            "MOD", "POWER", "SQRT", "EXP", "LOG", "LN", "LOG10", "LOG2", "SIN", "COS", "TAN",
            "ASIN", "ACOS", "ATAN", "ATAN2", "SINH", "COSH", "TANH", "COT", "DEGREES", "RADIANS", "PI",
            "E", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0" // 添加数字，这些不应该被识别为列名
        ];
        
        keywords.into_iter().map(|k| k.to_string()).collect()
    }
    
    fn is_sql_keyword(&self, identifier: &str) -> bool {
        // 转大写后检查
        self.sql_keywords.contains(&identifier.to_uppercase())
    }

    pub fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        let dialect = self.get_dialect(&audit_log.database_type)?;
        self.parse_sql_with_dialect(&audit_log.sql_text, &audit_log.database_type, dialect)
    }

    pub fn normalize_sql(&self, sql: &str) -> String {
        let mut normalized = sql.to_string();
        
        // 移除注释
        normalized = self.remove_comments(&normalized);
        
        // 标准化空白字符
        normalized = self.normalize_whitespace(&normalized);
        
        // 标准化大小写
        normalized = self.normalize_case(&normalized);
        
        // 标准化引号
        normalized = self.normalize_quotes(&normalized);
        
        normalized
    }

    pub fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let dialect = self.get_dialect(&self.db_type)?;
        self.parse_sql_with_dialect(&normalized_sql, &self.db_type, dialect)
    }

    fn get_dialect(&self, db_type: &DatabaseType) -> Result<Box<dyn Dialect>> {
        match db_type {
            DatabaseType::MySQL => Ok(Box::new(MySqlDialect {})),
            DatabaseType::PostgreSQL => Ok(Box::new(PostgreSqlDialect {})),
            DatabaseType::SQLServer => Ok(Box::new(MsSqlDialect {})),
            DatabaseType::Oracle => Ok(Box::new(OracleDialect {})),
            DatabaseType::Hive => Ok(Box::new(HiveDialect {})),
            DatabaseType::GaussDB => Ok(Box::new(GaussDBDialect {})),
            DatabaseType::Kingbase => Ok(Box::new(KingbaseDialect {})),
            DatabaseType::Highgo => Ok(Box::new(HighgoDialect {})),
            DatabaseType::Greenplum => Ok(Box::new(GreenplumDialect {})),
            DatabaseType::Vastbase => Ok(Box::new(VastbaseDialect {})),
            DatabaseType::Sybase => Ok(Box::new(MySqlDialect {})),
            DatabaseType::DB2 => Ok(Box::new(DB2Dialect {})),
            DatabaseType::Dameng => Ok(Box::new(DamengDialect {})),
            DatabaseType::SQLite => Ok(Box::new(SQLiteDialect {})),
        }
    }

    fn parse_sql_with_dialect(
        &self,
        sql: &str,
        db_type: &DatabaseType,
        dialect: Box<dyn Dialect>,
    ) -> Result<ParseResult> {
        let mut parser = Parser::new(&*dialect).try_with_sql(sql)?;
        let statements = parser.parse_statements()?;

        if statements.is_empty() {
            return Err(ParseError::SqlParseError("无法解析 SQL 语句".to_string()));
        }

        let mut databases = HashSet::new();
        let mut schemas = HashSet::new();
        let mut tables = HashSet::new();
        let mut columns = HashSet::new();
        let mut objects = Vec::new();
        let mut operation_type = OperationType::OTHER;

        for statement in &statements {
            self.extract_objects_from_statement(
                statement,
                &mut databases,
                &mut schemas,
                &mut tables,
                &mut columns,
                &mut objects,
                &mut operation_type,
            );
        }

        Ok(ParseResult {
            database_type: db_type.clone(),
            original_sql: sql.to_string(),
            databases,
            schemas,
            tables,
            columns,
            objects,
            operation_type,
            parse_time_ms: 0,
        })
    }

    fn extract_objects_from_statement(
        &self,
        statement: &Statement,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
        operation_type: &mut OperationType,
    ) {
        // 创建临时集合用于收集所有可能的标识符
        let mut all_tables = HashSet::new();
        let mut all_schemas = HashSet::new();
        let mut all_databases = HashSet::new();
        let mut all_columns = HashSet::new();
        let mut all_objects = Vec::new();

        match statement {
            Statement::Query(query) => {
                *operation_type = OperationType::SELECT;
                
                // 使用临时集合调用extract_from_query
                self.extract_from_query(query, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_columns, &mut all_objects);
            }
            Statement::Insert { table_name, columns: insert_columns, .. } => {
                *operation_type = OperationType::INSERT;
                // 提取表名信息到临时集合
                self.extract_table_name(table_name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
                
                // 处理INSERT语句中的列名
                for col in insert_columns {
                    let col_str = col.to_string();
                    all_columns.insert(col_str);
                }
            }
            Statement::Update { table, .. } => {
                *operation_type = OperationType::UPDATE;
                // 提取表名信息到临时集合
                match table.relation {
                    TableFactor::Table { ref name, .. } => {
                        self.extract_table_name(name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
                    },
                    _ => {}
                }
            }
            Statement::Delete { from, .. } => {
                *operation_type = OperationType::DELETE;
                // 提取表名信息到临时集合
                for table_ref in from {
                    match &table_ref.relation {
                        TableFactor::Table { ref name, .. } => {
                            self.extract_table_name(name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
                        },
                        _ => {}
                    }
                }
            }
            Statement::CreateTable { name, .. } => {
                *operation_type = OperationType::CREATE;
                // 提取表名信息到临时集合
                self.extract_table_name(name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
            }
            Statement::Drop { names, .. } => {
                *operation_type = OperationType::DROP;
                // 提取表名信息到临时集合
                for name in names {
                    self.extract_table_name(name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
                }
            }
            Statement::AlterTable { name, .. } => {
                *operation_type = OperationType::ALTER;
                // 提取表名信息到临时集合
                self.extract_table_name(name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
            }
            Statement::Truncate { table_name, .. } => {
                *operation_type = OperationType::TRUNCATE;
                // 提取表名信息到临时集合
                self.extract_table_name(table_name, &mut all_databases, &mut all_schemas, &mut all_tables, &mut all_objects);
            }
            _ => {}
        }
        
        // 定义SQL关键字列表（完整版本）
        let sql_keywords = [
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", 
            "ALTER", "TABLE", "VIEW", "INDEX", "JOIN", "INNER", "LEFT", "RIGHT", "OUTER",
            "FULL", "CROSS", "ON", "AS", "GROUP", "BY", "HAVING", "ORDER", "LIMIT", 
            "OFFSET", "DISTINCT", "ALL", "EXISTS", "IN", "BETWEEN", "LIKE", "ILIKE", 
            "AND", "OR", "NOT", "IS", "NULL", "TRUE", "FALSE", "UNION", "INTERSECT",
            "EXCEPT", "WITH", "AS", "OVER", "PARTITION", "ORDER", "LAG", "LEAD", 
            "FIRST_VALUE", "LAST_VALUE", "ROW_NUMBER", "RANK", "DENSE_RANK", "PERCENT_RANK",
            "CUME_DIST", "NTILE", "COLLATE", "CAST", "CONVERT", "TRY_CAST", "TRY_CONVERT",
            "IF", "CASE", "WHEN", "THEN", "ELSE", "END", "WHILE", "FOR", "LOOP", 
            "REPEAT", "UNTIL", "BREAK", "CONTINUE", "DECLARE", "SET", "EXEC", "CALL",
            "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT", "GRANT", "REVOKE", "DENY",
            "TRIGGER", "PROCEDURE", "FUNCTION", "INDEX", "CONSTRAINT", "PRIMARY", "KEY",
            "FOREIGN", "REFERENCES", "UNIQUE", "CHECK", "DEFAULT", "AUTO_INCREMENT",
            "IDENTITY", "NOT NULL", "NULL", "INTO", "VALUES", "RETURN", "DISTINCT",
            "HAVING", "WINDOW", "CURRENT_DATE", "CURRENT_TIME", "CURRENT_TIMESTAMP",
            "EXTRACT", "DATE_TRUNC", "DATE_PART", "UPPER", "LOWER", "LENGTH", "SUBSTRING",
            "CONCAT", "REPLACE", "TRIM", "LTRIM", "RTRIM", "CAST", "ROUND", "FLOOR", "CEILING",
            "AVG", "COUNT", "MAX", "MIN", "SUM", "STDDEV", "VAR", "ANY", "SOME", "ALL",
            "EXISTS", "UNIQUE", "INTERSECT", "EXCEPT", "UNION", "WITH", "AS", "OVER",
            "PARTITION", "ORDER", "LAG", "LEAD", "FIRST_VALUE", "LAST_VALUE", "ROW_NUMBER",
            "RANK", "DENSE_RANK", "PERCENT_RANK", "CUME_DIST", "NTILE", "SEMI", "ANTI",
            "TABLESAMPLE", "DISTRIBUTE", "SORT", "BUCKETS", "STORED", "ORC", "TEXTFILE",
            "PARQUET", "JSONFILE", "SEQUENCEFILE", "RCFILE", "INPUTFORMAT", "OUTPUTFORMAT"
        ];
        
        // 最终的严格过滤函数
        let filter_identifier = |ident: &str| -> Option<String> {
            // 先清理标识符
            let cleaned = self.clean_identifier(ident);
            
            // 检查是否为空
            if cleaned.is_empty() {
                return None;
            }
            
            // 检查是否为SQL关键字（大小写不敏感）
            let upper_cleaned = cleaned.to_uppercase();
            if sql_keywords.iter().any(|&kw| kw == upper_cleaned) {
                return None;
            }
            
            // 检查是否包含无效字符或格式
            if cleaned.contains('(') || cleaned.contains(')') || cleaned.contains(';') || 
               cleaned.contains(',') || cleaned.contains('+') || cleaned.contains('-') || 
               cleaned.contains('*') || cleaned.contains('/') || cleaned.contains('=') || 
               cleaned.contains('>') || cleaned.contains('<') || cleaned.contains('!') || 
               cleaned.contains('%') || cleaned.contains('&') || cleaned.contains('|') || 
               cleaned.contains('^') || cleaned.contains('~') || cleaned.contains('@') || 
               cleaned.contains('#') || cleaned.contains('$') || cleaned.contains('\'') || 
               cleaned.contains('"') || cleaned.contains('[') || cleaned.contains(']') ||
               cleaned.contains('{') || cleaned.contains('}') || cleaned.contains('`') {
                return None;
            }
            
            // 检查是否只包含数字
            if cleaned.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            
            // 检查是否为纯空格
            if cleaned.trim().is_empty() {
                return None;
            }
            
            Some(cleaned)
        };
        
        // 对表名进行最终严格过滤
        for table in all_tables {
            if let Some(filtered) = filter_identifier(&table) {
                tables.insert(filtered);
            }
        }
        
        // 对模式名进行最终严格过滤
        for schema in all_schemas {
            if let Some(filtered) = filter_identifier(&schema) {
                schemas.insert(filtered);
            }
        }
        
        // 对数据库名进行最终严格过滤
        for database in all_databases {
            if let Some(filtered) = filter_identifier(&database) {
                databases.insert(filtered);
            }
        }
        
        // 对列名进行最终严格过滤
        for column in all_columns {
            if let Some(filtered) = filter_identifier(&column) {
                columns.insert(filtered);
            }
        }
        
        // 过滤对象
        for mut obj in all_objects {
            // 使用新的过滤函数处理表名
            if let Some(filtered_table) = filter_identifier(&obj.table) {
                obj.table = filtered_table;
                
                // 使用新的过滤函数处理数据库名
                if let Some(db) = &mut obj.database {
                    if let Some(filtered_db) = filter_identifier(db) {
                        *db = filtered_db;
                    } else {
                        obj.database = None;
                    }
                }
                
                // 使用新的过滤函数处理模式名
                if let Some(schema) = &mut obj.schema {
                    if let Some(filtered_schema) = filter_identifier(schema) {
                        *schema = filtered_schema;
                    } else {
                        obj.schema = None;
                    }
                }
                
                objects.push(obj);
            }
        }
    }
    
    /// 验证标识符是否有效（非关键字、非纯数字、不含括号、长度>0）
    fn is_valid_identifier(&self, identifier: &str) -> bool {
        // 空字符串直接返回false
        if identifier.is_empty() {
            return false;
        }
        
        // 严格检查是否为SQL关键字（不区分大小写）
        let identifier_upper = identifier.to_uppercase();
        if self.is_sql_keyword(&identifier_upper) {
            return false;
        }
        
        // 检查是否为纯数字
        if identifier.chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // 检查是否包含无效字符
        if identifier.contains(['(', ')', ';', ',', '=', '<', '>', '&', '|', '!', '*', '/', '+', '-', '%', '^']) {
            return false;
        }
        
        // 检查是否完全匹配SQL关键字
        let sql_keywords = [
            "SELECT", "FROM", "WHERE", "JOIN", "GROUP", "BY", "ORDER", "INSERT", 
            "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "IN", "NOT", "BETWEEN", 
            "LIKE", "EXISTS", "INNER", "LEFT", "RIGHT", "FULL", "OUTER", "ON", "AS", 
            "HAVING", "DISTINCT", "ALL", "UNION", "INTERSECT", "EXCEPT", "LIMIT", 
            "OFFSET", "FOR", "WHILE", "CASE", "WHEN", "THEN", "ELSE", "END", 
            "AND", "OR", "IS", "NULL", "TRUE", "FALSE", "DEFAULT", "PRIMARY", "KEY",
            "FOREIGN", "REFERENCES", "INDEX", "VIEW", "TABLE", "TRIGGER",
            "PROCEDURE", "FUNCTION", "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT",
            "SET", "WITH", "OVER", "PARTITION", "SEMI", "ANTI", "CROSS", "NATURAL",
            "USING", "EXPLAIN", "ANALYZE", "TEMP", "TEMPORARY"
        ];
        
        for keyword in sql_keywords {
            if identifier_upper == keyword {
                return false;
            }
        }
        
        true
    }

    /// 从ObjectName提取表名信息（严格过滤版本）
    fn extract_table_name(
        &self,
        name: &ObjectName,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        let mut database = None;
        let mut schema = None;
        let mut table_name = String::new();

        // 解析表名并去除反引号
        if name.0.len() == 1 {
            table_name = name.0[0].value.clone().trim_matches('`').to_string();
        } else if name.0.len() == 2 {
            schema = Some(name.0[0].value.clone().trim_matches('`').to_string());
            table_name = name.0[1].value.clone().trim_matches('`').to_string();
        } else if name.0.len() >= 3 {
            database = Some(name.0[0].value.clone().trim_matches('`').to_string());
            schema = Some(name.0[1].value.clone().trim_matches('`').to_string());
            table_name = name.0[2].value.clone().trim_matches('`').to_string();
        }

        // 严格过滤：只接受有效的表名（非关键字、非纯数字、不含特殊字符）
        if self.is_valid_table_name(&table_name) {
            if let Some(db) = &database {
                if self.is_valid_identifier(db) {
                    databases.insert(db.clone());
                } else {
                    database = None; // 无效的数据库名不添加
                }
            }
            if let Some(sch) = &schema {
                if self.is_valid_identifier(sch) {
                    schemas.insert(sch.clone());
                } else {
                    schema = None; // 无效的schema名不添加
                }
            }
            
            tables.insert(table_name.clone());

            objects.push(SqlObject {
                database: database.clone(),
                schema: schema.clone(),
                table: table_name,
                column: None,
                alias: None,
            });
        }
    }
    
    /// 验证表名是否有效（严格版本）
    fn is_valid_table_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        
        // 检查是否为SQL关键字（不区分大小写）
        let name_upper = name.to_uppercase();
        let sql_keywords = [
            "SELECT", "FROM", "WHERE", "JOIN", "GROUP", "BY", "ORDER", "INSERT", 
            "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "IN", "NOT", "BETWEEN", 
            "LIKE", "EXISTS", "INNER", "LEFT", "RIGHT", "FULL", "OUTER", "ON", "AS", 
            "HAVING", "DISTINCT", "ALL", "UNION", "INTERSECT", "EXCEPT", "LIMIT", 
            "OFFSET", "FOR", "WHILE", "CASE", "WHEN", "THEN", "ELSE", "END", 
            "AND", "OR", "IS", "NULL", "TRUE", "FALSE", "DEFAULT", "PRIMARY", "KEY",
            "FOREIGN", "REFERENCES", "INDEX", "VIEW", "TABLE", "TRIGGER",
            "PROCEDURE", "FUNCTION", "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT",
            "SET", "WITH", "OVER", "PARTITION", "SEMI", "ANTI", "CROSS", "NATURAL",
            "USING", "EXPLAIN", "ANALYZE", "TEMP", "TEMPORARY", "VALUES", "INTO",
            "DISTRIBUTE", "SORT", "BUCKET", "CLUSTER", "STORED", "ORC", "PARQUET"
        ];
        
        if sql_keywords.iter().any(|&kw| kw == name_upper) {
            return false;
        }
        
        // 检查是否为纯数字
        if name.chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // 检查是否包含无效字符
        if name.contains(['(', ')', ';', ',', '=', '<', '>', '&', '|', '!', '*', '/', '+', '-', '%', '^', '.']) {
            return false;
        }
        
        // 检查长度是否合理（1-64个字符）
        if name.len() < 1 || name.len() > 64 {
            return false;
        }
        
        // 检查是否为常见的列名模式（小写字母、下划线分隔）
        if name.chars().all(|c| c.is_lowercase() || c == '_') && name.len() <= 30 {
            return false;
        }
        
        // 检查是否为中文（中文字符通常不是表名）
        if name.chars().any(|c| c >= '\u{4e00}' && c <= '\u{9fff}') {
            return false;
        }
        
        // 检查是否为常见的列名（更严格的过滤）
        let common_column_names = [
            "GENDER", "FIRST_NAME", "LAST_NAME", "CLS_ID", "ID", "NAME", "AGE", "SALARY",
            "ADDRESS", "CITY", "STATE", "ZIP", "PHONE", "EMAIL", "USERNAME", "PASSWORD",
            "DESCRIPTION", "COMMENT", "NOTE", "TITLE", "CAPTION", "LABEL", "TAG",
            "VALUE", "AMOUNT", "PRICE", "COST", "FEE", "RATE", "PERCENT", "QUANTITY",
            "COUNT", "SUM", "AVG", "MIN", "MAX", "TOTAL", "AVERAGE", "NUMBER",
            "DATE", "TIME", "YEAR", "MONTH", "DAY", "HOUR", "MINUTE", "SECOND",
            "STATUS", "TYPE", "CATEGORY", "CLASS", "GROUP", "LEVEL", "RANK", "SCORE",
            "WEIGHT", "HEIGHT", "WIDTH", "LENGTH", "SIZE", "VOLUME", "AREA", "DISTANCE",
            "SPEED", "TEMPERATURE", "PRESSURE", "DENSITY", "MASS", "FORCE", "ENERGY",
            "POWER", "VOLTAGE", "CURRENT", "RESISTANCE", "CAPACITANCE", "INDUCTANCE",
            "FREQUENCY", "WAVELENGTH", "AMPLITUDE", "PHASE", "ANGLE", "RADIUS",
            "DIAMETER", "CIRCUMFERENCE", "PERIMETER", "AREA", "SURFACE", "VOLUME",
            "COL1", "COL2", "COL3", "COL4", "COL5", "COL6", "COL7", "COL8", "COL9", "COL10",
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"
        ];
        
        if common_column_names.iter().any(|&col| col == name_upper) {
            return false;
        }
        
        // 检查是否为单字符（通常是列别名）
        if name.len() == 1 && name.chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }
        
        true
    }
    
    /// 清理标识符，移除特殊字符和无效部分
    fn clean_identifier(&self, identifier: &str) -> String {
        // 移除括号、引号和特殊字符
        let cleaned = identifier
            .replace(['(', ')', '\'', '"', '`'], "")
            .trim()
            .to_string();
        
        // 定义完整的SQL关键字列表
        let sql_keywords = vec![
            "SELECT", "FROM", "WHERE", "JOIN", "GROUP", "BY", "ORDER", "INSERT", 
            "UPDATE", "DELETE", "CREATE", "DROP", "ALTER", "IN", "NOT", "BETWEEN", 
            "LIKE", "EXISTS", "INNER", "LEFT", "RIGHT", "FULL", "OUTER", "ON", "AS", 
            "HAVING", "DISTINCT", "ALL", "UNION", "INTERSECT", "EXCEPT", "LIMIT", 
            "OFFSET", "FOR", "WHILE", "CASE", "WHEN", "THEN", "ELSE", "END", 
            "AND", "OR", "IS", "NULL", "TRUE", "FALSE", "DEFAULT", "PRIMARY", "KEY",
            "FOREIGN", "REFERENCES", "INDEX", "VIEW", "TABLE", "TRIGGER",
            "PROCEDURE", "FUNCTION", "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT",
            "SET", "WITH", "OVER", "PARTITION", "SEMI", "ANTI", "CROSS", "NATURAL",
            "USING", "EXPLAIN", "ANALYZE", "TEMP", "TEMPORARY"
        ];
        
        // 分割标识符为多个部分
        let parts: Vec<_> = cleaned.split(|c: char| {
            c.is_whitespace() || c == '=' || c == ',' || c == ';' || 
            c == '>' || c == '<' || c == '.' || c == ':' || c == '\''
        }).collect();
        
        // 收集非关键字部分
        let mut valid_parts = Vec::new();
        for part in parts {
            let trimmed = part.trim();
            let part_upper = trimmed.to_uppercase();
            
            // 跳过空部分和纯关键字
            if !trimmed.is_empty() && 
               !sql_keywords.iter().any(|&term| part_upper == term) &&
               !trimmed.chars().all(|c| c.is_numeric()) {
                
                // 进一步清理每个部分
                let further_cleaned = trimmed.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_')
                    .collect::<String>();
                
                if !further_cleaned.is_empty() {
                    valid_parts.push(further_cleaned);
                }
            }
        }
        
        // 如果有有效部分，使用它们的组合
        if !valid_parts.is_empty() {
            // 对于多个部分，尝试找到最可能是实际标识符的部分
            // 优先选择非关键字且长度适中的部分
            for part in &valid_parts {
                if self.is_valid_identifier(part) && part.len() > 1 {
                    return part.to_string();
                }
            }
            
            // 如果没有找到完全有效的部分，返回第一个非空部分
            return valid_parts[0].to_string();
        }
        
        // 如果所有部分都被过滤掉，返回空字符串
        "".to_string()
    }

    fn extract_from_query(
        &self,
        query: &sqlparser::ast::Query,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        // 创建临时集合用于收集所有可能的标识符
        let mut all_tables = HashSet::new();
        let mut all_columns = HashSet::new();
        let mut all_databases = HashSet::new();
        let mut all_schemas = HashSet::new();
        let mut all_objects = Vec::new();
        
        // 处理 SELECT 列
        if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
            let mut temp_columns = HashSet::new();
            for select_item in &select.projection {
                self.extract_from_select_item(select_item, &mut temp_columns);
            }
            
            // 收集所有可能的列名
            all_columns.extend(temp_columns);
            
            // 处理 FROM 子句中的表
            for table_with_join in &select.from {
                match &table_with_join.relation {
                    TableFactor::Table { name, .. } => {
                        // 提取表名信息到临时集合
                        let mut temp_databases = HashSet::new();
                        let mut temp_schemas = HashSet::new();
                        let mut temp_tables = HashSet::new();
                        let mut temp_objects = Vec::new();
                        
                        self.extract_table_name(name, &mut temp_databases, &mut temp_schemas, &mut temp_tables, &mut temp_objects);
                        
                        // 收集所有可能的标识符
                        all_databases.extend(temp_databases);
                        all_schemas.extend(temp_schemas);
                        all_tables.extend(temp_tables);
                        all_objects.extend(temp_objects);
                    },
                    _ => {}
                }
            }
        }
        
        // 最终严格过滤所有标识符
        // 过滤表名
        for table in all_tables {
            let cleaned = self.clean_identifier(&table);
            if self.is_valid_identifier(&cleaned) {
                tables.insert(cleaned);
            }
        }
        
        // 过滤列名
        for col in all_columns {
            let cleaned = self.clean_identifier(&col);
            if self.is_valid_identifier(&cleaned) {
                columns.insert(cleaned);
            }
        }
        
        // 过滤数据库名
        for db in all_databases {
            let cleaned = self.clean_identifier(&db);
            if self.is_valid_identifier(&cleaned) {
                databases.insert(cleaned);
            }
        }
        
        // 过滤模式名
        for schema in all_schemas {
            let cleaned = self.clean_identifier(&schema);
            if self.is_valid_identifier(&cleaned) {
                schemas.insert(cleaned);
            }
        }
        
        // 过滤对象
        for mut obj in all_objects {
            // 清理并验证表名
            obj.table = self.clean_identifier(&obj.table);
            if !obj.table.is_empty() && self.is_valid_identifier(&obj.table) {
                // 清理并验证数据库名
                if let Some(db) = &mut obj.database {
                    *db = self.clean_identifier(db);
                    if !self.is_valid_identifier(db) {
                        obj.database = None;
                    }
                }
                
                // 清理并验证模式名
                if let Some(schema) = &mut obj.schema {
                    *schema = self.clean_identifier(schema);
                    if !self.is_valid_identifier(schema) {
                        obj.schema = None;
                    }
                }
                
                objects.push(obj);
            }
        }
    }

    fn extract_from_table_factor(
        &self,
        table_factor: &TableFactor,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        match table_factor {
            TableFactor::Table { name, alias, .. } => {
                let mut database = None;
                let mut schema = None;
                let mut table_name = String::new();

                if name.0.len() == 1 {
                    table_name = name.0[0].value.clone();
                } else if name.0.len() == 2 {
                    schema = Some(name.0[0].value.clone());
                    table_name = name.0[1].value.clone();
                } else if name.0.len() == 3 {
                    database = Some(name.0[0].value.clone());
                    schema = Some(name.0[1].value.clone());
                    table_name = name.0[2].value.clone();
                }

                // 严格过滤：只接受合法的表名（非关键字、非纯数字、不含括号）
                let is_valid_table = !self.is_sql_keyword(&table_name) && 
                                    !table_name.chars().all(|c| c.is_numeric()) && 
                                    !table_name.contains('(') && 
                                    !table_name.contains(')') && 
                                    table_name.len() > 0;
                
                if is_valid_table {
                    tables.insert(table_name.clone());

                    // 严格过滤schema名
                    if let Some(sch) = &schema {
                        let is_valid_schema = !self.is_sql_keyword(sch) && 
                                            !sch.chars().all(|c| c.is_numeric()) && 
                                            !sch.contains('(') && 
                                            !sch.contains(')') && 
                                            sch.len() > 0;
                        if is_valid_schema {
                            schemas.insert(sch.clone());
                        }
                    }

                    // 严格过滤数据库名
                    if let Some(db) = &database {
                        let is_valid_db = !self.is_sql_keyword(db) && 
                                        !db.chars().all(|c| c.is_numeric()) && 
                                        !db.contains('(') && 
                                        !db.contains(')') && 
                                        db.len() > 0;
                        if is_valid_db {
                            databases.insert(db.clone());
                        }
                    }

                    objects.push(SqlObject {
                        database: database.clone(),
                        schema: schema.clone(),
                        table: table_name,
                        column: None,
                        alias: alias.as_ref().map(|a| a.name.value.clone()),
                    });
                }
            }
            TableFactor::Derived { .. } => {}
            TableFactor::NestedJoin { .. } => {}
            _ => {}
        }
    }

    fn extract_from_select_item(&self, select_item: &sqlparser::ast::SelectItem, columns: &mut HashSet<String>) {
        match select_item {
            sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                // 处理未命名的表达式，使用临时集合过滤
                let mut temp_columns = HashSet::new();
                self.extract_columns_from_expr(expr, &mut temp_columns);
                
                // 严格过滤后再添加到结果集合
                for col in temp_columns {
                    if self.is_valid_identifier(&col) {
                        columns.insert(col);
                    }
                }
            }
            sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } => {
                // 处理带别名的表达式，先处理表达式
                let mut temp_columns = HashSet::new();
                self.extract_columns_from_expr(expr, &mut temp_columns);
                
                // 严格过滤表达式中的列名
                for col in temp_columns {
                    if self.is_valid_identifier(&col) {
                        columns.insert(col);
                    }
                }
                
                // 也提取别名作为列名，确保严格过滤
                let cleaned_alias = self.clean_identifier(&alias.value);
                if self.is_valid_identifier(&cleaned_alias) {
                    columns.insert(cleaned_alias);
                }
            }
            sqlparser::ast::SelectItem::QualifiedWildcard(..) => {}
            sqlparser::ast::SelectItem::Wildcard(_) => {}
        }
    }

    fn extract_columns_from_expr(&self, expr: &sqlparser::ast::Expr, columns: &mut HashSet<String>) {
        match expr {
            sqlparser::ast::Expr::Identifier(ident) => {
                // 清理并严格过滤列名
                let cleaned_value = self.clean_identifier(&ident.value);
                if self.is_valid_identifier(&cleaned_value) {
                    columns.insert(cleaned_value);
                }
            }
            sqlparser::ast::Expr::CompoundIdentifier(idents) => {
                if let Some(last_ident) = idents.last() {
                    // 提取最后一部分作为列名并清理
                    let cleaned_value = self.clean_identifier(&last_ident.value);
                    if self.is_valid_identifier(&cleaned_value) {
                        columns.insert(cleaned_value);
                    }
                }
            }
            sqlparser::ast::Expr::Function(function) => {
                for arg in &function.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(expr) = arg {
                        if let sqlparser::ast::FunctionArgExpr::Expr(expr) = expr {
                            self.extract_columns_from_expr(expr, columns);
                        }
                    }
                }
            }
            sqlparser::ast::Expr::BinaryOp { left, right, .. } => {
                // 递归处理左右表达式
                self.extract_columns_from_expr(left, columns);
                self.extract_columns_from_expr(right, columns);
            }
            // 添加更多表达式类型的处理
            sqlparser::ast::Expr::Subquery(_subquery) => {
                // 由于无法直接访问子查询内容，我们暂时跳过子查询的处理
                // 这避免了编译错误
            }
            sqlparser::ast::Expr::InList { expr, list, .. } => {
                self.extract_columns_from_expr(expr, columns);
                for item in list {
                    self.extract_columns_from_expr(item, columns);
                }
            }
            sqlparser::ast::Expr::Between { expr, low, high, .. } => {
                self.extract_columns_from_expr(expr, columns);
                self.extract_columns_from_expr(low, columns);
                self.extract_columns_from_expr(high, columns);
            }
            _ => {}
        }
    }

    fn remove_comments(&self, sql: &str) -> String {
        let re_single_line = regex::Regex::new(r"--.*$").unwrap();
        let re_multi_line = regex::Regex::new(r"/\*.*?\*/").unwrap();
        
        let no_single_line = re_single_line.replace_all(sql, "");
        let no_multi_line = re_multi_line.replace_all(&no_single_line, "");
        
        no_multi_line.to_string()
    }

    fn normalize_whitespace(&self, sql: &str) -> String {
        let re_whitespace = regex::Regex::new(r"\s+").unwrap();
        re_whitespace.replace_all(sql.trim(), " ").to_string()
    }

    fn normalize_case(&self, sql: &str) -> String {
        // 将 SQL 关键字转换为大写
        let keywords = vec![
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP",
            "ALTER", "TABLE", "INDEX", "VIEW", "JOIN", "INNER", "OUTER", "LEFT", "RIGHT",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "UNION", "AND",
            "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL", "COUNT", "SUM",
            "AVG", "MIN", "MAX", "DISTINCT", "AS", "CASE", "WHEN", "THEN", "ELSE", "END",
        ];
        
        let mut normalized = sql.to_string();
        for keyword in keywords {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", keyword)).unwrap();
            normalized = re.replace_all(&normalized, keyword).to_string();
        }
        
        normalized
    }

    fn normalize_quotes(&self, sql: &str) -> String {
        // 标准化引号，将不同类型的引号统一为双引号
        let re_single_quotes = regex::Regex::new(r"'([^']*)'").unwrap();
        let re_backticks = regex::Regex::new(r"`([^`]*)`").unwrap();
        
        let step1 = re_single_quotes.replace_all(sql, "\"$1\"");
        let step2 = re_backticks.replace_all(&step1, "\"$1\"");
        
        step2.to_string()
    }
}

// 自定义方言已移至单独的dialects模块中实现