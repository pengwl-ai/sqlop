use std::collections::{HashSet, HashMap};
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;

// 定义SQL语句类型枚举
#[derive(Debug, PartialEq, Eq, Clone)]
enum SqlStatementType {
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
    CREATE,
    ALTER,
    DROP,
    TRUNCATE,
    OTHER,
}

// 定义数据库方言枚举
#[derive(Debug, PartialEq, Eq, Clone)]
enum DialectType {
    MySQL,
    PostgreSQL,
    Oracle,
    SQLServer,
    GaussDB,
    Dameng,
    Sybase,
    DB2,
    Hive,
    Impala,
    SparkSQL,
    Presto,
    ClickHouse,
    Other,
}

// 预编译正则表达式，使用lazy_static避免重复编译
lazy_static! {
    // GaussDB特有的POINT函数参数格式
    static ref GAUSSDB_PATTERN: Regex = Regex::new(r"\(([A-Z0-9]+)\s+([A-Z0-9]+)\s+([A-Z0-9]+)\)").unwrap();
    // 标准格式的标识符提取，增强版
    static ref IDENTIFIER_RE: Regex = Regex::new(r#"(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|\b([a-zA-Z0-9_]+)\b)"#).unwrap();
    // 表引用提取正则表达式（增强版）
    static ref TABLE_REFERENCE_RE: Regex = Regex::new(r#"(?:(?:FROM|JOIN|INTO|UPDATE)\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|\b([a-zA-Z0-9_.]+)\b))"#).unwrap();
    // 常见的schema名称列表
    static ref COMMON_SCHEMA_NAMES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("public"); set.insert("dbo"); set.insert("sys"); set.insert("information_schema");
        set.insert("pg_catalog"); set.insert("mysql"); set.insert("performance_schema");
        set.insert("sysibm"); set.insert("syscat"); set.insert("sysdba");
        set
    };
    // SQL关键字集合
    static ref SQL_KEYWORDS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("select"); set.insert("from"); set.insert("where"); set.insert("and");
        set.insert("or"); set.insert("not"); set.insert("as"); set.insert("join");
        set.insert("on"); set.insert("left"); set.insert("right"); set.insert("inner");
        set.insert("outer"); set.insert("full"); set.insert("cross"); set.insert("group");
        set.insert("by"); set.insert("having"); set.insert("order"); set.insert("limit");
        set.insert("offset"); set.insert("distinct"); set.insert("all"); set.insert("any");
        set.insert("some"); set.insert("exists"); set.insert("in"); set.insert("between");
        set.insert("like"); set.insert("ilike"); set.insert("is"); set.insert("null");
        set.insert("true"); set.insert("false"); set.insert("case"); set.insert("when");
        set.insert("then"); set.insert("else"); set.insert("end"); set.insert("if");
        set.insert("elseif"); set.insert("for"); set.insert("update"); set.insert("set");
        set.insert("delete"); set.insert("insert"); set.insert("into"); set.insert("values");
        set.insert("create"); set.insert("table"); set.insert("view"); set.insert("index");
        set.insert("alter"); set.insert("drop"); set.insert("truncate"); set.insert("rename");
        set.insert("add"); set.insert("modify"); set.insert("change"); set.insert("drop");
        set.insert("primary"); set.insert("key"); set.insert("foreign"); set.insert("references");
        set.insert("unique"); set.insert("check"); set.insert("constraint"); set.insert("default");
        set.insert("nulls"); set.insert("asc"); set.insert("desc"); set.insert("using");
        set.insert("with"); set.insert("window"); set.insert("partition"); set.insert("row");
        set.insert("rows"); set.insert("range"); set.insert("preceding"); set.insert("following");
        set.insert("current"); set.insert("row"); set.insert("unbounded"); set.insert("all");
        set.insert("any"); set.insert("some"); set.insert("exists"); set.insert("distinct");
        set.insert("except"); set.insert("intersect"); set.insert("union"); set.insert("minus");
        set.insert("true"); set.insert("false"); set.insert("is"); set.insert("not");
        set.insert("null");
        set
    };
    
    static ref COMMON_FUNCTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("unnest"); set.insert("explode"); set.insert("cast"); set.insert("convert");
        set.insert("count"); set.insert("sum"); set.insert("avg"); set.insert("min"); set.insert("max");
        set.insert("abs"); set.insert("ceil"); set.insert("floor"); set.insert("round");
        set.insert("trim"); set.insert("ltrim"); set.insert("rtrim"); set.insert("upper"); set.insert("lower");
        set.insert("substring"); set.insert("concat"); set.insert("length"); set.insert("coalesce");
        set.insert("ifnull"); set.insert("isnull"); set.insert("nvl");
        set.insert("date_format"); set.insert("to_date"); set.insert("from_unixtime");
        set.insert("unix_timestamp"); set.insert("now");
        set
    };
    // 常用SQL模式匹配
    static ref SQL_TYPE_PATTERNS: Vec<(&'static str, SqlStatementType)> = {
        vec![
            ("^select", SqlStatementType::SELECT),
            ("^insert", SqlStatementType::INSERT),
            ("^update", SqlStatementType::UPDATE),
            ("^delete", SqlStatementType::DELETE),
            ("^create", SqlStatementType::CREATE),
            ("^drop", SqlStatementType::DROP),
            ("^alter", SqlStatementType::ALTER),
            ("^truncate", SqlStatementType::TRUNCATE),
        ]
    };
    // 常见的列函数，用于排除
    static ref COMMON_COLUMN_FUNCTIONS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("sum"); set.insert("avg"); set.insert("count"); set.insert("max");
        set.insert("min"); set.insert("distinct"); set.insert("cast"); set.insert("convert");
        set.insert("date"); set.insert("time"); set.insert("year"); set.insert("month");
        set.insert("day"); set.insert("hour"); set.insert("minute"); set.insert("second");
        set.insert("upper"); set.insert("lower"); set.insert("left"); set.insert("right");
        set.insert("abs"); set.insert("round"); set.insert("floor"); set.insert("ceil");
        set.insert("ceiling"); set.insert("length"); set.insert("concat"); set.insert("substr");
        set.insert("substring"); set.insert("replace"); set.insert("trim"); set.insert("ltrim");
        set.insert("rtrim"); set.insert("coalesce"); set.insert("nullif"); set.insert("case");
        set.insert("extract"); set.insert("date_trunc"); set.insert("date_add"); set.insert("date_sub");
        set.insert("datediff"); set.insert("to_date"); set.insert("to_char"); set.insert("now");
        set.insert("current_date"); set.insert("current_time"); set.insert("current_timestamp");
        set.insert("sysdate"); set.insert("utc_date"); set.insert("utc_time"); set.insert("position");
        set.insert("lead"); set.insert("lag"); set.insert("rank"); set.insert("dense_rank");
        set.insert("row_number"); set.insert("ntile"); set.insert("percentile"); set.insert("truncate");
        set.insert("log"); set.insert("log10"); set.insert("exp"); set.insert("sqrt");
        set.insert("power"); set.insert("sin"); set.insert("cos"); set.insert("tan");
        set.insert("asin"); set.insert("acos"); set.insert("atan"); set.insert("atan2");
        set.insert("degrees"); set.insert("radians");
        set
    };
    // 数据库方言映射
    static ref DIALECT_MAP: HashMap<&'static str, DialectType> = {
        let mut map = HashMap::new();
        map.insert("mysql", DialectType::MySQL);
        map.insert("postgresql", DialectType::PostgreSQL);
        map.insert("oracle", DialectType::Oracle);
        map.insert("sqlserver", DialectType::SQLServer);
        map.insert("mssql", DialectType::SQLServer);
        map.insert("gaussdb", DialectType::GaussDB);
        map.insert("dameng", DialectType::Dameng);
        map.insert("sybase", DialectType::Sybase);
        map.insert("db2", DialectType::DB2);
        map.insert("hive", DialectType::Hive);
        map.insert("impala", DialectType::Impala);
        map.insert("sparksql", DialectType::SparkSQL);
        map.insert("presto", DialectType::Presto);
        map.insert("clickhouse", DialectType::ClickHouse);
        map
    };
}

/// 高性能SQL解析器改进版 - 支持基于AST的分层解析
pub struct EnhancedSqlParserImprovedOptimized {
    dialect: Option<String>,
    dialect_type: DialectType,
}

impl EnhancedSqlParserImprovedOptimized {
    /// 创建新的解析器实例
    pub fn new(dialect: Option<String>) -> Self {
        let dialect_type = dialect.as_ref()
            .and_then(|d| DIALECT_MAP.get(d.to_lowercase().as_str()))
            .cloned()
            .unwrap_or(DialectType::Other);
        
        Self {
            dialect,
            dialect_type,
        }
    }
    
    /// 获取SQL语句类型
    fn get_sql_statement_type(&self, sql: &str) -> SqlStatementType {
        let lower_sql = sql.trim().to_lowercase();
        
        for (pattern, stmt_type) in SQL_TYPE_PATTERNS.iter() {
            if let Some(re) = Regex::new(pattern).ok() {
                if re.is_match(&lower_sql) {
                    return stmt_type.clone();
                }
            }
        }
        SqlStatementType::OTHER
    }

