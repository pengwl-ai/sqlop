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
    // SQL子句模式匹配
    static ref SQL_CLAUSE_PATTERN: Regex = Regex::new(r"(?i)\b(SELECT|FROM|WHERE|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER|JOIN|GROUP|ORDER|HAVING|LIMIT|OFFSET|TOP|DISTINCT)\b").unwrap();
    // SQL关键字模式匹配
    static ref SQL_KEYWORD_PATTERN: Regex = Regex::new(r"(?i)\b(WITH|WHERE|BY|GROUP|ORDER|FROM|JOIN|EXISTS|INNER|LEFT|RIGHT)\b").unwrap();
    // Schema模式匹配
    static ref SCHEMA_PATTERN: Regex = Regex::new(r"(?i)\b(public|private|internal|external|default|sys|system|temp|tempdb|information_schema)\b").unwrap();
    // 标识符模式匹配
    static ref IDENTIFIER_PATTERN: Regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    // 列名后缀模式匹配
    static ref COLUMN_SUFFIX_PATTERN: Regex = Regex::new(r"(?i)_(id|name|code|type|date|time|value|count|flag)$").unwrap();
    // 表名后缀模式匹配
    static ref TABLE_SUFFIX_PATTERN: Regex = Regex::new(r"(?i)s$|_table$|_data$|_info$").unwrap();
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
                     databases: &mut Vec<String>, 
                     schemas: &mut Vec<String>, 
                     tables: &mut Vec<String>, 
                     columns: &mut Vec<String>) -> bool {
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
        
        // 清理最终解析结果
        self.cleanup_result(databases, schemas, tables, columns);
        
        true
    }
    
    /// 解析SELECT语句
    fn parse_select_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 特殊处理SELECT *语句
        if lower_sql.contains("select *") || lower_sql.contains("select\t*") {
            columns.push("*".to_string());
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
    fn parse_insert_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                                    columns.push(identifier);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 解析UPDATE语句
    fn parse_update_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                        columns.push(identifier);
                    }
                }
            }
        }
        
        // 提取WHERE子句中的列名
        if let Some(where_pos) = lower_sql.find(" where ") {
            let where_clause = &sql[where_pos + 7..];
            let identifiers = self.extract_identifiers_from_expression(where_clause);
            for id in identifiers {
                columns.push(id);
            }
        }
    }
    
    /// 解析DELETE语句
    fn parse_delete_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                columns.push(id);
            }
        }
    }
    
    /// 解析CREATE语句
    fn parse_create_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                    databases.push(EnhancedSqlParserImprovedOptimized::normalize_identifier(&db_part));
                }
            }
        }
        // 处理CREATE SCHEMA
        else if lower_sql.contains("create schema") {
            if let Some(schema_pos) = lower_sql.find("create schema ") {
                let after_schema = &sql[schema_pos + 14..];
                let schema_part = self.extract_first_identifier(after_schema);
                if !schema_part.is_empty() {
                    schemas.push(EnhancedSqlParserImprovedOptimized::normalize_identifier(&schema_part));
                }
            }
        }
    }
    
    /// 解析ALTER语句
    fn parse_alter_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                                columns.push(EnhancedSqlParserImprovedOptimized::normalize_identifier(&col_name));
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// 解析DROP和TRUNCATE语句
    fn parse_drop_truncate_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, _columns: &mut Vec<String>) {
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
    fn parse_generic_statement(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        // 提取表引用
        self.extract_table_references(sql, tables, schemas, databases);
        // 尝试提取列引用
        self.extract_column_references(sql, columns);
    }
    
    /// 处理子查询 - 增强版，处理各种位置的子查询
    fn handle_subqueries(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        let lower_sql = sql.to_lowercase();
        let chars: Vec<char> = lower_sql.chars().collect();
        let char_count = chars.len();
        
        // 子查询开始标记模式
        let subquery_patterns = [
            // FROM 子句中的子查询
            "from (", 
            // IN 子句中的子查询
            "in (", 
            // EXISTS 子句中的子查询
            "exists (", 
            // 作为表达式一部分的子查询
            "= (", 
            "> (", 
            "< (", 
            ">= (", 
            "<= (", 
            // JOIN 子句中的子查询
            "join (", 
            "inner join (", 
            "left join (", 
            "right join (", 
            "full join ("
        ];
        
        // 遍历所有可能的子查询位置
        let mut i = 0;
        while char_count >= 5 && i < char_count - 5 { // 确保有足够字符进行比较
            // 检查是否匹配任何子查询模式
            let mut matched = false;
            
            for pattern in &subquery_patterns {
                let pattern_len = pattern.len();
                if i + pattern_len <= char_count {
                    let window = chars[i..i+pattern_len]
                        .iter()
                        .collect::<String>()
                        .to_lowercase();
                    
                    if window == *pattern {
                        // 找到子查询开始
                        let start_pos = i + pattern_len - 1; // 指向左括号位置
                        
                        // 转换字符索引为字节索引
                        let start_byte = chars[..start_pos].iter().map(|c| c.len_utf8()).sum();
                        
                        // 查找匹配的右括号
                        if let Some(end_byte) = self.find_matching_parenthesis(sql, start_byte) {
                            // 提取子查询（不包含括号）
                            let subquery = &sql[start_byte+1..end_byte];
                            
                            // 递归解析子查询
                            self.parse_sql(subquery, databases, schemas, tables, columns);
                            
                            // 跳过已处理的部分
                            i = chars[..end_byte+1].iter().map(|c| c.len_utf8()).sum();
                            matched = true;
                            break;
                        }
                    }
                }
            }
            
            if !matched {
                i += 1;
            }
        }
        
        // 直接调用handle_nested_subqueries处理复杂嵌套情况
        self.handle_nested_subqueries(sql, databases, schemas, tables, columns);
    }
    
    // 这个函数已在文件后面定义，这里不再重复
    
    /// 处理嵌套子查询 - 专门处理复杂嵌套情况
    fn handle_nested_subqueries(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        let chars: Vec<char> = sql.chars().collect();
        let mut paren_stack = Vec::new();
        let mut potential_subqueries = Vec::new();
        
        // 收集所有括号组
        for (i, c) in chars.iter().enumerate() {
            if *c == '(' {
                paren_stack.push(i);
            } else if *c == ')' && !paren_stack.is_empty() {
                let start = paren_stack.pop().unwrap();
                // 检查括号内容是否可能是SQL查询
                if i - start > 10 { // 至少包含一些基本SQL关键字
                    // 安全地获取括号内容，避免UTF-8字符边界问题
                    let content = sql.chars()
                        .skip(start + 1)
                        .take(i - start - 1)
                        .collect::<String>();
                    let lower_content = content.to_lowercase();
                    
                    // 检查是否包含基本SQL关键字组合
                    if (lower_content.contains("select") && lower_content.contains("from")) ||
                       (lower_content.contains("with") && (lower_content.contains("select") || lower_content.contains("as")))
                    {
                        potential_subqueries.push((start, i));
                    }
                }
            }
        }
        
        // 解析潜在的子查询（从最内层开始，避免重复处理）
        potential_subqueries.sort_by(|a, b| b.0.cmp(&a.0)); // 按起始位置降序排序
        
        for (start, end) in potential_subqueries {
            // 安全地获取子查询内容，避免UTF-8字符边界问题
            let subquery = sql.chars()
                .skip(start + 1)
                .take(end - start - 1)
                .collect::<String>();
            self.parse_sql(&subquery, databases, schemas, tables, columns);
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
    fn extract_columns_from_create_table(&self, sql: &str, columns: &mut Vec<String>) {
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
                                        // 避免重复添加，保持顺序
                                    if !columns.contains(&col_name) {
                                        columns.push(col_name);
                                    }
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
    fn handle_with_statements(&self, sql: &str, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        let lower_sql = sql.to_lowercase();
        
        // 检查是否包含WITH子句（支持"with "和" with "两种格式）
        if let Some(with_pos) = lower_sql.find("with ").or_else(|| lower_sql.find(" with ")) {
            // 查找主查询的开始位置，考虑SELECT、INSERT、UPDATE、DELETE等
            if let Some(main_query_start) = self.find_main_query_start(&lower_sql, with_pos) {
                let cte_section = &sql[with_pos + 5..main_query_start].trim();
                
                // 分割多个CTE定义，考虑括号嵌套情况
                let cte_definitions = self.split_cte_definitions(cte_section);
                
                for cte_def in cte_definitions {
                    if let Some((cte_name, cte_columns, cte_body)) = self.parse_single_cte(&cte_def) {
                        // 添加CTE名称到表集合
                        if !cte_name.is_empty() {
                            tables.push(cte_name);
                        }
                        
                        // 添加CTE列定义到列集合
                        for col in cte_columns {
                            if !col.is_empty() {
                                columns.push(col);
                            }
                        }
                        
                        // 递归解析CTE内部的SQL
                        if !cte_body.is_empty() {
                            self.parse_sql(&cte_body, databases, schemas, tables, columns);
                        }
                    }
                }
            }
        }
    }
    
    // 查找主查询的开始位置
    fn find_main_query_start(&self, lower_sql: &str, with_pos: usize) -> Option<usize> {
        // 查找主查询开始的位置，可以是SELECT、INSERT、UPDATE、DELETE等关键字
        let main_query_patterns = ["select ", "insert ", "update ", "delete ", "create ", "alter ", "drop "];
        
        let mut min_pos = None;
        for pattern in &main_query_patterns {
            if let Some(pos) = lower_sql[with_pos..].find(pattern) {
                let absolute_pos = with_pos + pos;
                match min_pos {
                    Some(current_min) if absolute_pos < current_min => min_pos = Some(absolute_pos),
                    None => min_pos = Some(absolute_pos),
                    _ => {}
                }
            }
        }
        
        min_pos
    }
    
    // 分割多个CTE定义，考虑括号嵌套
    fn split_cte_definitions(&self, cte_section: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut start = 0;
        let mut depth = 0usize; // 显式声明为usize类型
        
        for (i, c) in cte_section.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth = if depth > 0 { depth - 1 } else { 0 },
                ',' if depth == 0 => {
                    // 只有在顶级括号层级才能分割CTE定义
                    result.push(cte_section[start..i].trim().to_string());
                    start = i + 1;
                },
                _ => {}
            }
        }
        
        // 添加最后一个CTE定义
        if start < cte_section.len() {
            result.push(cte_section[start..].trim().to_string());
        }
        
        result
    }
    
    // 解析单个CTE定义，返回(CTE名称, CTE列定义, CTE主体SQL)
    fn parse_single_cte(&self, cte_def: &str) -> Option<(String, Vec<String>, String)> {
        // 支持多种AS关键字格式
        let lower_cte = cte_def.to_lowercase();
        let as_pos = lower_cte.find(" as ").or_else(|| lower_cte.find("as "))?;
        
        // 提取CTE名称部分（包括可能的列定义）
        let name_part = cte_def[..as_pos].trim();
        
        // 解析CTE名称和列定义
        let (cte_name, cte_columns) = self.parse_cte_name_and_columns(name_part);
        
        // 提取并解析CTE主体
        let body_start = as_pos + (if lower_cte[as_pos..].starts_with(" as ") { 4 } else { 3 });
        let cte_body = cte_def[body_start..].trim();
        
        // 处理CTE主体（可能有括号包围）
        let inner_body = if cte_body.starts_with('(') {
            match self.find_matching_parenthesis(cte_def, body_start) {
                Some(right_paren_pos) => {
                    if right_paren_pos > body_start + 1 {
                        cte_def[body_start + 1..right_paren_pos].trim().to_string()
                    } else {
                        "".to_string()
                    }
                },
                None => cte_body.to_string() // 如果没有匹配的右括号，就使用整个部分
            }
        } else {
            cte_body.to_string()
        };
        
        Some((cte_name, cte_columns, inner_body))
    }
    
    // 解析CTE名称和列定义
    fn parse_cte_name_and_columns(&self, name_part: &str) -> (String, Vec<String>) {
        let mut cte_name = "".to_string();
        let mut cte_columns = Vec::new();
        
        // 尝试找到列定义的左括号
        if let Some(left_paren_pos) = name_part.find('(') {
            // 提取CTE名称
            cte_name = name_part[..left_paren_pos].trim().to_string();
            
            // 提取列定义
            if let Some(right_paren_pos) = self.find_matching_parenthesis(name_part, left_paren_pos) {
                let columns_str = &name_part[left_paren_pos + 1..right_paren_pos];
                let column_parts = self.split_sql_parts(columns_str, ',');
                
                for col in column_parts {
                    let trimmed_col = col.trim();
                    if !trimmed_col.is_empty() {
                        cte_columns.push(EnhancedSqlParserImprovedOptimized::normalize_identifier(&trimmed_col));
                    }
                }
            }
        } else {
            // 简单情况：只有CTE名称
            cte_name = name_part.trim().to_string();
        }
        
        (cte_name, cte_columns)
    }

    fn cleanup_result(&self, databases: &mut Vec<String>, schemas: &mut Vec<String>, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        // 保存原始表集合，用于后续分析
        let original_tables = tables.clone();
        
        // 创建临时集合用于去重和快速查找
        let mut column_set = HashSet::new();
        let mut table_set = HashSet::new();
        let mut schema_set = HashSet::new();
        
        // 1. 清理列集合 - 更宽松的过滤规则，并保持顺序
        let mut clean_columns = Vec::new();
        for col in columns.iter() {
            if col == "*" && !column_set.contains("*") {
                clean_columns.push(col.clone());
                column_set.insert("*".to_string());
                continue;
            }
            
            let trimmed = col.trim();
            // 更宽松的过滤：只移除明显无效的项
            if trimmed.is_empty() || 
               trimmed.parse::<f64>().is_ok() || // 纯数字
               trimmed.contains(';') || // 分号通常表示语句结束
               // 保留可能包含点号的标识符（例如 table.column 形式）
               (trimmed.contains('.') && !self.is_likely_table_column_reference(trimmed)) {
                continue;
            }
            
            // 更宽松的关键字过滤：仅过滤明显的SQL语句关键字
            let lower_trimmed = trimmed.to_lowercase();
            let is_essential_keyword = matches!(lower_trimmed.as_str(), 
                "select" | "from" | "where" | "insert" | "update" | "delete" | 
                "create" | "drop" | "alter" | "with" | "group" | "order" | 
                "having" | "limit" | "offset" | "join" | "on" | "as" | "into" | 
                "values" | "table" | "view" | "index" | "by" | "set");
            
            if is_essential_keyword {
                continue;
            }
            
            // 移除字符串字面量（更准确的检测）
            if (trimmed.starts_with("'") && trimmed.ends_with("'") && trimmed.len() > 1) ||
               (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1) ||
               (trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() > 1) ||
               (trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() > 1) {
                continue;
            }
            
            // 避免重复添加
            if !column_set.contains(trimmed) {
                clean_columns.push(trimmed.to_string());
                column_set.insert(trimmed.to_string());
            }
        }
        
        // 2. 清理表集合 - 更宽松的规则，提高表名识别率
        let mut clean_tables = Vec::new();
        for table in tables.iter() {
            let trimmed = table.trim();
            
            // 最小过滤：只移除空字符串和明显不是表名的项
            if trimmed.is_empty() {
                continue;
            }
            
            // 更宽松的关键字检测：仅过滤明显不可能是表名的关键字
            let lower_table = trimmed.to_lowercase();
            let is_definitely_not_table = matches!(lower_table.as_str(),
                "select" | "from" | "where" | "and" | "or" | "not" | "join" | "on" |
                "group" | "order" | "having" | "limit" | "offset" | "distinct" |
                "as" | "insert" | "update" | "delete" | "into" | "values" |
                "set" | "create" | "drop" | "alter" | "with" | "by");
            
            if is_definitely_not_table {
                continue;
            }
            
            // 表名特征检测 - 保留更多可能性
            let is_table_like = 
                lower_table.ends_with("s") ||  // 宽松：大多数复数形式可能是表名
                lower_table.ends_with("_table") ||
                lower_table.ends_with("_view") ||
                lower_table.ends_with("_data") ||
                lower_table.ends_with("_info") ||
                lower_table.contains("_");  // 包含下划线的标识符更可能是表名
            
            // 列名模式检查 - 平衡的规则
            let is_column_pattern = 
                lower_table.ends_with("_id") || 
                lower_table.ends_with("_name") ||
                lower_table.starts_with("is_") ||
                lower_table.starts_with("has_") ||
                // 关键的常见列名过滤
                ["website", "status", "type", "code", "date", "time", "value", "count", 
                 "flag", "name", "id"].contains(&lower_table.as_str()) ||
                // 单字母或双小写字母通常是别名而非表名
                (lower_table.len() == 1 && lower_table.chars().all(|c| c.is_ascii_lowercase())) ||
                (lower_table.len() == 2 && lower_table.chars().all(|c| c.is_ascii_lowercase())) ||
                // 常见的CTE临时表名过滤
                lower_table == "temp" || lower_table == "tmp" || lower_table == "cte";
            
            // 如果是明显的列名模式，无论是否在列集合中都排除
            if is_column_pattern {
                continue;
            }
            
            // 更宽松的保留策略：除非确定是列名，否则都尝试保留
            if !table_set.contains(trimmed) {
                clean_tables.push(trimmed.to_string());
                table_set.insert(trimmed.to_string());
            }
        }
        
        // 3. 处理表名和列名重叠问题 - 更智能的判断
        let overlapping_items: Vec<String> = table_set
            .intersection(&column_set)
            .cloned()
            .collect();
        
        for item in overlapping_items {
            // 在重叠情况下，基于上下文智能判断
            let lower_item = item.to_lowercase();
            
            // 判断是否明显是列名
            let is_obviously_column = 
                lower_item.ends_with("_id") || 
                lower_item.ends_with("_name") ||
                lower_item.starts_with("is_") ||
                lower_item.starts_with("has_") ||
                ["id", "name", "type"].contains(&lower_item.as_str());
            
            // 判断是否明显是表名
            let should_be_table = 
                (lower_item.ends_with("s") && !lower_item.ends_with("ss") && 
                !lower_item.ends_with("ous") && !lower_item.ends_with("us")) ||
                original_tables.contains(&item);
            
            // 智能决策：如果有强表名特征，保留为表名；否则保留为列名
            if should_be_table && !is_obviously_column {
                // 如果应该是表名，从列集合中移除
                if let Some(pos) = clean_columns.iter().position(|x| x == &item) {
                    clean_columns.remove(pos);
                }
                column_set.remove(&item);
            } else if is_obviously_column {
                // 如果明显是列名，从表集合中移除
                if let Some(pos) = clean_tables.iter().position(|x| x == &item) {
                    clean_tables.remove(pos);
                }
                table_set.remove(&item);
            }
            // 对于边界情况，保留在两个集合中
        }
        
        // 4. 清理schema集合 - 保留更多可能的schema
        let mut clean_schemas = Vec::new();
        // 扩展常见schema列表
        let common_schemas = ["public", "private", "internal", "external", "default", "sys", "system", 
                             "temp", "tempdb", "information_schema", "dbo", "pg_catalog", 
                             "mysql", "performance_schema", "sysibm", "syscat", "sysdba"];
        
        for schema in schemas.iter() {
            let lower_schema = schema.to_lowercase();
            // 保留常见schema名称，且不在表集合中（允许与列集合重叠）
            if common_schemas.contains(&lower_schema.as_str()) && 
               !table_set.contains(schema) &&
               !schema_set.contains(schema) {
                clean_schemas.push(schema.clone());
                schema_set.insert(schema.clone());
            }
        }
        
        // 5. 保留非空的数据库集合，不盲目清空
        if !databases.is_empty() {
            let mut unique_databases = Vec::new();
            let mut db_set = HashSet::new();
            for db in databases.iter() {
                let trimmed = db.trim();
                if !trimmed.is_empty() && !db_set.contains(trimmed) && 
                   !table_set.contains(trimmed) && !column_set.contains(trimmed) {
                    unique_databases.push(trimmed.to_string());
                    db_set.insert(trimmed.to_string());
                }
            }
            *databases = unique_databases;
        }
        
        // 6. 如果没有列但有表，添加通配符
        if clean_columns.is_empty() && !clean_tables.is_empty() {
            clean_columns.push("*".to_string());
        }
        
        // 更新原始集合
        *columns = clean_columns;
        *tables = clean_tables;
        *schemas = clean_schemas;
    }
    
    // 判断字符串是否可能是 table.column 格式的引用
     fn is_likely_table_column_reference(&self, identifier: &str) -> bool {
         // 简单检查：只包含一个点号且前后都有字符
         let parts: Vec<&str> = identifier.split('.').collect();
         if parts.len() == 2 {
             let table_part = parts[0].trim();
             let column_part = parts[1].trim();
             // 确保两部分都不为空
             return !table_part.is_empty() && !column_part.is_empty();
         }
         false
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
    fn extract_table_references(&self, sql: &str, tables: &mut Vec<String>, schemas: &mut Vec<String>, databases: &mut Vec<String>) {
        // 查找FROM、JOIN、INTO等关键字后的表名
        let lower_sql = sql.to_lowercase();
        
        // 处理WITH子句中的CTE和引用的表
        self.handle_with_clauses(sql, tables, schemas, databases);
        
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
        
        // 处理CREATE TABLE AS SELECT语句
        if let Some(create_pos) = lower_sql.find("create table ") {
            let after_create = &lower_sql[create_pos + 13..];
            if let Some(as_pos) = after_create.find(" as ") {
                let table_name_part = &sql[create_pos + 13..create_pos + 13 + as_pos].trim();
                self.extract_single_table_name(table_name_part, tables);
                
                // 处理AS SELECT后面的表名
                let select_part = &sql[create_pos + 13 + as_pos + 4..];
                self.extract_table_references_from_select(select_part, tables, schemas, databases);
            }
        }
        
        // 直接从整个SQL中提取可能的表名（作为补充机制）
        self.extract_tables_from_whole_sql(sql, tables, schemas, databases);
    }
    
    /// 处理WITH子句中的CTE和引用的表
    fn handle_with_clauses(&self, sql: &str, tables: &mut Vec<String>, schemas: &mut Vec<String>, databases: &mut Vec<String>) {
        let lower_sql = sql.to_lowercase();
        
        if let Some(with_pos) = lower_sql.find("with ") {
            // 找到第一个WITH关键字后的第一个SELECT
            if let Some(main_select_pos) = lower_sql[with_pos..].find("select ") {
                let cte_part = &sql[with_pos + 5..with_pos + main_select_pos].trim();
                
                // 处理每个CTE定义
                let cte_definitions = self.split_sql_parts(cte_part, ',');
                for cte_def in cte_definitions {
                    let parts: Vec<&str> = cte_def.trim().splitn(2, |c: char| c == '(' || c.is_whitespace()).collect();
                    if let Some(_cte_name) = parts.first() {
                        // 提取CTE中引用的表名
                        if let Some(select_start) = cte_def.find('(') {
                            let inner_sql = &cte_def[select_start + 1..];
                            self.extract_table_references_from_select(inner_sql, tables, schemas, databases);
                        }
                    }
                }
            }
        }
    }
    
    /// 从SELECT语句中提取表引用
    fn extract_table_references_from_select(&self, sql: &str, tables: &mut Vec<String>, schemas: &mut Vec<String>, databases: &mut Vec<String>) {
        // 递归提取嵌套SELECT中的表
        let lower_select = sql.to_lowercase();
        
        // 处理FROM子句
        if let Some(from_pos) = lower_select.find(" from ") {
            let from_part = &sql[from_pos + 6..];
            self.extract_tables_from_clause(from_part, tables, schemas, databases);
        }
        
        // 处理JOIN子句
        for (i, _) in lower_select.match_indices(" join ") {
            let start_pos = i + 6;
            let join_part = &sql[start_pos..];
            self.extract_tables_from_clause(join_part, tables, schemas, databases);
        }
    }
    
    /// 从整个SQL中提取可能的表名（补充机制）
    fn extract_tables_from_whole_sql(&self, sql: &str, tables: &mut Vec<String>, _schemas: &mut Vec<String>, _databases: &mut Vec<String>) {
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
    fn extract_single_table_name(&self, table_str: &str, tables: &mut Vec<String>) {
        if table_str.is_empty() {
            return;
        }
        
        // 移除别名部分
        let table_name = if let Some(as_pos) = table_str.to_lowercase().find(" as ") {
            table_str[..as_pos].trim()
        } else if let Some(space_pos) = table_str.find(|c: char| c.is_whitespace()) {
            let clean_table = table_str[..space_pos].trim();
            let alias_part = table_str[space_pos+1..].trim();
            
            // 如果后面的部分是SQL关键字或包含特殊字符，则认为前面是表名
            if !alias_part.is_empty() && 
               (SQL_KEYWORDS.contains(alias_part.to_lowercase().as_str()) || 
                alias_part.starts_with('(') || 
                alias_part.contains('=') || 
                alias_part.contains(',')) {
                clean_table
            } else {
                clean_table
            }
        } else {
            table_str
        };
        
        // 移除可能的括号
        let table_name = table_name.trim_matches(|c| c == '(' || c == ')');
        
        // 处理带引号的标识符
        let table_name = if (table_name.starts_with('"') && table_name.ends_with('"')) || 
                           (table_name.starts_with('`') && table_name.ends_with('`')) ||
                           (table_name.starts_with('[') && table_name.ends_with(']')) {
            &table_name[1..table_name.len()-1]
        } else {
            table_name
        };
        
        // 更严格的表名过滤逻辑
        if !table_name.is_empty() && 
           // 过滤掉纯数字
           !table_name.chars().all(|c| c.is_numeric()) &&
           // 过滤掉字符串字面量
           !table_name.starts_with('\'') && !table_name.ends_with('\'') &&
           // 过滤掉单个特殊字符（如'=', ')', '('等）
           table_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-') &&
           // 过滤掉SQL关键字（除非用引号括起来，但我们已经处理了引号）
           !SQL_KEYWORDS.contains(table_name.to_lowercase().as_str()) &&
           // 过滤掉常见的列名后缀
           !table_name.to_lowercase().ends_with("_id") &&
           !table_name.to_lowercase().ends_with("_cd") &&
           !table_name.to_lowercase().ends_with("_name") &&
           !table_name.to_lowercase().ends_with("_type") &&
           !table_name.to_lowercase().ends_with("_status") &&
           !table_name.to_lowercase().starts_with("is_") &&
           !table_name.to_lowercase().starts_with("has_") &&
           // 过滤掉常见的列名
           table_name.to_lowercase() != "id" &&
           table_name.to_lowercase() != "status" &&
           table_name.to_lowercase() != "name" &&
           // 过滤掉长度过短的标识符（通常表名不会是单个字符）
           (table_name.len() > 1 || table_name.chars().all(|c| c.is_uppercase())) {
            
            // 如果包含点，分别验证每个部分
            if table_name.contains('.') {
                let parts: Vec<&str> = table_name.split('.').filter(|s| !s.is_empty()).collect();
                // 确保所有部分都不是关键字或纯数字
                if parts.iter().all(|&p| 
                    !SQL_KEYWORDS.contains(p.to_lowercase().as_str()) && 
                    !p.chars().all(|c| c.is_numeric())
                ) {
                    // 避免重复添加，保持顺序
                    if !tables.contains(&table_name.to_string()) {
                        tables.push(table_name.to_string());
                    }
                }
            } else {
                // 避免重复添加，保持顺序
                if !tables.contains(&table_name.to_string()) {
                    tables.push(table_name.to_string());
                }
            }
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
                if let Some(_table_pos) = lower_sql[pos..].find(&lower_candidate) {
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
    fn extract_tables_from_clause(&self, clause: &str, tables: &mut Vec<String>, schemas: &mut Vec<String>, databases: &mut Vec<String>) {
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

    /// 判断标识符是否有效（用于表名、schema名、数据库名）
    fn is_valid_identifier(&self, identifier: &str) -> bool {
        // 基本检查
        if identifier.is_empty() {
            return false;
        }
        
        // 过滤掉纯数字
        if identifier.chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // 过滤掉特殊字符（只允许字母、数字、下划线、点和连字符）
        if !identifier.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == '-') {
            return false;
        }
        
        // 过滤掉SQL关键字
        if SQL_KEYWORDS.contains(identifier.to_lowercase().as_str()) {
            return false;
        }
        
        // 过滤掉长度过短的标识符（除非全部是大写）
        if identifier.len() == 1 && !identifier.chars().all(|c| c.is_uppercase()) {
            return false;
        }
        
        true
    }
    
    /// 解析表标识符，处理database.schema.table格式
    fn parse_table_identifier(&self, identifier: &str, tables: &mut Vec<String>, schemas: &mut Vec<String>, databases: &mut Vec<String>) {
        // 按点分割并过滤空部分
        let parts: Vec<&str> = identifier.split('.')
            .filter(|s| !s.is_empty())
            .collect();
        
        // 基本验证：确保至少有一个非空部分且标识符看起来有效
        if parts.is_empty() || !self.is_valid_identifier(&identifier.replace('.', "_")) {
            return;
        }
        
        match parts.len() {
            1 => {
                // 只有表名
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                // 更严格的过滤
                if self.is_valid_identifier(&table) && 
                   !COMMON_SCHEMA_NAMES.contains(&table.to_lowercase().as_str()) &&
                   // 额外检查：不是常见的列名模式
                   !table.to_lowercase().ends_with("_id") &&
                   !table.to_lowercase().ends_with("_cd") {
                    tables.push(table);
                }
            },
            2 => {
                // schema.table 或 database.table
                let first_part = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let second_part = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                let first_part_lower = first_part.to_lowercase();
                
                // 首先确保两个部分都有效
                if !self.is_valid_identifier(&first_part) || !self.is_valid_identifier(&second_part) {
                    return;
                }
                
                // 使用通用schema名称列表进行智能判断
                if COMMON_SCHEMA_NAMES.contains(&first_part_lower.as_str()) || 
                   first_part_lower == "schema" || 
                   first_part_lower.len() <= 4 || 
                   first_part_lower.ends_with("schema") {
                    // 很可能是schema名
                    schemas.push(first_part);
                    // 额外检查：第二部分不是常见的列名模式
                    if !second_part.to_lowercase().ends_with("_id") &&
                       !second_part.to_lowercase().ends_with("_cd") {
                        tables.push(second_part);
                    }
                } else if first_part_lower.len() > 8 && !first_part_lower.ends_with("s") {
                    // 可能是数据库名
                    databases.push(first_part);
                    // 额外检查：第二部分不是常见的列名模式
                    if !second_part.to_lowercase().ends_with("_id") &&
                       !second_part.to_lowercase().ends_with("_cd") {
                        tables.push(second_part);
                    }
                } else {
                    // 默认策略：较短的可能是schema
                    if first_part.len() <= second_part.len() {
                        schemas.push(first_part);
                        tables.push(second_part);
                    } else {
                        // 较长的可能是数据库名
                        databases.push(first_part);
                        tables.push(second_part);
                    }
                }
            },
            3 => {
                // database.schema.table
                let database = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let schema = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[2]);
                
                // 确保所有部分都有效
                if self.is_valid_identifier(&database) && 
                   self.is_valid_identifier(&schema) && 
                   self.is_valid_identifier(&table) {
                    databases.push(database);
                    schemas.push(schema);
                    // 额外检查：表名不是常见的列名模式
                    if !table.to_lowercase().ends_with("_id") &&
                       !table.to_lowercase().ends_with("_cd") {
                        tables.push(table);
                    }
                }
            },
            _ => {
                // 处理复杂情况，尝试智能解析
                if parts.len() >= 2 {
                    // 对于多部分标识符，确保最后两部分都有效
                    let part_n_minus_1 = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[parts.len()-2]);
                    let part_n = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[parts.len()-1]);
                    
                    if self.is_valid_identifier(&part_n_minus_1) && self.is_valid_identifier(&part_n) {
                        // 组合最后两部分作为schema.表名
                        let last_two = format!("{}.{}", part_n_minus_1, part_n);
                        tables.push(last_two);
                    }
                } else if let Some(table_part) = parts.last() {
                    let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(table_part);
                    if self.is_valid_identifier(&table) {
                        tables.push(table);
                    }
                }
            }
        }
    }

    /// 提取列引用 - 增强版（简化版，仅提取SELECT子句中的列）
    fn extract_column_references(&self, sql: &str, columns: &mut Vec<String>) {
        // 查找SELECT关键字后的列名
        let lower_sql = sql.to_lowercase();
        let has_select_star = lower_sql.contains("select *") || lower_sql.contains("select\t*");
        
        // 特殊处理：如果SELECT子句中有星号，确保它被添加到列名集合中
        if has_select_star {
            columns.push("*".to_string());
            return;
        }
        
        // 仅处理SELECT子句（包括嵌套查询中的SELECT）
        for (select_start, _) in lower_sql.match_indices("select ") {
            // 查找FROM、SET、INTO等关键字作为结束位置
            let end_pos = self.find_clause_end(&lower_sql, select_start + 7);
            let select_clause = &sql[select_start + 7..end_pos];
            
            self.extract_columns_from_select(select_clause, columns);
        }
        
        // 对于UPDATE语句，处理SET子句中的列引用
        if lower_sql.contains("update") && lower_sql.contains("set") {
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
                            columns.push(identifier);
                        }
                    }
                }
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
                            columns.push(col_name);
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
    fn extract_columns_from_select(&self, select_clause: &str, columns: &mut Vec<String>) {
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
                columns.push(id);
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
    fn handle_function_calls(&self, sql: &str, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
                        columns.push(id);
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
                        columns.push(id);
                    }
                }
            }
            // 处理CASE WHEN表达式
            else if lower_call.starts_with("case") {
                // 提取CASE WHEN中的列名
                let identifiers = self.extract_identifiers_from_expression(&call);
                for id in identifiers {
                    columns.push(id);
                }
            }
            // 处理CAST函数和类型转换
            else if lower_call.starts_with("cast(") || lower_call.starts_with("convert(") {
                // 提取CAST/CONVERT函数中的列名
                let params = self.extract_function_params(&call[call.find('(').unwrap() + 1..call.rfind(')').unwrap_or(call.len())]);
                for param in params {
                    let identifiers = self.extract_identifiers_from_expression(&param);
                    for id in identifiers {
                        columns.push(id);
                    }
                }
            }
            // 处理LIKE和ILIKE表达式
            else if lower_call.contains(" like ") || lower_call.contains(" ilike ") {
                // 提取LIKE/ILIKE左侧的列名
                let identifiers = self.extract_identifiers_from_expression(&call);
                for id in identifiers {
                    columns.push(id);
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
    fn extract_from_geo_functions(&self, function_call: &str, _tables: &mut Vec<String>, columns: &mut Vec<String>) {
        // 提取函数参数
        if let Some(start) = function_call.find('(') {
            if let Some(end) = function_call.rfind(')') {
                let params = &function_call[start + 1..end];
                let identifiers = self.extract_identifiers_from_expression(params);
                
                for id in identifiers {
                    columns.push(id);
                }
            }
        }
    }

    /// 处理方言特定的内容
    fn handle_dialect_specific(&self, sql: &str, dialect: &str, tables: &mut Vec<String>, columns: &mut Vec<String>) {
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
    fn handle_gaussdb_specific(&self, sql: &str, tables: &mut Vec<String>, columns: &mut Vec<String>) {
        // 处理GaussDB特有的POINT函数参数格式
        for captures in GAUSSDB_PATTERN.captures_iter(sql) {
            if let Some(m1) = captures.get(1) {
                tables.push(m1.as_str().to_string());
            }
            if let Some(m2) = captures.get(2) {
                tables.push(m2.as_str().to_string());
            }
            if let Some(m3) = captures.get(3) {
                columns.push(m3.as_str().to_string());
            }
        }
    }

    /// 处理Oracle特定的内容
    fn handle_oracle_specific(&self, _sql: &str, _tables: &mut Vec<String>, _columns: &mut Vec<String>) {
        // Oracle特定的处理逻辑
    }

    /// 处理MySQL特定的内容
    fn handle_mysql_specific(&self, _sql: &str, _tables: &mut Vec<String>, _columns: &mut Vec<String>) {
        // MySQL特定的处理逻辑
    }

    /// 处理PostgreSQL特定的内容
    fn handle_postgresql_specific(&self, _sql: &str, _tables: &mut Vec<String>, _columns: &mut Vec<String>) {
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
        
        // 去重但保持原始顺序
        let mut seen = HashSet::new();
        identifiers.retain(|id| seen.insert(id.clone()));
        
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