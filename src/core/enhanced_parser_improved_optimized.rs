use std::collections::HashSet;
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;


// 预编译正则表达式，使用lazy_static避免重复编译
lazy_static! {
    // GaussDB特有的POINT函数参数格式
    static ref GAUSSDB_PATTERN: Regex = Regex::new(r"\(([A-Z0-9]+)\s+([A-Z0-9]+)\s+([A-Z0-9]+)\)").unwrap();
    // 标准格式的标识符提取
    static ref IDENTIFIER_RE: Regex = Regex::new(r#"(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|\b([a-zA-Z0-9_]+)\b)"#).unwrap();
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
    // 常用SQL模式匹配
    static ref SQL_TYPE_PATTERNS: Vec<(&'static str, &'static str)> = {
        vec![
            ("^select", "SELECT"),
            ("^insert", "INSERT"),
            ("^update", "UPDATE"),
            ("^delete", "DELETE"),
            ("^create", "CREATE"),
            ("^drop", "DROP"),
            ("^alter", "ALTER"),
            ("^truncate", "TRUNCATE"),
        ]
    };
}

/// 高性能SQL解析器改进版
pub struct EnhancedSqlParserImprovedOptimized {
    dialect: Option<String>,
}

impl EnhancedSqlParserImprovedOptimized {
    /// 创建新的解析器实例
    pub fn new(dialect: Option<String>) -> Self {
        Self {
            dialect,
        }
    }

    /// 解析SQL，提取数据库对象信息 - 增强版
    /// 优化：整合改进并优化解析流程，更好地处理复杂SQL语句
    pub fn parse_sql(&self, 
                     sql: &str, 
                     databases: &mut HashSet<String>, 
                     schemas: &mut HashSet<String>, 
                     tables: &mut HashSet<String>, 
                     columns: &mut HashSet<String>) -> bool {
        // 预处理SQL
        let preprocessed_sql = self.preprocess_sql(sql);
        
        // 1. 首先处理方言特定内容（可能包含特殊语法）
        if let Some(dialect) = &self.dialect {
            self.handle_dialect_specific(&preprocessed_sql, dialect, tables, columns);
        }
        
        // 2. 提取表引用（支持复杂的JOIN、子查询等）
        self.extract_table_references(&preprocessed_sql, tables, schemas, databases);
        
        // 3. 提取列引用（支持各种子句和复杂表达式）
        self.extract_column_references(&preprocessed_sql, columns);
        
        // 4. 处理函数调用（支持多种函数类型）
        self.handle_function_calls(&preprocessed_sql, tables, columns);
        
        // 5. 处理子查询
        self.handle_subqueries(&preprocessed_sql, databases, schemas, tables, columns);
        
        // 6. 清理和优化结果
        self.cleanup_result(databases, schemas, tables, columns);
        
        true
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
            if Self::is_keyword(trimmed) {
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
            !id.to_uppercase().contains("RIGHT")
        });
        
        // 对列名应用额外过滤，避免无效标识符
        columns.retain(|id| {
            filter_identifier(id) && 
            // 避免列名中包含无效字符组合
            !id.contains(")") && // 避免类似 "s)" 这样的无效列名
            !id.contains("))") && // 避免类似 "products))" 这样的无效列名
            // 避免过于简单的列名被误识别
            (id.trim().len() > 1 || 
             (id.trim().len() == 1 && id.chars().all(|c| c.is_alphabetic() && !c.is_ascii_uppercase())))
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
        let parts: Vec<&str> = identifier.split('.').collect();
        
        match parts.len() {
            1 => {
                // 只有表名
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                tables.insert(table);
            },
            2 => {
                // schema.table
                let schema = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                schemas.insert(schema);
                tables.insert(table);
            },
            3 => {
                // database.schema.table
                let database = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[0]);
                let schema = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[1]);
                let table = EnhancedSqlParserImprovedOptimized::normalize_identifier(parts[2]);
                databases.insert(database);
                schemas.insert(schema);
                tables.insert(table);
            },
            _ => {
                // 格式复杂，取最后一个部分作为表名
                if let Some(table_part) = parts.last() {
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
        
        // 处理SELECT子句（包括嵌套查询中的SELECT）
        for (select_start, _) in lower_sql.match_indices("select ") {
            // 查找FROM、SET、INTO等关键字作为结束位置
            let end_pos = self.find_clause_end(&lower_sql, select_start + 7);
            let select_clause = &sql[select_start + 7..end_pos];
            
            self.extract_columns_from_select(select_clause, columns);
        }
        
        // 处理WHERE子句中的列引用
        for (where_start, _) in lower_sql.match_indices(" where ") {
            let where_clause = &sql[where_start + 7..];
            let identifiers = self.extract_identifiers_from_expression(where_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理SET子句中的列引用
        for (set_start, _) in lower_sql.match_indices(" set ") {
            let set_clause = &sql[set_start + 5..];
            let identifiers = self.extract_identifiers_from_expression(set_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理HAVING子句中的列引用
        for (having_start, _) in lower_sql.match_indices(" having ") {
            let having_clause = &sql[having_start + 7..];
            let identifiers = self.extract_identifiers_from_expression(having_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理ORDER BY子句中的列引用
        for (order_start, _) in lower_sql.match_indices(" order by ") {
            let order_clause = &sql[order_start + 9..];
            let identifiers = self.extract_identifiers_from_expression(order_clause);
            for id in identifiers {
                columns.insert(id);
            }
        }
        
        // 处理GROUP BY子句中的列引用
        for (group_start, _) in lower_sql.match_indices(" group by ") {
            let group_clause = &sql[group_start + 9..];
            let identifiers = self.extract_identifiers_from_expression(group_clause);
            for id in identifiers {
                columns.insert(id);
            }
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
                        if !Self::is_keyword(&function_name) {
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
    fn is_keyword(s: &str) -> bool {
        SQL_KEYWORDS.contains(&s.to_lowercase().as_str())
    }

    /// 辅助方法：从表达式中提取标识符 - 增强版
    /// 优化：支持带别名的标识符提取，提高解析成功率
    fn extract_identifiers_from_expression(&self, expr: &str) -> Vec<String> {
        let mut identifiers = Vec::with_capacity(10); // 预分配合理容量
        
        // 预处理表达式，处理别名
        let processed_expr = self.preprocess_expression_for_aliases(expr);
        
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
        
        // 2. 然后使用正则表达式提取标准格式的标识符
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
            
            // 过滤掉常见的SQL关键字
            if !Self::is_keyword(identifier) {
                identifiers.push(Self::normalize_identifier(identifier));
            }
        }
        
        // 处理别名
        self.extract_aliases(expr, &mut identifiers);
        
        // 去重
        identifiers.sort();
        identifiers.dedup();
        
        identifiers
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
            if !alias.is_empty() && !Self::is_keyword(alias) {
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
                if !Self::is_keyword(last_token) && tokens[tokens.len()-2].to_lowercase() != "as" {
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