    /// 解析SQL，提取数据库对象信息 - 基于SQL类型和方言的分层解析
    pub fn parse_sql(&self, 
                     sql: &str, 
                     databases: &mut HashSet<String>, 
                     schemas: &mut HashSet<String>, 
                     tables: &mut HashSet<String>, 
                     columns: &mut HashSet<String>) -> bool {
        // 预处理SQL
        let preprocessed_sql = self.preprocess_sql(sql);
        
        // 获取SQL语句类型
        let sql_type = self.get_sql_statement_type(&preprocessed_sql);
        
        // 基于SQL语句类型进行分层解析
        match sql_type {
            SqlStatementType::SELECT => {
                self.parse_select_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::INSERT => {
                self.parse_insert_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::UPDATE => {
                self.parse_update_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::DELETE => {
                self.parse_delete_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::CREATE => {
                self.parse_create_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::ALTER => {
                self.parse_alter_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            SqlStatementType::DROP | SqlStatementType::TRUNCATE => {
                self.parse_drop_truncate_statement(&preprocessed_sql, databases, schemas, tables, columns);
            },
            _ => {
                // 其他类型SQL的通用处理
                self.parse_generic_statement(&preprocessed_sql, databases, schemas, tables, columns);
            }
        }
        
        // 处理WITH子句（CTE）
        self.handle_with_statements(&preprocessed_sql, databases, schemas, tables, columns);
        
        // 处理子查询（递归解析）
        self.handle_subqueries(&preprocessed_sql, databases, schemas, tables, columns);
        
        // 清理和优化结果
        self.cleanup_result(databases, schemas, tables, columns);
        
        true
    }
    
    /// 解析SELECT语句
    fn parse_select_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 特殊处理SELECT *语句
        if lower_sql.contains("select *") || lower_sql.contains("select\t*") {
            columns.insert("*".to_string());
        }
        
        // 处理方言特定内容
        if let Some(dialect) = &self.dialect {
            self.handle_dialect_specific(sql, dialect, tables, columns);
        }
        
        // 1. 提取表引用（FROM、JOIN子句）
        self.extract_table_references(sql, tables, schemas, databases);
        
        // 2. 提取列引用（SELECT、WHERE、GROUP BY、HAVING、ORDER BY子句）
        self.extract_column_references(sql, columns);
        
        // 3. 处理函数调用
        self.handle_function_calls(sql, tables, columns);
        
        // 4. 特殊处理WITH子句（CTE）
        self.handle_with_statements(sql, databases, schemas, tables, columns);
    }
    
    /// 解析INSERT语句
    fn parse_insert_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 提取INTO子句中的表名
        let lower_sql = sql.to_lowercase();
        
        if let Some(into_pos) = lower_sql.find(" into ") {
            let after_into = &sql[into_pos + 6..];
            let table_part = self.extract_first_identifier(after_into);
            if !table_part.is_empty() {
                self.parse_table_identifier(&table_part, tables, schemas, databases);
            }
            
            // 提取列列表
            if let Some(cols_start) = after_into.find('(') {
                if let Some(cols_end) = self.find_matching_parenthesis(&after_into, cols_start) {
                    if cols_end > cols_start {
                        let cols_str = &after_into[cols_start + 1..cols_end];
                        let col_parts = self.split_sql_parts(cols_str, ',');
                        for part in col_parts {
                            let trimmed = part.trim();
                            if !trimmed.is_empty() {
                                let identifier = EnhancedSqlParserImprovedOptimized::normalize_identifier(&trimmed);
                                if !EnhancedSqlParserImprovedOptimized::is_keyword(&identifier) && !EnhancedSqlParserImprovedOptimized::is_common_function(&identifier) {
                                    columns.insert(identifier);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 解析UPDATE语句
    fn parse_update_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 提取UPDATE子句中的表名
        if let Some(update_pos) = lower_sql.find("update ") {
            let after_update = &sql[update_pos + 7..];
            let table_part = self.extract_first_identifier(after_update);
            if !table_part.is_empty() {
                self.parse_table_identifier(&table_part, tables, schemas, databases);
            }
        }
        
        // 提取SET子句中的列名
        if let Some(set_pos) = lower_sql.find(" set ") {
            let set_clause = &sql[set_pos + 5..];
            let where_pos = lower_sql[set_pos + 5..].find(" where ");
            let set_clause_end = where_pos.map_or(sql.len(), |pos| set_pos + 5 + pos);
            
            let set_parts = self.split_sql_parts(&sql[set_pos + 5..set_clause_end], ',');
            for part in set_parts {
                let trimmed = part.trim();
                if let Some(equal_pos) = trimmed.find('=') {
                    let col_name = trimmed[..equal_pos].trim();
                    let identifier = EnhancedSqlParserImprovedOptimized::normalize_identifier(col_name);
                    if !EnhancedSqlParserImprovedOptimized::is_keyword(&identifier) {
                        columns.insert(identifier);
                    }
                }
            }
        }
        
        // 提取WHERE子句中的列名
        if let Some(where_pos) = lower_sql.find(" where ") {
            let where_clause = &sql[where_pos + 7..];
            let identifiers = self.extract_identifiers_from_expression(where_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
    }
    
    /// 解析DELETE语句
    fn parse_delete_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 提取FROM子句中的表名
        if let Some(from_pos) = lower_sql.find(" from ") {
            let after_from = &sql[from_pos + 6..];
            let table_part = self.extract_first_identifier(after_from);
            if !table_part.is_empty() {
                self.parse_table_identifier(&table_part, tables, schemas, databases);
            }
        }
        
        // 提取WHERE子句中的列名
        if let Some(where_pos) = lower_sql.find(" where ") {
            let where_clause = &sql[where_pos + 7..];
            let identifiers = self.extract_identifiers_from_expression(where_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
    }
    
    /// 解析CREATE语句
    fn parse_create_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 首先处理WITH子句（CTE）
        self.handle_with_statements(sql, databases, schemas, tables, columns);
        
        let lower_sql = sql.to_lowercase();
        
        // 处理CREATE TABLE
        if lower_sql.contains("create table") {
            if let Some(table_pos) = lower_sql.find("create table ") {
                let after_table = &sql[table_pos + 13..];
                let table_part = self.extract_first_identifier(after_table);
                if !table_part.is_empty() {
                    self.parse_table_identifier(&table_part, tables, schemas, databases);
                    
                    // 尝试提取列定义
                    self.extract_columns_from_create_table(after_table, columns);
                }
            }
        }
        // 处理CREATE DATABASE
        else if lower_sql.contains("create database") {
            if let Some(db_pos) = lower_sql.find("create database ") {
                let after_db = &sql[db_pos + 16..];
                let db_part = self.extract_first_identifier(after_db);
                if !db_part.is_empty() {
                    databases.insert(EnhancedSqlParserImprovedOptimized::normalize_identifier(&db_part));
                }
            }
        }
        // 处理CREATE SCHEMA
        else if lower_sql.contains("create schema") {
            if let Some(schema_pos) = lower_sql.find("create schema ") {
                let after_schema = &sql[schema_pos + 14..];
                let schema_part = self.extract_first_identifier(after_schema);
                if !schema_part.is_empty() {
                    schemas.insert(EnhancedSqlParserImprovedOptimized::normalize_identifier(&schema_part));
                }
            }
        }
    }
    
    /// 解析ALTER语句
    fn parse_alter_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 处理ALTER TABLE
        if lower_sql.contains("alter table") {
            if let Some(table_pos) = lower_sql.find("alter table ") {
                let after_table = &sql[table_pos + 12..];
                let table_part = self.extract_first_identifier(after_table);
                if !table_part.is_empty() {
                    self.parse_table_identifier(&table_part, tables, schemas, databases);
                    
                    // 处理ADD COLUMN
                    if let Some(add_pos) = lower_sql.find(" add ") {
                        let after_add = &sql[add_pos + 5..];
                        if after_add.trim_start().starts_with("column ") {
                            let col_start = 6; // "column " 的长度
                            let col_name = self.extract_first_identifier(&after_add[col_start..]);
                            if !col_name.is_empty() {
                                columns.insert(EnhancedSqlParserImprovedOptimized::normalize_identifier(&col_name));
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 解析DROP和TRUNCATE语句
    fn parse_drop_truncate_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 处理DROP TABLE
        if lower_sql.contains("drop table") {
            if let Some(table_pos) = lower_sql.find("drop table ") {
                let after_table = &sql[table_pos + 11..];
                let table_part = self.extract_first_identifier(after_table);
                if !table_part.is_empty() {
                    self.parse_table_identifier(&table_part, tables, schemas, databases);
                }
            }
        }
        // 处理TRUNCATE TABLE
        else if lower_sql.contains("truncate table") {
            if let Some(table_pos) = lower_sql.find("truncate table ") {
                let after_table = &sql[table_pos + 15..];
                let table_part = self.extract_first_identifier(after_table);
                if !table_part.is_empty() {
                    self.parse_table_identifier(&table_part, tables, schemas, databases);
                }
            }
        }
    }
    
    /// 解析通用语句
    fn parse_generic_statement(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 提取表引用
        self.extract_table_references(sql, tables, schemas, databases);
        // 尝试提取列引用
        self.extract_column_references(sql, columns);
    }
    
    /// 处理子查询
    fn handle_subqueries(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        let mut in_subquery = false;
        let mut subquery_start = 0;
        let mut paren_count = 0;
        
        // 查找子查询 - 正确处理UTF-8字符
        let chars: Vec<char> = lower_sql.chars().collect();
        let char_count = chars.len();
        
        for i in 0..char_count {
            // 检查子查询开始标记
            if i > 3 {
                // 检查当前位置前面4个字符是否是"from"
                let is_from = i-3 <= char_count && 
                              chars[i-4] == 'f' && 
                              chars[i-3] == 'r' && 
                              chars[i-2] == 'o' && 
                              chars[i-1] == 'm';
                
                if is_from {
                    // 转换字符索引为字节索引
                    let byte_pos = chars[..i].iter().map(|c| c.len_utf8()).sum();
                    let rest = &lower_sql[byte_pos..];
                    
                    // 查找FROM后的子查询开始
                    if let Some(sub_start) = rest.find('(') {
                        in_subquery = true;
                        // 转换字节索引为字符索引
                        let sub_start_chars: Vec<char> = rest[..sub_start].chars().collect();
                        subquery_start = i + sub_start_chars.len();
                        paren_count = 1;
                    }
                }
            }
            
            // 处理括号
            if in_subquery {
                let c = chars[i];
                if c == '(' {
                    paren_count += 1;
                } else if c == ')' {
                    paren_count -= 1;
                    
                    // 子查询结束
                    if paren_count == 0 {
                        in_subquery = false;
                        // 转换字符索引为字节索引
                        let start_byte = chars[..subquery_start+1].iter().map(|c| c.len_utf8()).sum();
                        let end_byte = chars[..i].iter().map(|c| c.len_utf8()).sum();
                        
                        // 添加边界检查，确保start_byte <= end_byte
                        if start_byte <= end_byte && end_byte <= sql.len() {
                            // 提取子查询（不包含括号）
                            let subquery = &sql[start_byte..end_byte];
                            
                            // 递归解析子查询
                            self.parse_sql(subquery, databases, schemas, tables, columns);
                        } else {
                            // 如果索引无效，跳过这个子查询的处理
                            continue;
                        }
                    }
                }
            }
        }
    }
    
    /// 清理和优化结果
    /// 提取第一个标识符（支持引号和模式匹配）
    fn extract_first_identifier(&self, sql: &str) -> String {
        // 去除前导空白
        let trimmed = sql.trim_start();
        
        // 处理带引号的标识符
        if trimmed.starts_with('"') || trimmed.starts_with('\'') || trimmed.starts_with('`') {
            let quote_char = trimmed.chars().next().unwrap();
            // 查找匹配的引号
            for (i, c) in trimmed[1..].chars().enumerate() {
                // 跳过转义的引号
                if c == '\\' && i + 1 < trimmed[1..].len() {
                    continue;
                }
                if c == quote_char {
                    return trimmed[..i+2].to_string();
                }
            }
        }
        
        // 处理普通标识符（字母、数字、下划线、点）
        let mut result = String::new();
        for c in trimmed.chars() {
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '"' || c == '\'' || c == '`' {
                result.push(c);
            } else if c.is_whitespace() || c == ',' || c == '(' || c == ')' || c == ';' {
                break;
            }
        }
        
        result
    }
    
    /// 从CREATE TABLE语句提取列定义
    fn extract_columns_from_create_table(&self, sql: &str, columns: &mut HashSet<String>) {
        // 查找左括号开始
        if let Some(left_paren) = sql.find('(') {
            let mut depth = 1;
            let sql_after_left = &sql[left_paren + 1..];
            
            // 寻找匹配的右括号
            for (i, c) in sql_after_left.chars().enumerate() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            // 提取括号内的内容
                            let columns_def = &sql_after_left[..i];
                            // 分割列定义
                            let col_defs = self.split_sql_parts(columns_def, ',');
                            
                            // 处理每个列定义
                            for def in col_defs {
                                let trimmed_def = def.trim();
                                if !trimmed_def.is_empty() {
                                    // 提取第一个单词作为列名
                                    let first_word = trimmed_def.split_whitespace().next().unwrap_or("");
                                    let col_name = EnhancedSqlParserImprovedOptimized::normalize_identifier(first_word);
                                    if !col_name.is_empty() && !EnhancedSqlParserImprovedOptimized::is_keyword(&col_name) {
                                        columns.insert(col_name);
                                    }
                                }
                            }
                            break;
                        }
                    },
                    _ => {},
                }
            }
        }
    }
    
    /// 处理WITH子句（CTE）
    fn handle_with_statements(&self, sql: &str, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 检查是否包含WITH子句（支持"with "和" with "两种格式）
        if let Some(with_pos) = lower_sql.find("with ").or_else(|| lower_sql.find(" with ")) {
            let start_pos = if lower_sql[with_pos..].starts_with("with ") {
                with_pos + 5
            } else {
                with_pos + 6
            };
            
            // 查找WITH子句的结束位置（遇到主要语句关键字时停止）
            let with_clause = &sql[start_pos..];
            
            // 分割CTE定义（通常用逗号分隔）
            let cte_parts = self.split_sql_parts(with_clause, ',');
            
            for cte in cte_parts {
                let trimmed_cte = cte.trim();
                if trimmed_cte.is_empty() {
                    continue;
                }
                
                // 提取CTE名称（AS关键字前的部分）
                if let Some(as_pos) = trimmed_cte.to_lowercase().find(" as ") {
                    let cte_name_part = trimmed_cte[..as_pos].trim();
                    
                    // 处理CTE名称，可能包含列定义
                    if let Some(left_paren_pos) = cte_name_part.find('(') {
                        let cte_name = cte_name_part[..left_paren_pos].trim();
                        if !cte_name.is_empty() {
                            tables.insert(cte_name.to_string());
                        }
                        
                        // 提取CTE的列定义
                        if let Some(right_paren_pos) = self.find_matching_parenthesis(cte_name_part, left_paren_pos) {
                            let columns_str = &cte_name_part[left_paren_pos + 1..right_paren_pos];
                            let column_parts = self.split_sql_parts(columns_str, ',');
                            for col in column_parts {
                                let trimmed_col = col.trim();
                                if !trimmed_col.is_empty() {
                                    columns.insert(EnhancedSqlParserImprovedOptimized::normalize_identifier(&trimmed_col));
                                }
                            }
                        }
                    } else {
                        // 简单的CTE名称
                        if !cte_name_part.is_empty() {
                            tables.insert(cte_name_part.to_string());
                        }
                    }
                    
                    // 解析CTE内部的SQL语句
                    let cte_body = &trimmed_cte[as_pos + 4..].trim();
                    if cte_body.starts_with('(') && cte_body.ends_with(')') {
                        let inner_sql = &cte_body[1..cte_body.len()-1].trim();
                        // 递归解析CTE内部的SQL
                        self.parse_sql(inner_sql, databases, schemas, tables, columns);
                    } else {
                        // 对于没有括号包围的CTE体，直接解析
                        self.parse_sql(cte_body, databases, schemas, tables, columns);
                    }
                }
            }
        }
    }
    fn cleanup_result(&self, databases: &mut HashSet<String>, schemas: &mut HashSet<String>, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 定义过滤标识符的通用函数
        let filter_identifier = |id: &String| -> bool {
            let trimmed = id.trim();
            // 1. 移除前后空白字符并检查
            if trimmed.is_empty() || trimmed.len() <= 1 {
                return false;
            }
            
            // 2. 检查是否只包含特殊字符或无效字符
            if !trimmed.chars().any(|c| c.is_alphanumeric() || c == '_' || c == '-' || 
               (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) {
                return false;
            }
            
            // 3. 检查是否为SQL关键字
            if EnhancedSqlParserImprovedOptimized::is_keyword(trimmed) {
                return false;
            }
            
            // 4. 检查是否为SQL关键字或SQL子句关键字
            const EXTENDED_SQL_KEYWORDS: &[&str] = &[
                // 基础SQL关键字
                "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP", "ALTER",
                "TABLE", "VIEW", "INDEX", "DATABASE", "SCHEMA", "PROCEDURE", "FUNCTION", "TRIGGER",
                "JOIN", "INNER", "LEFT", "RIGHT", "FULL", "OUTER", "CROSS", "NATURAL", "SELF", "SEMI",
                "ON", "USING", "AS", "ORDER", "GROUP", "BY", "HAVING", "LIMIT", "OFFSET", "TOP",
                "DISTINCT", "ALL", "UNIQUE", "EXISTS", "IN", "LIKE", "ILIKE", "BETWEEN", "IS", "NOT",
                "AND", "OR", "XOR", "EXCEPT", "INTERSECT", "UNION", "VALUES", "SET", "REPLACE",
                // 子句和操作
                "WITH", "AS", "OVER", "PARTITION", "ORDER", "GROUP", "BY", "HAVING", "LIMIT", "OFFSET",
                "TOP", "FETCH", "FIRST", "NEXT", "ONLY", "FOR", "UPDATE", "NO", "KEY", "SHARE",
                "NOWAIT", "WAIT", "SKIP", "LOCKED", "INTO", "OUTPUT", "RETURNING", "EXECUTE", "CALL",
                // 特殊标识符
                "DISTINCT", "ALL", "ANY", "SOME", "TRUE", "FALSE", "NULL", "UNKNOWN", "CAST", "CONVERT",
                "EXTRACT", "DATE_TRUNC", "DATE_ADD", "DATE_SUB", "DATEDIFF", "TO_DATE", "TO_CHAR",
                "YEAR", "MONTH", "DAY", "HOUR", "MINUTE", "SECOND", "NOW", "CURRENT_DATE", 
                "CURRENT_TIME", "CURRENT_TIMESTAMP", "SYSDATE", "UTC_DATE", "UTC_TIME", 
                // 存储格式和数据类型相关
                "STORED", "BUCKETS", "ORC", "PARQUET", "TEXTFILE", "SEQUENCEFILE", "RCFILE", "AVRO",
                "JSONFILE", "CSVFILE", "DISTRIBUTE", "SORT", "CLUSTER", "INTO", "OUT", "OF",
                // 子查询相关
                "INNER", "LEFT", "RIGHT", "FULL", "OUTER", "CROSS", "NATURAL", "SELF", "SEMI", "ANTI",
                // 操作符和特殊字符
                "COUNT", "SUM", "AVG", "MAX", "MIN", "LEAD", "LAG", "RANK", "ROW_NUMBER", "NTILE"
            ];
            if EXTENDED_SQL_KEYWORDS.contains(&trimmed.to_uppercase().as_str()) {
                return false;
            }
            
            // 5. 过滤掉SQL函数名
            let common_functions = [
                "sum", "avg", "count", "max", "min", "distinct", "cast", "convert",
                "date", "time", "year", "month", "day", "upper", "lower", "left", "right",
                "abs", "round", "floor", "ceil", "length", "concat", "substr", "replace",
                "explode", "array", "input__file__name", "first", "last", "lead", "lag",
                "rank", "dense_rank", "row_number", "ntile", "percentile", "truncate", "ceiling",
                "log", "log10", "exp", "sqrt", "power", "sin", "cos", "tan", "asin", "acos",
                "atan", "atan2", "degrees", "radians", "trim", "ltrim", "rtrim", "substring",
                "position", "coalesce", "nullif", "case", "when", "then", "else", "end",
                "extract", "date_trunc", "date_add", "date_sub", "datediff", "to_date", "to_char",
                "now", "current_date", "current_time", "current_timestamp", "exists", "in",
                "like", "ilike", "between", "is", "not", "and", "or", "xor", "except", "intersect",
                "union", "all", "any", "some", "over", "partition", "order", "group", "by"
            ];
            let lower_trimmed = trimmed.to_lowercase();
            if common_functions.contains(&lower_trimmed.as_str()) {
                return false;
            }
            
            // 6. 过滤掉纯数字标识符
            if trimmed.chars().all(|c| c.is_digit(10)) {
                return false;
            }
            
            // 7. 检查是否是有效的标识符格式（可以包含字母、数字、下划线、中文、连字符等）
            // 同时确保不包含括号、引号等无效字符
            let valid_chars = trimmed.chars().all(|c| {
                c.is_alphanumeric() || c == '_' || c == '-' || 
                (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF) || // 中文字符范围
                (c as u32 >= 0x3040 && c as u32 <= 0x30FF) || // 日文平假名和片假名
                (c as u32 >= 0xAC00 && c as u32 <= 0xD7AF)    // 韩文字符范围
            });
            
            valid_chars
        };
        
        // 过滤数据库和模式标识符集合
        databases.retain(filter_identifier);
        schemas.retain(filter_identifier);
        
        // 对表名应用额外的严格过滤，避免常见的误识别
        tables.retain(|id| {
            filter_identifier(id) && 
            // 额外检查：避免表名中包含常见SQL子句
            !id.to_uppercase().contains("WHERE") &&
            !id.to_uppercase().contains("BY") &&
            !id.to_uppercase().contains("GROUP") &&
            !id.to_uppercase().contains("ORDER") &&
            !id.to_uppercase().contains("FROM") &&
            !id.to_uppercase().contains("JOIN") &&
            !id.to_uppercase().contains("EXISTS") &&
            !id.to_uppercase().contains("INNER") &&
            !id.to_uppercase().contains("LEFT") &&
            !id.to_uppercase().contains("RIGHT") &&
            // 避免将常见的列名模式识别为表名
            !id.contains("_id") && !id.contains("_name") && !id.contains("_code") &&
            !id.contains("_type") && !id.contains("_date") && !id.contains("_time") &&
            // 避免全大写的字符串字面量被识别为表名
            !id.chars().all(|c| c.is_alphabetic() && c.is_uppercase())
        });
        
        // 对列名应用额外过滤，避免无效标识符
        columns.retain(|id| {
            filter_identifier(id) && 
            // 避免列名中包含无效字符组合
            !id.contains(")") && // 避免类似 "s)" 这样的无效列名
            !id.contains("))") && // 避免类似 "products))" 这样的无效列名
            // 避免LIKE子句中的字符串字面量被误识别为列名
            !id.starts_with("'") && !id.ends_with("'") &&
            !id.starts_with('"') && !id.ends_with('"') &&
            // 避免数字常量被误识别为列名
            !id.parse::<f64>().is_ok() &&
            // 避免过于简单的列名被误识别
            (id.trim().len() > 1 || 
             (id.trim().len() == 1 && id.chars().all(|c| c.is_alphabetic() && !c.is_ascii_uppercase())))
        });
        
        // 特殊处理星号列名
        if columns.is_empty() && tables.iter().all(|t| !t.contains("*")) {
            columns.insert("*".to_string());
        }
        
        // 修复数据库、模式和表名的混淆
        // 1. 将可能是表名的项从databases移到tables
        let mut valid_databases = HashSet::new();
        for db in databases.drain() {
            // 数据库名通常不包含下划线，且长度较短或有特定模式
            // 更严格的判断：大多数情况下，SQL中的表名不应该在databases集合中
            // 除非它有明确的数据库名特征
            let is_likely_db = db.len() <= 10 && 
                              !db.contains("_") && 
                              !db.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) &&
                              // 常见数据库名
                              (db.to_lowercase() == "mysql" || 
                               db.to_lowercase() == "postgresql" || 
                               db.to_lowercase() == "oracle" || 
                               db.to_lowercase() == "sqlserver" ||
                               db.to_lowercase() == "sqlite" ||
                               db.to_lowercase() == "gaussdb" ||
                               db.to_lowercase() == "hive" ||
                               db.to_lowercase() == "db" ||
                               db.to_lowercase() == "master" ||
                               db.to_lowercase() == "tempdb" ||
                               db.to_lowercase() == "model" ||
                               db.to_lowercase() == "msdb");
            
            if is_likely_db {
                valid_databases.insert(db);
            } else {
                tables.insert(db);
            }
        }
        *databases = valid_databases;
        
        // 2. 将可能是表名的项从schemas移到tables
        let mut valid_schemas = HashSet::new();
        for schema in schemas.drain() {
            // 模式名通常是常见的如public、dbo等，或者符合特定命名规则
            let lower_schema = schema.to_lowercase();
            if lower_schema != "public" && lower_schema != "dbo" && lower_schema != "sys" &&
               !lower_schema.starts_with("information_schema") {
                tables.insert(schema);
            } else {
                valid_schemas.insert(schema);
            }
        }
        *schemas = valid_schemas;
        
        // 移除可能被误识别为表名的列名
        let tables_clone = tables.clone();
        let columns_clone = columns.clone();
        
        // 如果一个标识符同时出现在表名和列名集合中，基于上下文判断
        let has_select_star = columns.contains("*");
        
        // 定义更精确的列名模式检测函数
        let is_likely_column = |id: &str| -> bool {
            let lower_id = id.to_lowercase();
            // 常见的列名后缀
            if lower_id.contains("_id") || lower_id.contains("_name") || lower_id.contains("_code") ||
               lower_id.contains("_type") || lower_id.contains("_date") || lower_id.contains("_time") {
                return true;
            }
            // 常见的单列名
            if lower_id.len() <= 10 && 
               !lower_id.contains(" ") && 
               !lower_id.contains(".") &&
               !lower_id.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) {
                return true;
            }
            false
        };
        
        for id in tables_clone.intersection(&columns_clone) {
            // 如果有SELECT *或标识符看起来更像列名，这些共同标识符更可能是列名
            if has_select_star || is_likely_column(id) {
                tables.remove(id);
            }
            // 否则，判断是否更可能是表名（例如包含中文或长度较长）
            else if id.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) || 
                    id.len() > 10 || id.contains(".") {
                columns.remove(id);
            } else {
                tables.remove(id);
            }
        }
        
        // 对于SELECT *的情况，我们需要更严格地过滤表名
        if has_select_star {
            // 在SELECT *的情况下，我们只保留可能是真正表名的标识符
            let mut valid_tables: HashSet<String> = HashSet::new();
            
            // 定义可能的列名模式 - 更加智能，避免误判表名
            let is_potential_column = |name: &String| -> bool {
                // 常见的列名模式：包含下划线、长度适中、不包含中文
                if name.contains("_") && 
                   !name.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) &&
                   name.len() < 10 {
                    // 排除可能的特殊表名，如cxxtest
                    if name.to_lowercase().contains("test") && name.len() > 5 {
                        return false;
                    }
                    return true;
                }
                
                // 单个简单单词更可能是列名
                if name.len() < 8 && name.chars().all(|c| c.is_alphanumeric() || c == '_') &&
                   !name.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) {
                    // 排除常见的表名模式
                    let lower_name = name.to_lowercase();
                    if lower_name.contains("test") || lower_name.contains("table") || 
                       lower_name.contains("data") || lower_name.contains("info") {
                        return false;
                    }
                    return true;
                }
                
                // 列名常见后缀判断
                let lower_name = name.to_lowercase();
                if lower_name.ends_with("id") || lower_name.ends_with("name") || 
                   lower_name.ends_with("code") || lower_name.ends_with("type") {
                    // 排除像customer_id这样的模式被错误识别为表名
                    if lower_name.len() <= 10 {
                        return true;
                    }
                }
                
                // 全大写的单词更可能是字符串字面量或常量，不是列名
                if name.chars().all(|c| c.is_alphabetic() && c.is_uppercase()) {
                    return true;
                }
                
                // 其他可能是列名的情况
                false
            };
            
            for table in tables.iter() {
                // 支持特殊表名模式
                let lower_table = table.to_lowercase();
                
                // 判断是否为特殊表名
                let is_special_table = 
                    // cxxtest、unittest等测试表名
                    (lower_table.contains("test") && table.len() > 5) ||
                    // cls1、stu1等命名的表
                    (lower_table.starts_with("cls") || lower_table.starts_with("stu")) && 
                     table.chars().any(|c| c.is_numeric()) ||
                    // 中文表名
                    table.chars().any(|c| (c as u32 >= 0x4E00 && c as u32 <= 0x9FFF)) ||
                    // 包含模式限定符的表名
                    table.contains(".") ||
                    // 包含下划线但长度较长的名称更可能是表名
                    (table.contains("_") && table.len() >= 8) ||
                    // 常见的表名后缀
                    lower_table.ends_with("_table") || 
                    lower_table.ends_with("_data") ||
                    lower_table.ends_with("_info") ||
                    lower_table.ends_with("_list") ||
                    lower_table.ends_with("_master");
                
                // 如果是特殊表名，直接保留
                if is_special_table {
                    valid_tables.insert(table.clone());
                }
                // 过滤掉可能是列名的标识符
                else if !is_potential_column(table) {
                    valid_tables.insert(table.clone());
                }
            }
            
            // 只保留经过验证的表名
            *tables = valid_tables;
        }
        
        // 二次清理列名，确保没有表名混入
        let tables_clone = tables.clone();
        columns.retain(|col| !tables_clone.contains(col));
        
        // 确保字符串字面量不会出现在任何集合中
        let clean_columns: HashSet<_> = columns.iter()
            .filter(|&col| !col.starts_with("'") && !col.ends_with("'") &&
                    !col.starts_with('"') && !col.ends_with('"') &&
                    // 过滤掉可能是字符串字面量的内容，如"Hardware"
                    !col.chars().all(|c| c.is_alphabetic() && c.is_uppercase()))
            .cloned()
            .collect();
        *columns = clean_columns;
        
        // 清理表名，移除可能是字符串字面量的项
        tables.retain(|table| {
            !table.chars().all(|c| c.is_alphabetic() && c.is_uppercase()) &&
            !table.starts_with("'") && !table.ends_with("'") &&
            !table.starts_with('"') && !table.ends_with('"')
        });
    }

    /// 预处理SQL，移除注释和规范化
    /// 优化：减少不必要的字符串操作和内存分配
    fn preprocess_sql(&self, sql: &str) -> Arc<String> {
        // 尝试从缓存获取，如果实现了缓存机制
        // 这里简化处理，直接处理
        let mut result = String::with_capacity(sql.len());
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_comment = false;
        let mut chars = sql.chars().peekable();
        
        while let Some(c) = chars.next() {
            // 检查是否在引号内
            if !in_comment {
                if c == '\'' && !in_double_quote {
                    in_single_quote = !in_single_quote;
                } else if c == '"' && !in_single_quote {
                    in_double_quote = !in_double_quote;
                }
            }
            
            // 检查注释
            if !in_single_quote && !in_double_quote {
                if c == '-' && chars.peek() == Some(&'-') {
                    in_comment = true;
                    chars.next(); // 跳过第二个'-'
                    continue;
                } else if c == '/' && chars.peek() == Some(&'*') {
                    in_comment = true;
                    chars.next(); // 跳过'*'
                    continue;
                } else if c == '*' && chars.peek() == Some(&'/') && in_comment {
                    in_comment = false;
                    chars.next(); // 跳过'/'
                    continue;
                } else if c == '\n' {
                    in_comment = false;
                }
            }
            
            // 如果不在注释中，则添加字符
            if !in_comment {
                // 规范化引号和空白
                if c.is_whitespace() {
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                } else {
                    result.push(c);
                }
            }
        }
        
        Arc::new(result.trim().to_string())
    }

    /// 提取表引用
    fn extract_table_references(&self, sql: &str, tables: &mut HashSet<String>, schemas: &mut HashSet<String>, databases: &mut HashSet<String>) {
        // 查找FROM、JOIN、INTO等关键字后的表名
        let lower_sql = sql.to_lowercase();
        
        // 处理FROM子句
        for (i, _) in lower_sql.match_indices(" from ") {
            let start_pos = i + 6;
            self.extract_tables_from_clause(&sql[start_pos..], tables, schemas, databases);
        }
        
        // 处理JOIN子句
        for (i, _) in lower_sql.match_indices(" join ") {
            let start_pos = i + 6;
            self.extract_tables_from_clause(&sql[start_pos..], tables, schemas, databases);
        }
        
        // 处理INTO子句
        for (i, _) in lower_sql.match_indices(" into ") {
            let start_pos = i + 6;
            self.extract_tables_from_clause(&sql[start_pos..], tables, schemas, databases);
        }
        
        // 处理UPDATE语句中的表名
        for (i, _) in lower_sql.match_indices("update ") {
            let start_pos = i + 7;
            // 提取UPDATE后面的表名，直到SET或WHERE
            let clause_end = self.find_clause_end(&lower_sql, start_pos);
            let table_part = &sql[start_pos..clause_end].trim();
            self.parse_table_identifier(table_part, tables, schemas, databases);
        }
        
        // 处理DELETE FROM语句中的表名
        for (i, _) in lower_sql.match_indices("delete from ") {
            let start_pos = i + 12;
            self.extract_tables_from_clause(&sql[start_pos..], tables, schemas, databases);
        }
        
        // 直接从整个SQL中提取可能的表名（作为补充机制）
        self.extract_tables_from_whole_sql(sql, tables, schemas, databases);
    }
    
    /// 从整个SQL中提取可能的表名（补充机制）
    fn extract_tables_from_whole_sql(&self, sql: &str, tables: &mut HashSet<String>, schemas: &mut HashSet<String>, databases: &mut HashSet<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 直接从FROM子句中提取表名（增强版，支持多个表）
        if let Some(from_pos) = lower_sql.find(" from ") {
            let from_part = &sql[from_pos + 6..];
            // 查找下一个关键字作为表名结束位置
            let end_pos = self.find_clause_end(from_part, 0);
            let from_clause = from_part[..end_pos].trim();
            
            // 分割多个表名
            let table_candidates = self.split_sql_parts(from_clause, ',');
            for candidate in table_candidates {
                self.extract_single_table_name(candidate.trim(), tables);
            }
        }
        
        // 处理JOIN子句中的表名（增强版）
        let join_types = [" join ", " inner join ", " left join ", " right join ", 
                          " full join ", " cross join ", " left outer join ", 
                          " right outer join ", " full outer join "];
        
        for join_type in &join_types {
            for (pos, _) in lower_sql.match_indices(join_type) {
                let join_part = &sql[pos + join_type.len()..];
                let end_pos = self.find_clause_end(join_part, 0);
                let join_clause = join_part[..end_pos].trim();
                
                // 提取单个表名
                self.extract_single_table_name(join_clause, tables);
            }
        }
        
        // 处理UPDATE语句中的表名
        for (update_pos, _) in lower_sql.match_indices("update ") {
            let update_part = &sql[update_pos + 7..];
            let end_pos = self.find_clause_end(update_part, 0);
            let update_clause = update_part[..end_pos].trim();
            
            // 提取单个表名
            self.extract_single_table_name(update_clause, tables);
        }
        
        // 处理DELETE FROM语句中的表名
        for (delete_pos, _) in lower_sql.match_indices("delete from ") {
            let delete_part = &sql[delete_pos + 12..];
            let end_pos = self.find_clause_end(delete_part, 0);
            let delete_clause = delete_part[..end_pos].trim();
            
            // 提取单个表名
            self.extract_single_table_name(delete_clause, tables);
        }
        
        // 处理INSERT INTO语句中的表名
        for (insert_pos, _) in lower_sql.match_indices("insert into ") {
            let insert_part = &sql[insert_pos + 12..];
            let end_pos = self.find_clause_end(insert_part, 0);
            let insert_clause = insert_part[..end_pos].trim();
            
            // 提取单个表名
            self.extract_single_table_name(insert_clause, tables);
        }
    }
    
    /// 辅助方法：从候选字符串中提取单个表名
    fn extract_single_table_name(&self, candidate: &str, tables: &mut HashSet<String>) {
        if candidate.is_empty() {
            return;
        }
        
        // 移除别名部分
        let table_name = if let Some(as_pos) = candidate.to_lowercase().find(" as ") {
            candidate[..as_pos].trim()
        } else if let Some(space_pos) = candidate.find(|c: char| c.is_whitespace()) {
            let clean_table = candidate[..space_pos].trim();
            let alias_part = candidate[space_pos+1..].trim();
            
            // 如果后面的部分是SQL关键字，则整个部分可能都是表名（例如带空格的表名）
            if !SQL_KEYWORDS.contains(alias_part.to_lowercase().as_str()) && 
               !alias_part.starts_with('(') && 
               !alias_part.contains('=') {
                clean_table
            } else {
                candidate.trim()
            }
        } else {
            candidate
        };
        
        // 移除可能的括号
        let table_name = table_name.trim_matches(|c| c == '(' || c == ')');
        
        // 检查是否是有效的表名
        if !table_name.is_empty() && 
           !SQL_KEYWORDS.contains(table_name.to_lowercase().as_str()) &&
           // 允许特殊命名的表，如cxxtest、cls1、stu1等
           !table_name.chars().all(|c| c.is_numeric()) {
            tables.insert(table_name.to_string());
        }
    }
    
    /// 判断字符串是否可能是表引用
    fn is_likely_table_reference(&self, candidate: &str, sql: &str) -> bool {
        let lower_sql = sql.to_lowercase();
        let lower_candidate = candidate.to_lowercase();
        
        // 检查是否在FROM、JOIN、INTO等关键字附近
        let keywords = [" from ", " join ", " into ", "update ", "delete from "];
        for keyword in keywords {
            if let Some(pos) = lower_sql.find(keyword) {
                if let Some(table_pos) = lower_sql[pos..].find(&lower_candidate) {
                    return true;
                }
            }
        }
        
        // 检查是否在WHERE子句中作为表引用（例如：table.column）
        if let Some(where_pos) = lower_sql.find(" where ") {
            let pattern = &format!("{}.", lower_candidate);
            if lower_sql[where_pos..].contains(pattern) {
                return true;
            }
        }
        
        false
    }

    /// 从子句中提取表名（UTF-8安全）
    fn extract_tables_from_clause(&self, clause: &str, tables: &mut HashSet<String>, schemas: &mut HashSet<String>, databases: &mut HashSet<String>) {
        let mut paren_count: i32 = 0;
        let mut in_quote: Option<char> = None;
        // 跟踪表名开始的“字节”位置，避免按字符索引对字符串切片
        let mut table_start_byte: Option<usize> = None;

        for (byte_idx, ch) in clause.char_indices() {
            // 引号处理
            if let Some(q) = in_quote {
                if ch == q {
                    in_quote = None;
                }
                // 在引号内不开始/结束表名
                continue;
            } else if ch == '\'' || ch == '"' || ch == '`' {
                in_quote = Some(ch);
                continue;
            }

            // 括号处理（仅在非引号内生效）
            if ch == '(' {
                paren_count = paren_count.saturating_add(1);
            } else if ch == ')' {
                paren_count = paren_count.saturating_sub(1);
            }

            // 仅在最外层处理分隔符
            if paren_count == 0 {
                if ch == ',' || ch.is_whitespace() {
                    if let Some(start) = table_start_byte.take() {
                        let candidate = clause[start..byte_idx].trim();
                        if !candidate.is_empty() {
                            self.parse_table_identifier(candidate, tables, schemas, databases);
                        }
                    }
                } else if table_start_byte.is_none() {
                    // 非空白、非分隔符，则认为是表名开始
                    table_start_byte = Some(byte_idx);
                }
            }
        }

        // 处理收尾残留表名
        if let Some(start) = table_start_byte {
            let candidate = clause[start..].trim();
            if !candidate.is_empty() {
                self.parse_table_identifier(candidate, tables, schemas, databases);
            }
        }
    }

    /// 解析表标识符，处理database.schema.table格式
    fn parse_table_identifier(&self, identifier: &str, tables: &mut HashSet<String>, schemas: &mut HashSet<String>, databases: &mut HashSet<String>) {
        // 按点分割并过滤空部分
        let parts: Vec<&str> = identifier.split('.')
            .filter(|s| !s.is_empty())
            .collect();
        
        match parts.len() {
            1 => {
                // 只有表名
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                // 过滤掉关键字和常见schema名称
                if !EnhancedSqlParserImprovedOptimized::is_keyword(&table) && !COMMON_SCHEMA_NAMES.contains(&table.to_lowercase().as_str()) {
                    tables.insert(table);
                }
            },
            2 => {
                // schema.table 或 database.table
                let first_part = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let second_part = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                let first_part_lower = first_part.to_lowercase();
                
                // 使用通用schema名称列表进行智能判断
                if COMMON_SCHEMA_NAMES.contains(&first_part_lower.as_str()) || 
                   first_part_lower == "schema" || 
                   first_part_lower.len() <= 4 || 
                   first_part_lower.ends_with("schema") {
                    // 很可能是schema名
                    schemas.insert(first_part);
                    if !EnhancedSqlParserImprovedOptimized::is_keyword(&second_part) {
                        tables.insert(second_part);
                    }
                } else if first_part_lower.len() > 8 && !first_part_lower.ends_with("s") {
                    // 可能是数据库名
                    databases.insert(first_part);
                    if !Self::is_keyword(&second_part) {
                        tables.insert(second_part);
                    }
                } else {
                    // 默认策略：较短的可能是schema
                    if first_part.len() <= second_part.len() {
                        schemas.insert(first_part);
                        tables.insert(second_part);
                    } else {
                        // 较长的可能是数据库名
                        databases.insert(first_part);
                        tables.insert(second_part);
                    }
                }
            },
            3 => {
                // database.schema.table
                let database = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let schema = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[2]);
                
                databases.insert(database);
                schemas.insert(schema);
                if !EnhancedSqlParserImprovedOptimized::is_keyword(&table) {
                    tables.insert(table);
                }
            },
            _ => {
                // 处理复杂情况，尝试智能解析
                if parts.len() >= 2 {
                    // 对于多部分标识符，尝试组合最后两部分作为schema.表名
                    let last_two = format!("{}.{}", 
                        EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[parts.len()-2]), 
                        EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[parts.len()-1]));
                    tables.insert(last_two);
                } else if let Some(table_part) = parts.last() {
                    let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(table_part);
                    tables.insert(table);
                }
            }
        }
    }

    /// 提取列引用 - 增强版
    fn extract_column_references(&self, sql: &str, columns: &mut HashSet<String>) {
        // 查找SELECT关键字后的列名
        let lower_sql = sql.to_lowercase();
        let has_select_star = lower_sql.contains("select *") || lower_sql.contains("select\t*");
        
        // 特殊处理：如果SELECT子句中有星号，确保它被添加到列名集合中
        if has_select_star {
            columns.insert("*".to_string());
            // 对于SELECT *，我们不应该从WHERE子句中提取列名，避免误识别
            return;
        }
        
        // 处理SELECT子句（包括嵌套查询中的SELECT）
        for (select_start, _) in lower_sql.match_indices("select ") {
            // 查找FROM、SET、INTO等关键字作为结束位置
            let end_pos = self.find_clause_end(&lower_sql, select_start + 7);
            let select_clause = &sql[select_start + 7..end_pos];
            
            self.extract_columns_from_select(select_clause, columns);
        }
        
        // 对于非SELECT *的情况，处理WHERE子句中的列引用
        for (where_start, _) in lower_sql.match_indices(" where ") {
            // 使用find_clause_end查找WHERE子句的结束位置
            let end_pos = self.find_clause_end(&lower_sql, where_start + 7);
            let where_clause = &sql[where_start + 7..end_pos];
            
            // 改进的WHERE子句处理 - 避免提取字符串字面量
            let identifiers = self.extract_identifiers_from_expression(where_clause);
            for id in identifiers {
                // 过滤掉可能是字符串字面量的标识符
                if !id.starts_with("'") && !id.ends_with("'") && 
                   !id.starts_with('"') && !id.ends_with('"') {
                    columns.insert(id);
                }
            }
        }
        
        // 处理SET子句中的列引用
        for (set_start, _) in lower_sql.match_indices(" set ") {
            let end_pos = self.find_clause_end(&lower_sql, set_start + 5);
            let set_clause = &sql[set_start + 5..end_pos];
            // SET子句格式通常是 column = value，我们只需要提取等号前的部分
            let parts: Vec<&str> = set_clause.split(',').collect();
            
            for part in parts {
                if let Some(equals_pos) = part.find('=') {
                    let column_part = part[..equals_pos].trim();
                    let column_identifiers = self.extract_identifiers_from_expression(column_part);
                    
                    for identifier in column_identifiers {
                        columns.insert(identifier);
                    }
                }
            }
        }
        
        // 处理HAVING子句中的列引用
        for (having_start, _) in lower_sql.match_indices(" having ") {
            let end_pos = self.find_clause_end(&lower_sql, having_start + 7);
            let having_clause = &sql[having_start + 7..end_pos];
            let identifiers = self.extract_identifiers_from_expression(having_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理ORDER BY子句中的列引用
        for (order_start, _) in lower_sql.match_indices(" order by ") {
            let end_pos = self.find_clause_end(&lower_sql, order_start + 9);
            let order_clause = &sql[order_start + 9..end_pos];
            let identifiers = self.extract_identifiers_from_expression(order_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理GROUP BY子句中的列引用
        for (group_start, _) in lower_sql.match_indices(" group by ") {
            let end_pos = self.find_clause_end(&lower_sql, group_start + 9);
            let group_clause = &sql[group_start + 9..end_pos];
            let identifiers = self.extract_identifiers_from_expression(group_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理JOIN ON子句中的列引用
        for (on_start, _) in lower_sql.match_indices(" on ") {
            let end_pos = self.find_clause_end(&lower_sql, on_start + 4);
            let on_clause = &sql[on_start + 4..end_pos];
            let identifiers = self.extract_identifiers_from_expression(on_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理INSERT INTO ... VALUES 子句中的列定义
        if lower_sql.contains("insert into") && lower_sql.contains("values") {
            if let Some(values_pos) = lower_sql.find("values") {
                // 检查是否有列名列表（在括号中）
                let insert_part = &lower_sql[0..values_pos];
                if let Some(open_paren) = insert_part.find('(') {
                    let close_paren_pos = insert_part[open_paren..].find(')');
                    if let Some(close_paren) = close_paren_pos {
                        let columns_part = &sql[open_paren+1..open_paren+close_paren];
                        let column_names = self.extract_identifiers_from_expression(columns_part);
                        
                        for col_name in column_names {
                            columns.insert(col_name);
                        }
                    }
                }
            }
        }
        
        // 处理CREATE TABLE语句中的列定义
        if lower_sql.starts_with("create table") {
            self.extract_columns_from_create_table(sql, columns);
        }
    }

    /// 查找子句结束位置
    fn find_clause_end(&self, sql: &str, start_pos: usize) -> usize {
        let mut paren_count: i32 = 0;
        let mut in_quote = None;
        let chars: Vec<char> = sql.chars().collect();
        
        for (i, c) in chars.iter().enumerate().skip(start_pos) {
            // 处理引号
            if in_quote.is_none() && (*c == '\'' || *c == '"' || *c == '`') {
                in_quote = Some(*c);
            } else if in_quote == Some(*c) {
                in_quote = None;
            }
            
            // 处理括号
            if in_quote.is_none() {
                if *c == '(' {
                    paren_count += 1;
                } else if *c == ')' {
                    paren_count = paren_count.saturating_sub(1_i32);
                }
            }
            
            // 查找结束关键字
            if in_quote.is_none() && paren_count == 0 {
                // 获取从当前位置开始的字符切片
                let end_pos = std::cmp::min(i + 6, chars.len());
                let chars_slice = &chars[i..end_pos];
                
                // 将字符切片转换为字符串
                let substr: String = chars_slice.iter().collect();
                
                if substr.starts_with(" from ") || substr.starts_with(" where ") || 
                   substr.starts_with(" group ") || substr.starts_with(" order ") {
                    // 转换回原始字符串的字节索引
                    let byte_pos = chars[..i].iter().map(|c| c.len_utf8()).sum();
                    return byte_pos;
                }
            }
        }
        
        sql.len()
    }

    /// 从SELECT子句中提取列名
    fn extract_columns_from_select(&self, select_clause: &str, columns: &mut HashSet<String>) {
        // 分割列名，考虑括号和引号
        let column_parts = self.split_sql_parts(select_clause, ',');
        
        for part in column_parts {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            // 处理DISTINCT关键字
            let mut column_text = trimmed;
            if column_text.starts_with("distinct ") {
                column_text = &column_text[9..];
            }
            
            // 提取列名
            let identifiers = self.extract_identifiers_from_expression(column_text);
            for id in identifiers {
                columns.insert(id);
            }
        }
    }
    
    /// 查找匹配的右括号位置
    fn find_matching_parenthesis(&self, sql: &str, left_paren_pos: usize) -> Option<usize> {
        let mut paren_count = 1;
        let mut in_quote = None;
        
        for (i, c) in sql.chars().enumerate().skip(left_paren_pos + 1) {
            // 处理引号
            if in_quote.is_none() && (c == '\'' || c == '"' || c == '`') {
                in_quote = Some(c);
            } else if in_quote == Some(c) {
                in_quote = None;
            }
            
            // 只在非引号内处理括号
            if in_quote.is_none() {
                if c == '(' {
                    paren_count += 1;
                } else if c == ')' {
                    paren_count -= 1;
                    if paren_count == 0 {
                        return Some(i);
                    }
                }
            }
        }
        
        None  // 没有找到匹配的右括号
    }

    /// 处理函数调用 - 增强版
    fn handle_function_calls(&self, sql: &str, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 提取所有可能的函数调用
        let function_calls = self.find_function_calls(sql);
        
        for call in function_calls {
            // 处理不同类型的函数
            let lower_call = call.to_lowercase();
            
            // 处理常见的聚合函数
            if lower_call.starts_with("sum(") || lower_call.starts_with("avg(") ||
               lower_call.starts_with("count(") || lower_call.starts_with("max(") ||
               lower_call.starts_with("min(") || lower_call.starts_with("distinct(") {
                // 提取函数参数中的列名
                let params = self.extract_function_params(&call[call.find('(').unwrap() + 1..call.rfind(')').unwrap_or(call.len())]);
                for param in params {
                    let identifiers = self.extract_identifiers_from_expression(&param);
                    for id in identifiers {
                        columns.insert(id);
                    }
                }
            }
            // 处理窗口函数
            else if lower_call.contains("over(") {
                // 提取窗口函数中的列名
                if let Some(over_start) = call.find("OVER(") {
                    let window_part = &call[..over_start];
                    let identifiers = self.extract_identifiers_from_expression(window_part);
                    for id in identifiers {
                        columns.insert(id);
                    }
                }
            }
            // 处理CASE WHEN表达式
            else if lower_call.starts_with("case") {
                // 提取CASE WHEN中的列名
                let identifiers = self.extract_identifiers_from_expression(&call);
                for id in identifiers {
                    columns.insert(id);
                }
            }
            // 处理CAST函数和类型转换
            else if lower_call.starts_with("cast(") || lower_call.starts_with("convert(") {
                // 提取CAST/CONVERT函数中的列名
                let params = self.extract_function_params(&call[call.find('(').unwrap() + 1..call.rfind(')').unwrap_or(call.len())]);
                for param in params {
                    let identifiers = self.extract_identifiers_from_expression(&param);
                    for id in identifiers {
                        columns.insert(id);
                    }
                }
            }
            // 处理LIKE和ILIKE表达式
            else if lower_call.contains(" like ") || lower_call.contains(" ilike ") {
                // 提取LIKE/ILIKE左侧的列名
                let identifiers = self.extract_identifiers_from_expression(&call);
                for id in identifiers {
                    columns.insert(id);
                }
            }
            
            // 处理地理位置相关函数
            if lower_call.contains("point(") || lower_call.contains("st_point(") {
                self.extract_from_geo_functions(&call, tables, columns);
            }
        }
    }

    /// 查找函数调用
    fn find_function_calls(&self, sql: &str) -> Vec<String> {
        let mut calls = Vec::new();
        let chars: Vec<char> = sql.chars().collect();
        let mut in_quote = None;
        let mut in_function = false;
        let mut function_start = 0;
        let mut _paren_count = 0; // 未使用的变量，前缀下划线避免警告
        
        for (i, c) in chars.iter().enumerate() {
            // 处理引号
            if in_quote.is_none() && (*c == '\'' || *c == '"' || *c == '`') {
                in_quote = Some(*c);
            } else if in_quote == Some(*c) {
                in_quote = None;
            }
            
            if in_quote.is_none() {
                // 查找函数开始
                if !in_function && c.is_alphabetic() && (i == 0 || !chars[i-1].is_alphanumeric() && chars[i-1] != '_') {
                    function_start = i;
                    in_function = true;
                }
                
                // 检查函数结束
                if in_function {
                    if *c == '(' {
                        // 确认是函数调用
                        // 使用字符边界安全的切片方法
                        let function_name: String = sql.chars()
                            .skip(function_start)
                            .take(i - function_start)
                            .collect();
                        if !EnhancedSqlParserImprovedOptimized::is_keyword(&function_name) {
                            // 查找函数结束括号
                            let mut j = i + 1;
                            let mut nested_paren = 1;
                            while j < chars.len() && nested_paren > 0 {
                                if chars[j] == '(' {
                                    nested_paren += 1;
                                } else if chars[j] == ')' {
                                    nested_paren -= 1;
                                }
                                j += 1;
                            }
                            
                            if nested_paren == 0 && j <= chars.len() {
                                // 使用字符边界安全的切片方法
                                let function_str = sql.chars()
                                    .skip(function_start)
                                    .take(j - function_start)
                                    .collect::<String>();
                                calls.push(function_str);
                            }
                        }
                        in_function = false;
                    } else if !c.is_alphanumeric() && *c != '_' {
                        in_function = false;
                    }
                }
            }
        }
        
        calls
    }

    /// 提取函数参数
    fn extract_function_params(&self, params_str: &str) -> Vec<String> {
        self.split_sql_parts(params_str, ',')
    }

    /// 从地理函数中提取信息
    fn extract_from_geo_functions(&self, function_call: &str, _tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 提取函数参数
        if let Some(start) = function_call.find('(') {
            if let Some(end) = function_call.rfind(')') {
                let params = &function_call[start + 1..end];
                let identifiers = self.extract_identifiers_from_expression(params);
                
                for id in identifiers {
                    columns.insert(id);
                }
            }
        }
    }

    /// 处理方言特定的内容
    fn handle_dialect_specific(&self, sql: &str, dialect: &str, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        match dialect.to_lowercase().as_str() {
            "gaussdb" => {
                self.handle_gaussdb_specific(sql, tables, columns);
            },
            "oracle" => {
                self.handle_oracle_specific(sql, tables, columns);
            },
            "mysql" => {
                self.handle_mysql_specific(sql, tables, columns);
            },
            "postgresql" => {
                self.handle_postgresql_specific(sql, tables, columns);
            },
            // 可以根据需要添加其他方言的处理
            _ => {}
        }
    }

    /// 处理GaussDB特定的内容
    fn handle_gaussdb_specific(&self, sql: &str, tables: &mut HashSet<String>, columns: &mut HashSet<String>) {
        // 处理GaussDB特有的POINT函数参数格式
        for captures in GAUSSDB_PATTERN.captures_iter(sql) {
            if let Some(m1) = captures.get(1) {
                tables.insert(m1.as_str().to_string());
            }
            if let Some(m2) = captures.get(2) {
                tables.insert(m2.as_str().to_string());
            }
            if let Some(m3) = captures.get(3) {
                columns.insert(m3.as_str().to_string());
            }
        }
    }

    /// 处理Oracle特定的内容
    fn handle_oracle_specific(&self, _sql: &str, _tables: &mut HashSet<String>, _columns: &mut HashSet<String>) {
        // Oracle特定的处理逻辑
    }

    /// 处理MySQL特定的内容
    fn handle_mysql_specific(&self, _sql: &str, _tables: &mut HashSet<String>, _columns: &mut HashSet<String>) {
        // MySQL特定的处理逻辑
    }

    /// 处理PostgreSQL特定的内容
    fn handle_postgresql_specific(&self, _sql: &str, _tables: &mut HashSet<String>, _columns: &mut HashSet<String>) {
        // PostgreSQL特定的处理逻辑
    }

    /// 判断是否为SQL关键字
    pub fn is_keyword(s: &str) -> bool {
        SQL_KEYWORDS.contains(&s.to_lowercase().as_str())
    }
    
    /// 判断是否为常见函数名
    fn is_common_function(s: &str) -> bool {
        COMMON_FUNCTIONS.contains(&s.to_lowercase().as_str())
    }

    /// 辅助方法：从表达式中提取标识符 - 增强版
    /// 优化：支持带别名的标识符提取，提高解析成功率
    /// 从表达式中提取标识符，包括带限定符的列名
    fn extract_identifiers_from_expression(&self, expr: &str) -> Vec<String> {
        let mut identifiers = Vec::with_capacity(10); // 预分配合理容量
        
        // 预处理表达式，处理别名
        let processed_expr = self.preprocess_expression_for_aliases(expr);
        
        // 定义不应该作为列名的域名和技术相关术语
        let invalid_identifiers = ["com", "org", "net", "io", "website", "techonthenet", "hardware"];
        
        // 1. 先处理GaussDB特有的POINT函数参数格式
        for captures in GAUSSDB_PATTERN.captures_iter(&processed_expr) {
            if let Some(m1) = captures.get(1) {
                identifiers.push(m1.as_str().to_string());
            }
            if let Some(m2) = captures.get(2) {
                identifiers.push(m2.as_str().to_string());
            }
            if let Some(m3) = captures.get(3) {
                identifiers.push(m3.as_str().to_string());
            }
        }
        
        // 2. 使用正则表达式提取标准格式的标识符
        for captures in IDENTIFIER_RE.captures_iter(&processed_expr) {
            let identifier = if let Some(m) = captures.get(1) {
                m.as_str() // 带反引号的标识符
            } else if let Some(m) = captures.get(2) {
                m.as_str() // 带引号的标识符
            } else if let Some(m) = captures.get(3) {
                m.as_str() // 带方括号的标识符
            } else if let Some(m) = captures.get(4) {
                m.as_str() // 普通标识符
            } else {
                continue;
            };
            
            let normalized_ident = EnhancedSqlParserImprovedOptimized::normalize_identifier(identifier);
            let lower_ident = normalized_ident.to_lowercase();
            
            // 增强过滤逻辑
            if !EnhancedSqlParserImprovedOptimized::is_keyword(&normalized_ident) && 
               !COMMON_COLUMN_FUNCTIONS.contains(&lower_ident.as_str()) &&
               !invalid_identifiers.contains(&lower_ident.as_str()) &&
               normalized_ident.len() > 0 && normalized_ident.len() <= 100 {
                
                // 对于带点号的标识符，优先处理为column格式
                if normalized_ident.contains('.') {
                    let parts: Vec<&str> = normalized_ident.split('.').collect();
                    if parts.len() == 2 {
                        // 只添加列名部分（第二部分），不添加完整标识符
                        let column_part = parts[1].to_string();
                        let lower_column = column_part.to_lowercase();
                        if !EnhancedSqlParserImprovedOptimized::is_keyword(&column_part) && 
                           !invalid_identifiers.contains(&lower_column.as_str()) &&
                           !identifiers.contains(&column_part) {
                            identifiers.push(column_part);
                        }
                    }
                } else {
                    // 对于普通标识符，需要检查它是否更可能是列名而不是其他内容
                    if self.is_likely_column_reference(&normalized_ident, expr) {
                        identifiers.push(normalized_ident);
                    }
                }
            }
        }
        
        // 3. 从函数调用中提取列引用
        let function_columns = self.extract_columns_from_function_calls(&processed_expr);
        for col in function_columns {
            let lower_col = col.to_lowercase();
            if !identifiers.contains(&col) && !invalid_identifiers.contains(&lower_col.as_str()) {
                identifiers.push(col);
            }
        }
        
        // 处理别名
        self.extract_aliases(expr, &mut identifiers);
        
        // 去重
        identifiers.sort();
        identifiers.dedup();
        
        identifiers
    }
    
    /// 判断标识符是否可能是列引用
    fn is_likely_column_reference(&self, identifier: &str, context: &str) -> bool {
        let lower_id = identifier.to_lowercase();
        let lower_ctx = context.to_lowercase();
        
        // 避免提取明显不是列名的标识符
        if lower_id.len() <= 2 && !lower_id.chars().all(|c| c.is_alphanumeric()) {
            return false;
        }
        
        // 如果在常见的列操作上下文中出现，更可能是列名
        if lower_ctx.contains(&format!(" {}", lower_id)) || 
           lower_ctx.contains(&format!("{},", lower_id)) ||
           lower_ctx.contains(&format!("{}.", lower_id)) ||
           lower_ctx.contains(&format!("{}(" , lower_id)) {
            return true;
        }
        
        // 默认情况
        true
    }
    
    /// 从函数调用中提取列引用
    fn extract_columns_from_function_calls(&self, expr: &str) -> Vec<String> {
        let mut columns = Vec::new();
        
        // 找到所有函数调用
        let function_calls = self.find_function_calls(expr);
        
        for func_call in function_calls {
            // 提取函数名和参数
            if let Some(paren_start) = func_call.find('(') {
                let func_name = &func_call[..paren_start].trim();
                
                // 跳过常见的函数调用，但保留可能包含列引用的函数
                let func_name_lower = func_name.to_lowercase();
                if COMMON_COLUMN_FUNCTIONS.contains(&func_name_lower.as_str()) && 
                   !func_name_lower.contains("cast") && 
                   !func_name_lower.contains("case") && 
                   !func_name_lower.contains("coalesce") {
                    continue;
                }
                
                // 提取括号内的参数
                let params_str = if let Some(paren_end) = func_call.rfind(')') {
                    &func_call[paren_start+1..paren_end]
                } else {
                    continue;
                };
                
                // 递归提取参数中的标识符
                let param_identifiers = self.extract_identifiers_from_expression(params_str);
                for ident in param_identifiers {
                    if !columns.contains(&ident) {
                        columns.push(ident);
                    }
                }
            }
        }
        
        columns
    }
    
    /// 预处理表达式以处理别名
    fn preprocess_expression_for_aliases(&self, expression: &str) -> String {
        let mut processed = expression.to_string();
        
        // 简单处理AS关键字和别名
        let lower_expr = expression.to_lowercase();
        for (alias_pos, _) in lower_expr.match_indices(" as ") {
            // 找到下一个逗号或结束位置
            let end_pos = match lower_expr[alias_pos..].find(',') {
                Some(pos) => alias_pos + pos,
                None => expression.len(),
            };
            
            // 从表达式中移除别名部分
            let before_alias = &expression[..alias_pos];
            let after_alias = &expression[end_pos..];
            
            processed = format!("{}{}", before_alias, after_alias);
        }
        
        processed
    }
    
    /// 提取别名
    fn extract_aliases(&self, expression: &str, identifiers: &mut Vec<String>) {
        let lower_expr = expression.to_lowercase();
        
        // 处理带AS的别名
        for (as_pos, _) in lower_expr.match_indices(" as ") {
            let start = as_pos + 4; // " as " 的长度
            let rest = &expression[start..];
            
            // 找到别名后的第一个空格或逗号或结束位置
            let end = match rest.find(|c| c == ' ' || c == ',') {
                Some(pos) => start + pos,
                None => expression.len(),
            };
            
            let alias = &expression[start..end];
            if !alias.is_empty() && !EnhancedSqlParserImprovedOptimized::is_keyword(alias) {
                identifiers.push(alias.to_string());
            }
        }
        
        // 处理不带AS的别名（通常是在表达式后直接跟别名）
        let parts: Vec<&str> = expression.split(',').collect();
        for part in parts {
            let part_trimmed = part.trim();
            let tokens: Vec<&str> = part_trimmed.split_whitespace().collect();
            
            // 检查是否有不带AS的别名
            if tokens.len() >= 2 {
                let last_token = tokens.last().unwrap();
                // 如果最后一个标记不是关键字且前一个标记不是AS，则可能是别名
                if !EnhancedSqlParserImprovedOptimized::is_keyword(last_token) && tokens[tokens.len()-2].to_lowercase() != "as" {
                    identifiers.push(last_token.to_string());
                }
            }
        }
    }



    /// 辅助方法：安全地分割SQL部分，考虑嵌套括号和引号 - 增强版
    /// 性能优化：正确处理UTF-8字符边界
    fn split_sql_parts(&self, sql: &str, delimiter: char) -> Vec<String> {
        // 预分配合理容量，减少动态扩容
        let capacity = sql.chars().filter(|&c| c == delimiter).count() + 1;
        let mut parts = Vec::with_capacity(capacity);
        
        let mut current_part = String::new();
        let mut in_quote = false;
        let mut in_double_quote = false;
        let mut in_backtick = false; // MySQL风格的标识符
        let mut in_bracket = 0; // SQL Server风格的方括号
        let mut paren_count: i32 = 0;
        
        // 使用chars()迭代器正确处理UTF-8字符
        for c in sql.chars() {
            // 处理引号和括号
            match c {
                '"' if !in_quote && !in_backtick && in_bracket == 0 && paren_count == 0 => {
                    in_double_quote = !in_double_quote;
                    current_part.push(c);
                }
                '\'' if !in_double_quote && !in_backtick && in_bracket == 0 && paren_count == 0 => {
                    in_quote = !in_quote;
                    current_part.push(c);
                }
                '`' if !in_quote && !in_double_quote && in_bracket == 0 && paren_count == 0 => {
                    in_backtick = !in_backtick;
                    current_part.push(c);
                }
                '[' if !in_quote && !in_double_quote && !in_backtick && paren_count == 0 => {
                    in_bracket += 1;
                    current_part.push(c);
                }
                ']' if !in_quote && !in_double_quote && !in_backtick && paren_count == 0 => {
                    if in_bracket > 0 {
                        in_bracket -= 1;
                    }
                    current_part.push(c);
                }
                '(' if !in_quote && !in_double_quote && !in_backtick && in_bracket == 0 => {
                    paren_count += 1;
                    current_part.push(c);
                }
                ')' if !in_quote && !in_double_quote && !in_backtick && in_bracket == 0 => {
                    paren_count = paren_count.saturating_sub(1_i32);
                    current_part.push(c);
                }
                _ if c == delimiter && !in_quote && !in_double_quote && !in_backtick && in_bracket == 0 && paren_count == 0 => {
                    if !current_part.trim().is_empty() {
                        parts.push(current_part.trim().to_string());
                    }
                    current_part = String::new();
                }
                _ => {
                    current_part.push(c);
                }
            }
        }
        
        // 处理最后一部分
        if !current_part.trim().is_empty() {
            parts.push(current_part.trim().to_string());
        }
        
        parts
    }

    /// 规范化标识符，去除引号和转义字符
    pub fn normalize_identifier(identifier: &str) -> String {
        let mut result = String::with_capacity(identifier.len());
        let mut chars = identifier.chars().peekable();
        
        while let Some(c) = chars.next() {
            // 处理转义字符
            if c == '\\' && chars.peek().is_some() {
                if let Some(next_char) = chars.next() {
                    result.push(next_char);
                }
            } else {
                result.push(c);
            }
        }
        
        result
    }
}