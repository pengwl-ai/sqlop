// 数据库特定语法处理器
// 为不同的数据库类型提供特定的语法处理逻辑

use super::types::DatabaseType;
use regex::Regex;

/// 数据库特定语法处理器
pub struct DatabaseSpecificHandler;

impl DatabaseSpecificHandler {
    /// 根据数据库类型处理特定的SQL语法
    pub fn process_database_specific_syntax(sql: &str, db_type: &DatabaseType) -> String {
        let mut processed_sql = sql.to_string();

        match db_type {
            DatabaseType::MySQL => {
                processed_sql = Self::process_mysql_syntax(&processed_sql);
            },
            DatabaseType::PostgreSQL => {
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::SQLServer => {
                processed_sql = Self::process_sqlserver_syntax(&processed_sql);
            },
            DatabaseType::Oracle => {
                processed_sql = Self::process_oracle_syntax(&processed_sql);
            },
            DatabaseType::Hive => {
                processed_sql = Self::process_hive_syntax(&processed_sql);
            },
            DatabaseType::GaussDB => {
                // GaussDB 与 PostgreSQL 语法相似
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::Kingbase => {
                // Kingbase 与 PostgreSQL 语法相似
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::Highgo => {
                // Highgo 与 PostgreSQL 语法相似
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::Greenplum => {
                // Greenplum 与 PostgreSQL 语法相似
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::Vastbase => {
                // Vastbase 与 PostgreSQL 语法相似
                processed_sql = Self::process_postgresql_syntax(&processed_sql);
            },
            DatabaseType::Sybase => {
                // Sybase 与 SQL Server 语法相似
                processed_sql = Self::process_sqlserver_syntax(&processed_sql);
            },
            DatabaseType::DB2 => {
                processed_sql = Self::process_db2_syntax(&processed_sql);
            },
            DatabaseType::Dameng => {
                // Dameng 与 Oracle 语法有相似之处
                processed_sql = Self::process_dameng_syntax(&processed_sql);
            },
            DatabaseType::SQLite => {
                processed_sql = Self::process_sqlite_syntax(&processed_sql);
            },
        }

        processed_sql
    }

    /// 处理 MySQL 特有的语法
    fn process_mysql_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 MySQL 特有的注释语法
        let comment_re = Regex::new(r#"/\*![0-9]+ (.*?)\*/"#).unwrap();
        processed = comment_re.replace_all(&processed, "$1").to_string();

        // 处理 MySQL 特有的函数
        let mysql_functions = vec![
            ("NOW()", "CURRENT_TIMESTAMP"),
            ("CURDATE()", "CURRENT_DATE"),
            ("CURTIME()", "CURRENT_TIME"),
            ("UNIX_TIMESTAMP()", "EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)"),
            ("FROM_UNIXTIME", "TO_TIMESTAMP"),
            ("DATE_FORMAT", "TO_CHAR"),
            ("IF", "CASE"),
            ("CONCAT", "||"),
            ("LIMIT", "FETCH FIRST ROWS ONLY"),
        ];

        for (mysql_func, standard_func) in &mysql_functions {
            processed = processed.replace(mysql_func, standard_func);
        }

        // 处理 LIMIT 语法
        let limit_re = Regex::new(r#"LIMIT\s+(\d+)(?:\s*,\s*(\d+))?"#).unwrap();
        processed = limit_re.replace_all(&processed, |caps: &regex::Captures| {
            if let Some(second_cap) = caps.get(2) {
                format!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", second_cap.as_str(), caps.get(1).unwrap().as_str())
            } else {
                format!("FETCH FIRST {} ROWS ONLY", caps.get(1).unwrap().as_str())
            }
        }).to_string();

        processed
    }

    /// 处理 PostgreSQL 特有的语法
    fn process_postgresql_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 PostgreSQL 特有的语法扩展
        let pg_syntax_extensions = vec![
            ("||", "CONCAT"),  // PostgreSQL 使用 || 作为字符串连接
            ("::", "CAST( AS )"),  // PostgreSQL 的类型转换语法
        ];

        for (pg_syntax, standard_syntax) in &pg_syntax_extensions {
            processed = processed.replace(pg_syntax, standard_syntax);
        }

        // 处理 PostgreSQL 的 $ 占位符
        let placeholder_re = Regex::new(r#"\$\d+"#).unwrap();
        processed = placeholder_re.replace_all(&processed, "?").to_string();

        processed
    }

    /// 处理 SQL Server 特有的语法
    fn process_sqlserver_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 SQL Server 特有的函数和语法
        let sqlserver_functions = vec![
            ("GETDATE()", "CURRENT_TIMESTAMP"),
            ("TOP", "FETCH FIRST ROWS ONLY"),
            ("IDENTITY", "AUTO_INCREMENT"),
            ("nvarchar", "varchar"),
        ];

        for (mssql_func, standard_func) in &sqlserver_functions {
            processed = processed.replace(mssql_func, standard_func);
        }

        // 处理 TOP N 语法
        let top_re = Regex::new(r#"TOP\s+(\d+)"#).unwrap();
        processed = top_re.replace_all(&processed, "FETCH FIRST $1 ROWS ONLY").to_string();

        processed
    }

    /// 处理 Oracle 特有的语法
    fn process_oracle_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 Oracle 特有的函数和语法
        let oracle_functions = vec![
            ("SYSDATE", "CURRENT_TIMESTAMP"),
            ("TO_DATE", "CAST( AS DATE)"),
            ("NVL", "COALESCE"),
            ("ROWNUM <=", "FETCH FIRST ROWS ONLY"),
        ];

        for (oracle_func, standard_func) in &oracle_functions {
            processed = processed.replace(oracle_func, standard_func);
        }

        // 处理 ROWNUM 分页语法
        let rownum_re = Regex::new(r#"ROWNUM\s*<=\s*(\d+)"#).unwrap();
        processed = rownum_re.replace_all(&processed, "FETCH FIRST $1 ROWS ONLY").to_string();

        processed
    }

    /// 处理 Hive 特有的语法
    fn process_hive_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 Hive 特有的函数和语法
        let hive_functions = vec![
            ("unix_timestamp()", "EXTRACT(EPOCH FROM CURRENT_TIMESTAMP)"),
            ("from_unixtime", "TO_TIMESTAMP"),
            ("regexp_replace", "REGEXP_REPLACE"),
            ("concat_ws", "CONCAT_WS"),
        ];

        for (hive_func, standard_func) in &hive_functions {
            processed = processed.replace(hive_func, standard_func);
        }

        // 处理 Hive 的 lateral view 语法
        let lateral_view_re = Regex::new(r#"LATERAL VIEW"#).unwrap();
        processed = lateral_view_re.replace_all(&processed, "CROSS JOIN LATERAL").to_string();

        processed
    }

    /// 处理 DB2 特有的语法
    fn process_db2_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 DB2 特有的函数和语法
        let db2_functions = vec![
            ("CURRENT TIMESTAMP", "CURRENT_TIMESTAMP"),
            ("CURRENT DATE", "CURRENT_DATE"),
            ("VARCHAR_FORMAT", "TO_CHAR"),
        ];

        for (db2_func, standard_func) in &db2_functions {
            processed = processed.replace(db2_func, standard_func);
        }

        processed
    }

    /// 处理 Dameng 特有的语法
    fn process_dameng_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 Dameng 特有的函数和语法
        let dameng_functions = vec![
            ("SYSDATE", "CURRENT_TIMESTAMP"),
            ("TO_DATE", "CAST( AS DATE)"),
            ("NVL", "COALESCE"),
        ];

        for (dameng_func, standard_func) in &dameng_functions {
            processed = processed.replace(dameng_func, standard_func);
        }

        processed
    }

    /// 处理 SQLite 特有的语法
    fn process_sqlite_syntax(sql: &str) -> String {
        let mut processed = sql.to_string();

        // 处理 SQLite 特有的函数和语法
        let sqlite_functions = vec![
            ("DATETIME()", "CURRENT_TIMESTAMP"),
            ("DATE()", "CURRENT_DATE"),
            ("TIME()", "CURRENT_TIME"),
            ("INSTR", "POSITION"),
            ("LIMIT", "FETCH FIRST ROWS ONLY"),
        ];

        for (sqlite_func, standard_func) in &sqlite_functions {
            processed = processed.replace(sqlite_func, standard_func);
        }

        // 处理 SQLite 的 LIMIT 语法
        let limit_re = Regex::new(r#"LIMIT\s+(\d+)(?:\s*,\s*(\d+))?"#).unwrap();
        processed = limit_re.replace_all(&processed, |caps: &regex::Captures| {
            if let Some(second_cap) = caps.get(2) {
                format!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", second_cap.as_str(), caps.get(1).unwrap().as_str())
            } else {
                format!("FETCH FIRST {} ROWS ONLY", caps.get(1).unwrap().as_str())
            }
        }).to_string();

        processed
    }

    /// 提取 SQL 中的表名（增强版）
    pub fn extract_table_names_enhanced(sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        let sql_lower = sql.to_lowercase();

        // 使用正则表达式提取 FROM, JOIN, INTO 等子句中的表名（支持带引号和反引号的标识符）
        let table_regexes = vec![
            Regex::new(r#"from\s+(?:`([^`]+)`|"([^"]+)"|([a-zA-Z0-9_.]+))"#).unwrap(),
            Regex::new(r#"join\s+(?:`([^`]+)`|"([^"]+)"|([a-zA-Z0-9_.]+))"#).unwrap(),
            Regex::new(r#"into\s+(?:`([^`]+)`|"([^"]+)"|([a-zA-Z0-9_.]+))"#).unwrap(),
            Regex::new(r#"update\s+(?:`([^`]+)`|"([^"]+)"|([a-zA-Z0-9_.]+))"#).unwrap(),
            Regex::new(r#"delete\s+from\s+(?:`([^`]+)`|"([^"]+)"|([a-zA-Z0-9_.]+))"#).unwrap(),
        ];

        for regex in table_regexes {
            for captures in regex.captures_iter(&sql_lower) {
                // 尝试从不同的捕获组获取表名
                let table_name = if let Some(m) = captures.get(1) {
                    m.as_str() // 带反引号的表名
                } else if let Some(m) = captures.get(2) {
                    m.as_str() // 带引号的表名
                } else if let Some(m) = captures.get(3) {
                    m.as_str() // 普通表名
                } else {
                    continue;
                };
                
                tables.push(table_name.to_string());
            }
        }

        tables
    }

    /// 提取 SQL 中的列名（增强版）
    pub fn extract_column_names_enhanced(sql: &str) -> Vec<String> {
        let mut columns = Vec::new();
        let sql_lower = sql.to_lowercase();

        // 查找 SELECT 语句中的列名
        if let Some(select_start) = sql_lower.find("select") {
            if let Some(from_start) = sql_lower[select_start..].find(" from ") {
                let select_part = &sql[select_start + 6..select_start + from_start];
                
                // 分割列名
                let column_parts: Vec<&str> = select_part.split(',').collect();
                
                for part in column_parts {
                    let trimmed_part = part.trim();
                    // 跳过通配符
                    if trimmed_part == "*" {
                        continue;
                    }
                    
                    // 处理 AS 别名
                    if let Some(as_pos) = trimmed_part.to_lowercase().find(" as ") {
                        let column_part = &trimmed_part[..as_pos].trim();
                        columns.push(Self::extract_last_identifier(column_part));
                    } else {
                        // 提取最后一个标识符作为列名
                        columns.push(Self::extract_last_identifier(trimmed_part));
                    }
                }
            }
        }

        // 从 WHERE 子句中提取列名
        if let Some(where_start) = sql_lower.find(" where ") {
            let where_part = &sql[where_start + 7..];
            columns.extend(Self::extract_identifiers_from_expression(where_part));
        }

        // 从 GROUP BY 子句中提取列名
        if let Some(group_start) = sql_lower.find(" group by ") {
            let group_part = &sql[group_start + 9..];
            columns.extend(Self::extract_identifiers_from_expression(group_part));
        }

        // 从 ORDER BY 子句中提取列名
        if let Some(order_start) = sql_lower.find(" order by ") {
            let order_part = &sql[order_start + 9..];
            columns.extend(Self::extract_identifiers_from_expression(order_part));
        }

        columns
    }

    // 辅助方法：从表达式中提取标识符
    fn extract_identifiers_from_expression(expr: &str) -> Vec<String> {
        let mut identifiers = Vec::new();
        
        // 使用正则表达式提取可能的列名（包括带反引号和引号的）
        let identifier_re = Regex::new(r#"(?:`([^`]+)`|"([^"]+)"|\b([a-zA-Z0-9_]+)\b)"#).unwrap();
        
        for captures in identifier_re.captures_iter(expr) {
            let identifier = if let Some(m) = captures.get(1) {
                m.as_str() // 带反引号的标识符
            } else if let Some(m) = captures.get(2) {
                m.as_str() // 带引号的标识符
            } else if let Some(m) = captures.get(3) {
                m.as_str() // 普通标识符
            } else {
                continue;
            };
            
            // 过滤掉常见的SQL关键字
            if !Self::is_sql_keyword(identifier) {
                identifiers.push(identifier.to_string());
            }
        }
        
        identifiers
    }

    // 辅助方法：提取最后一个标识符
    fn extract_last_identifier(expr: &str) -> String {
        // 处理可能的表别名.列名格式
        if let Some(dot_pos) = expr.rfind('.') {
            let last_part = &expr[dot_pos + 1..];
            // 去除可能的括号和引号
            let clean_part = last_part.trim_matches(|c| c == '(' || c == ')' || c == '`' || c == '"').to_string();
            return clean_part;
        }
        
        // 去除可能的括号和引号
        let clean_expr = expr.trim_matches(|c| c == '(' || c == ')' || c == '`' || c == '"').to_string();
        clean_expr
    }

    // 辅助方法：检查是否是SQL关键字
    fn is_sql_keyword(word: &str) -> bool {
        let keywords = [
            "select", "from", "where", "and", "or", "group", "by", "order", "having",
            "join", "inner", "outer", "left", "right", "full", "on", "as", "distinct",
            "limit", "offset", "fetch", "rows", "only", "first", "next", "current",
            "timestamp", "date", "time", "like", "in", "exists", "not", "between",
            "null", "is", "true", "false", "case", "when", "then", "else", "end",
            "insert", "update", "delete", "create", "drop", "alter", "truncate",
            "into", "values", "set", "table", "database", "schema", "index", "view"
        ];
        
        keywords.contains(&word.to_lowercase().as_str())
    }
}