use std::collections::HashSet;
use regex::Regex;

/// 增强版SQL解析器，解决现有解析器的各种问题
pub struct EnhancedSqlParserImproved {
    // 数据库方言特定处理
    dialect: Option<String>,
}

impl EnhancedSqlParserImproved {
    pub fn new(dialect: Option<String>) -> Self {
        Self {
            dialect
        }
    }

    /// 解析SQL并提取对象信息
    pub fn parse_sql(
        &self, 
        _sql: &str, 
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) -> bool {
        // 预处理SQL以提高解析成功率
        let processed_sql = self.preprocess_sql(_sql);
        
        // 检查是否为空或无效SQL
        if processed_sql.trim().is_empty() {
            return false;
        }
        
        // 多阶段解析策略
        // 1. 尝试标准解析（如果有完整实现）
        // 2. 尝试增强版解析
        let success = self.enhanced_extract_objects(&processed_sql, databases, schemas, tables, columns);
        
        // 如果是ClickHouse方言，进行特定处理
        if let Some(dialect) = &self.dialect {
            if dialect.to_lowercase().contains("clickhouse") {
                self.handle_clickhouse_specific(&processed_sql, databases, schemas, tables, columns);
            }
        }
        
        success
    }

    /// 预处理SQL，规范化特殊字符和语法
    fn preprocess_sql(&self, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        // 1. 移除注释
        processed = Self::remove_comments(&processed);
        
        // 2. 规范化特殊字符
        processed = Self::normalize_special_chars(&processed);
        
        // 3. 规范化引号
        processed = Self::normalize_quotes(&processed);
        
        // 4. 处理特定数据库方言
        if let Some(dialect) = &self.dialect {
            processed = Self::process_dialect_specific(dialect, &processed);
        }
        
        processed
    }

    /// 增强版对象提取，解决现有解析器的问题
    fn enhanced_extract_objects(
        &self,
        sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) -> bool {
        let sql_lower = sql.to_lowercase();
        
        // 提取数据库信息
        self.extract_databases(&sql_lower, sql, databases);
        
        // 提取Schema信息
        self.extract_schemas(&sql_lower, sql, schemas);
        
        // 提取表名（增强版，支持各种语句类型）
        self.extract_tables_enhanced(&sql_lower, sql, databases, schemas, tables);
        
        // 提取列名（增强版，支持更多SQL结构）
        self.extract_columns_enhanced(&sql_lower, sql, columns);
        
        // 处理WITH语句
        self.handle_with_statements(&sql_lower, sql, databases, schemas, tables, columns);
        
        // 处理CREATE TABLE语句
        self.handle_create_table(&sql_lower, sql, databases, schemas, tables, columns);
        
        // 处理ALTER TABLE语句
        self.handle_alter_table(&sql_lower, sql, databases, schemas, tables, columns);
        
        // 处理INSERT语句
        self.handle_insert_statement(&sql_lower, sql, databases, schemas, tables, columns);
        
        // 处理窗口函数
        self.handle_window_functions(&sql_lower, sql, columns);
        
        // 专门处理函数调用，特别是复杂函数如GaussDB的DISTANCE
        self.handle_function_calls(&sql_lower, sql, tables, columns);
        
        // 特殊处理GaussDB语法
        self.handle_gaussdb_specific(&sql_lower, sql, tables, columns);
        
        // 返回是否成功提取了至少一个对象
        !tables.is_empty() || !columns.is_empty() || !databases.is_empty() || !schemas.is_empty()
    }
    
    /// 特殊处理GaussDB语法
    fn handle_gaussdb_specific(
        &self,
        sql_lower: &str,
        sql: &str,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        // 检查是否包含GaussDB特有的函数调用
        if sql_lower.contains("distance") && sql_lower.contains("point") {
            // 从SQL中提取可能的表引用
            let table_refs = self.extract_possible_table_references(sql);
            tables.extend(table_refs);
            
            // 提取可能的列引用
            let column_refs = self.extract_possible_column_references(sql);
            columns.extend(column_refs);
        }
    }
    
    /// 从SQL中提取可能的表引用
    fn extract_possible_table_references(&self, sql: &str) -> Vec<String> {
        let mut tables = Vec::new();
        
        // 使用简单的启发式方法查找可能的表名
        // 这是一个简化的实现，实际应用中可能需要更复杂的逻辑
        let words: Vec<&str> = sql.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            // 查找FROM关键字后面的单词
            if i > 0 && (words[i-1].eq_ignore_ascii_case("FROM") || 
                         words[i-1].eq_ignore_ascii_case("JOIN") ||
                         words[i-1].eq_ignore_ascii_case("INTO")) {
                let potential_table = Self::normalize_identifier(word);
                if !Self::is_keyword(&potential_table) {
                    tables.push(potential_table);
                }
            }
        }
        
        tables
    }
    
    /// 从SQL中提取可能的列引用
    fn extract_possible_column_references(&self, sql: &str) -> Vec<String> {
        let mut columns = Vec::new();
        
        // 使用正则表达式提取标识符
        if let Ok(re) = Regex::new(r"([a-zA-Z0-9_]+\.)?[a-zA-Z0-9_]+") {
            for capture in re.captures_iter(sql) {
                if let Some(m) = capture.get(0) {
                    let identifier = m.as_str();
                    let normalized = Self::normalize_identifier(identifier);
                    if !Self::is_keyword(&normalized) {
                        columns.push(normalized);
                    }
                }
            }
        }
        
        columns
    }
    
    /// 专门处理函数调用，从函数参数中提取表和列信息
    fn handle_function_calls(
        &self,
        sql_lower: &str,
        _sql: &str,  // Unused parameter
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        // 提取所有函数调用（简化版实现，避免复杂的正则表达式）
        let words: Vec<&str> = sql_lower.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            // 查找可能的函数名（后跟左括号的单词）
            if let Some((func_name, _)) = word.split_once('(') {
                if !Self::is_keyword(func_name) {
                    // 找到函数体
                    let func_content = self.find_function_content(&sql_lower[i..]);
                    
                    // 从函数参数中提取标识符
                    let identifiers = Self::extract_identifiers_from_expression(&func_content);
                    columns.extend(identifiers);
                    
                    // 特殊处理GaussDB的DISTANCE函数
                    if func_name.to_lowercase() == "distance" && func_content.contains("point") {
                        // 从POINT函数中提取可能的表或列引用
                        self.extract_from_geo_functions(&func_content, tables, columns);
                    }
                }
            }
        }
    }
    
    /// 查找函数体内容
    fn find_function_content(&self, sql_part: &str) -> String {
        let mut paren_count = 0;
        let mut in_function = false;
        let mut content = String::new();
        
        for c in sql_part.chars() {
            if c == '(' {
                paren_count += 1;
                if !in_function {
                    in_function = true;
                    continue;
                }
            }
            
            if c == ')' {
                paren_count -= 1;
                if paren_count == 0 {
                    break;
                }
            }
            
            if in_function {
                content.push(c);
            }
        }
        
        content
    }
    
    /// 从地理函数中提取表和列信息
    fn extract_from_geo_functions(
        &self,
        func_content: &str,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        // 提取POINT函数中的参数
        if let Some(point_start) = func_content.to_lowercase().find("point(") {
            let point_part = &func_content[point_start + 6..]; // 跳过 "point("
            
            if let Some(point_end) = self.find_matching_parenthesis(point_part) {
                let point_params = &point_part[..point_end];
                
                // 从POINT参数中提取标识符
                let identifiers = Self::extract_identifiers_from_expression(point_params);
                columns.extend(identifiers);
                
                // 检查是否有表引用（例如：table.col 格式）
                for part in point_params.split(',') {
                    let trimmed = part.trim();
                    if let Some(dot_pos) = trimmed.find('.') {
                        let table_name = &trimmed[..dot_pos];
                        let normalized_table = Self::normalize_identifier(table_name);
                        if !Self::is_keyword(&normalized_table) {
                            tables.insert(normalized_table);
                        }
                    }
                }
            }
        }
    }
    
    /// 查找匹配的右括号
    fn find_matching_parenthesis(&self, s: &str) -> Option<usize> {
        let mut paren_count = 1; // 已经在point(里面了，所以初始为1
        
        for (i, c) in s.chars().enumerate() {
            if c == '(' {
                paren_count += 1;
            } else if c == ')' {
                paren_count -= 1;
                if paren_count == 0 {
                    return Some(i);
                }
            }
        }
        
        None
    }

    /// 提取数据库信息
    fn extract_databases(&self, sql_lower: &str, sql: &str, databases: &mut HashSet<String>) {
        // 1. 从USE语句中提取数据库名
        if let Some(use_pos) = sql_lower.find("use ") {
            let after_use = &sql[use_pos + 4..];
            let tokens: Vec<&str> = after_use.split(&[' ', ';', '\n', '\t'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if let Some(database_name) = tokens.get(0) {
                databases.insert(Self::normalize_identifier(database_name));
            }
        }
        
        // 2. 从CREATE DATABASE语句中提取数据库名
        if let Some(create_db_pos) = sql_lower.find("create database ") {
            let after_create = &sql[create_db_pos + 16..];
            let tokens: Vec<&str> = after_create.split(&[' ', ';', '\n', '\t'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if let Some(database_name) = tokens.get(0) {
                databases.insert(Self::normalize_identifier(database_name));
            }
        }
        
        // 3. 从表名中提取数据库名（database.schema.table格式）
        let table_ref_regex = Regex::new(r#"(?:([a-zA-Z0-9_]+)\.){1,2}([a-zA-Z0-9_]+)"#).unwrap();
        for captures in table_ref_regex.captures_iter(sql) {
            if captures.len() >= 3 {
                // 匹配到 database.schema.table 或 database.table 格式
                let db_candidate = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                if !db_candidate.is_empty() && !Self::is_keyword(db_candidate) {
                    databases.insert(Self::normalize_identifier(db_candidate));
                }
            }
        }
    }

    /// 提取Schema信息
    fn extract_schemas(&self, _sql_lower: &str, sql: &str, schemas: &mut HashSet<String>) {
        // 从表名中提取Schema信息（database.schema.table或schema.table格式）
        let schema_ref_regex = Regex::new(r#"(?:([a-zA-Z0-9_]+)\.)?([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)"#).unwrap();
        for captures in schema_ref_regex.captures_iter(sql) {
            if captures.len() >= 4 {
                // 匹配到 database.schema.table 格式
                let schema_candidate = captures.get(2).map(|m| m.as_str()).unwrap_or("");
                if !schema_candidate.is_empty() && !Self::is_keyword(schema_candidate) {
                    schemas.insert(Self::normalize_identifier(schema_candidate));
                }
            }
        }
    }

    /// 增强版表名提取，支持更多SQL语句类型
    fn extract_tables_enhanced(
        &self,
        sql_lower: &str,
        _sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
    ) {
        // 使用正则表达式提取各种子句中的表名
        let table_regexes = vec![
            Regex::new(r#"from\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"join\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"into\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"update\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"delete\s+from\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"create\s+(?:table|view)\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"alter\s+table\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
            Regex::new(r#"drop\s+(?:table|view)\s+(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_\.]+))"#).unwrap(),
        ];

        for regex in table_regexes {
            for captures in regex.captures_iter(sql_lower) {
                // 尝试从不同的捕获组获取表名
                let table_name = if let Some(m) = captures.get(1) {
                    m.as_str() // 带反引号的表名
                } else if let Some(m) = captures.get(2) {
                    m.as_str() // 带引号的表名
                } else if let Some(m) = captures.get(3) {
                    m.as_str() // 带方括号的表名
                } else if let Some(m) = captures.get(4) {
                    m.as_str() // 普通表名
                } else {
                    continue;
                };
                
                // 处理表名中的数据库和schema
                self.extract_db_schema_from_table_name(table_name, databases, schemas, tables);
            }
        }
    }

    /// 从表名中提取数据库和schema信息
    fn extract_db_schema_from_table_name(
        &self,
        table_name: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
    ) {
        let parts: Vec<&str> = table_name.split('.').collect();
        
        match parts.len() {
            1 => {
                // 只有表名
                tables.insert(Self::normalize_identifier(table_name));
            },
            2 => {
                // schema.table 或 database.table 格式
                let first_part = Self::normalize_identifier(parts[0]);
                let table_part = Self::normalize_identifier(parts[1]);
                
                // 简单判断第一个部分是schema还是database
                // 在没有更多信息的情况下，我们假设是schema
                schemas.insert(first_part);
                tables.insert(table_part);
            },
            3 => {
                // database.schema.table 格式
                let db_part = Self::normalize_identifier(parts[0]);
                let schema_part = Self::normalize_identifier(parts[1]);
                let table_part = Self::normalize_identifier(parts[2]);
                
                databases.insert(db_part);
                schemas.insert(schema_part);
                tables.insert(table_part);
            },
            _ => {
                // 多于3个部分，取最后一个作为表名
                if let Some(table_part) = parts.last() {
                    tables.insert(Self::normalize_identifier(table_part));
                }
            }
        }
    }

    /// 增强版列名提取，支持更多SQL结构
    fn extract_columns_enhanced(&self, sql_lower: &str, sql: &str, columns: &mut HashSet<String>) {
        // 1. 处理SELECT子句中的列
        if let Some(select_start) = sql_lower.find("select") {
            let mut from_pos = None;
            let mut set_pos = None;
            let mut into_pos = None;
            
            // 查找多个可能的结束位置，取最早出现的
            if let Some(fp) = sql_lower[select_start..].find(" from ") {
                from_pos = Some(select_start + fp);
            }
            if let Some(sp) = sql_lower[select_start..].find(" set ") {
                set_pos = Some(select_start + sp);
            }
            if let Some(ip) = sql_lower[select_start..].find(" into ") {
                into_pos = Some(select_start + ip);
            }
            
            let end_pos = [from_pos, set_pos, into_pos].iter()
                .filter_map(|&pos| pos)
                .min();
            
            if let Some(end_pos) = end_pos {
                let select_part = &sql[select_start + 6..end_pos];
                
                // 分割列名
                let column_parts = Self::split_sql_parts(select_part, ',');
                
                for part in column_parts {
                    let trimmed_part = part.trim();
                    // 跳过通配符
                    if trimmed_part == "*" {
                        continue;
                    }
                    
                    // 处理AS别名
                    let (column_part, _) = Self::split_on_as(trimmed_part);
                    
                    // 提取列名
                    let column_name = Self::extract_last_identifier(&column_part);
                    if !column_name.is_empty() && !Self::is_keyword(&column_name) {
                        columns.insert(column_name);
                    }
                }
            }
        }

        // 2. 从WHERE子句中提取列名
        if let Some(where_start) = sql_lower.find(" where ") {
            let where_part = &sql[where_start + 7..];
            columns.extend(Self::extract_identifiers_from_expression(where_part));
        }

        // 3. 从GROUP BY子句中提取列名
        if let Some(group_start) = sql_lower.find(" group by ") {
            let group_part = &sql[group_start + 9..];
            columns.extend(Self::extract_identifiers_from_expression(group_part));
        }

        // 4. 从ORDER BY子句中提取列名
        if let Some(order_start) = sql_lower.find(" order by ") {
            let order_part = &sql[order_start + 9..];
            columns.extend(Self::extract_identifiers_from_expression(order_part));
        }
    }

    /// 处理WITH语句
    fn handle_with_statements(
        &self,
        sql_lower: &str,
        sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        if let Some(with_start) = sql_lower.find("with ") {
            // 查找WITH块的结束位置（下一个SELECT、INSERT等）
            let mut end_pos = sql_lower.len();
            let end_keywords = ["select", "insert", "update", "delete", "create"];
            
            for keyword in end_keywords {
                if let Some(pos) = sql_lower[with_start + 5..].find(&format!(" {}", keyword)) {
                    let abs_pos = with_start + 5 + pos;
                    if abs_pos < end_pos {
                        end_pos = abs_pos;
                    }
                }
            }
            
            let with_part = &sql[with_start + 5..end_pos];
            
            // 分割WITH子句中的各个CTE
            let cte_parts = Self::split_sql_parts(with_part, ',');
            
            for cte in cte_parts {
                if let Some(as_pos) = cte.to_lowercase().find(" as ") {
                    let cte_name = Self::normalize_identifier(&cte[..as_pos].trim());
                    let cte_query = &cte[as_pos + 4..].trim();
                    
                    // 将CTE名称添加为表名
                    tables.insert(cte_name);
                    
                    // 递归解析CTE查询中的对象
                    self.enhanced_extract_objects(cte_query, databases, schemas, tables, columns);
                }
            }
        }
    }

    /// 处理CREATE TABLE语句
    fn handle_create_table(
        &self,
        sql_lower: &str,
        sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        if let Some(create_start) = sql_lower.find("create table ") {
            // 提取表名
            let after_create = &sql[create_start + 13..];
            let tokens: Vec<&str> = after_create.split(&[' ', '(', ';', '\n', '\t'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if let Some(table_name) = tokens.get(0) {
                self.extract_db_schema_from_table_name(table_name, databases, schemas, tables);
            }
            
            // 提取列定义
            if let Some(left_paren) = after_create.find('(') {
                if let Some(right_paren) = after_create[left_paren..].find(')') {
                    let columns_part = &after_create[left_paren + 1..left_paren + right_paren];
                    
                    // 分割列定义
                    let column_defs = Self::split_sql_parts(columns_part, ',');
                    
                    for col_def in column_defs {
                        let trimmed_def = col_def.trim();
                        // 跳过约束和特殊定义
                        if trimmed_def.starts_with("primary key") || 
                           trimmed_def.starts_with("foreign key") ||
                           trimmed_def.starts_with("constraint") {
                            continue;
                        }
                        
                        // 提取列名
                        let col_tokens: Vec<&str> = trimmed_def.split(&[' ', '\t'][..])
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();
                        
                        if let Some(col_name) = col_tokens.get(0) {
                            let normalized = Self::normalize_identifier(col_name);
                            if !normalized.is_empty() && !Self::is_keyword(&normalized) {
                                columns.insert(normalized);
                            }
                        }
                    }
                }
            }
            
            // 处理AS SELECT子句
            if let Some(as_select_pos) = sql_lower[create_start..].find(" as select ") {
                let select_query = &sql[create_start + as_select_pos + 10..];
                self.enhanced_extract_objects(select_query, databases, schemas, tables, columns);
            }
        }
    }

    /// 处理ALTER TABLE语句
    fn handle_alter_table(
        &self,
        sql_lower: &str,
        sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        if let Some(alter_start) = sql_lower.find("alter table ") {
            // 提取表名
            let after_alter = &sql[alter_start + 12..];
            let tokens: Vec<&str> = after_alter.split(&[' ', ';', '\n', '\t'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            
            if let Some(table_name) = tokens.get(0) {
                self.extract_db_schema_from_table_name(table_name, databases, schemas, tables);
            }
            
            // 处理各种ALTER操作
            if sql_lower.contains(" add column ") {
                let add_column_re = Regex::new(r#"add\s+column\s+(?:if\s+not\s+exists\s+)?(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|([a-zA-Z0-9_]+))"#).unwrap();
                for captures in add_column_re.captures_iter(sql_lower) {
                    let col_name = if let Some(m) = captures.get(1) {
                        m.as_str()
                    } else if let Some(m) = captures.get(2) {
                        m.as_str()
                    } else if let Some(m) = captures.get(3) {
                        m.as_str()
                    } else if let Some(m) = captures.get(4) {
                        m.as_str()
                    } else {
                        continue;
                    };
                    columns.insert(Self::normalize_identifier(col_name));
                }
            }
            
            // 处理UPDATE操作
            if sql_lower.contains(" update ") {
                if let Some(update_pos) = sql_lower.find(" update ") {
                    let update_part = &sql[update_pos + 7..];
                    columns.extend(Self::extract_identifiers_from_expression(update_part));
                }
            }
        }
    }

    /// 处理INSERT语句
    fn handle_insert_statement(
        &self,
        sql_lower: &str,
        sql: &str,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        if let Some(insert_start) = sql_lower.find("insert ") {
            // 处理INSERT INTO ... VALUES
            if let Some(into_pos) = sql_lower[insert_start..].find(" into ") {
                let after_into = &sql[insert_start + into_pos + 6..];
                
                // 提取表名
                let tokens: Vec<&str> = after_into.split(&[' ', '(', ';', '\n', '\t'][..])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                if let Some(table_name) = tokens.get(0) {
                    self.extract_db_schema_from_table_name(table_name, databases, schemas, tables);
                }
                
                // 提取列列表
                if let Some(left_paren) = after_into.find('(') {
                    if let Some(right_paren) = after_into[left_paren..].find(')') {
                        let columns_part = &after_into[left_paren + 1..left_paren + right_paren];
                        
                        let col_names = columns_part.split(',').map(|s| s.trim()).collect::<Vec<&str>>();
                        for col in col_names {
                            let normalized = Self::normalize_identifier(col);
                            if !normalized.is_empty() && !Self::is_keyword(&normalized) {
                                columns.insert(normalized);
                            }
                        }
                    }
                }
            }
            
            // 处理INSERT INTO ... SELECT
            if let Some(select_pos) = sql_lower[insert_start..].find(" select ") {
                let select_query = &sql[insert_start + select_pos + 8..];
                self.enhanced_extract_objects(select_query, databases, schemas, tables, columns);
            }
        }
    }

    /// 处理窗口函数
    fn handle_window_functions(&self, sql_lower: &str, _sql: &str, columns: &mut HashSet<String>) {  // _sql is intentionally unused
        // 查找窗口函数的OVER子句
        let over_re = Regex::new(r#"over\s*\(.*?\)"#).unwrap();
        for captures in over_re.captures_iter(sql_lower) {
            if let Some(over_clause) = captures.get(0) {
                // 从OVER子句中提取PARTITION BY和ORDER BY中的列
                let window_content = over_clause.as_str();
                
                // 提取PARTITION BY中的列
                if let Some(partition_start) = window_content.find("partition by ") {
                    let partition_part = &window_content[partition_start + 12..];
                    columns.extend(Self::extract_identifiers_from_expression(partition_part));
                }
                
                // 提取ORDER BY中的列
                if let Some(order_start) = window_content.find("order by ") {
                    let order_part = &window_content[order_start + 9..];
                    columns.extend(Self::extract_identifiers_from_expression(order_part));
                }
            }
        }
    }

    /// 处理ClickHouse特定语法
    fn handle_clickhouse_specific(
        &self,
        sql: &str,
        _databases: &mut HashSet<String>,
        _schemas: &mut HashSet<String>,
        _tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
    ) {
        let sql_lower = sql.to_lowercase();
        
        // 处理ARRAY JOIN
        if sql_lower.contains(" array join ") {
            let array_join_re = Regex::new(r#"array\s+join\s+([a-zA-Z0-9_,\s]+)"#).unwrap();
            for captures in array_join_re.captures_iter(&sql_lower) {
                if let Some(array_cols) = captures.get(1) {
                    let cols = array_cols.as_str().split(',').map(|s| s.trim()).collect::<Vec<&str>>();
                    for col in cols {
                        let normalized = Self::normalize_identifier(col);
                        if !normalized.is_empty() && !Self::is_keyword(&normalized) {
                            columns.insert(normalized);
                        }
                    }
                }
            }
        }
        
        // 处理MergeTree引擎定义
        if sql_lower.contains(" engine = mergetree") {
            // 从CREATE TABLE部分已经提取了表名和列名，这里可以添加特殊处理
        }
        
        // 处理ClickHouse特有的函数和语法
        // 例如：arrayJoin, tuple, map等
        let clickhouse_functions = ["arrayjoin", "tuple", "map", "array", "dictget"];
        for func in &clickhouse_functions {
            let func_re = Regex::new(&format!(r#"{}\s*\("#, func)).unwrap();
            for captures in func_re.captures_iter(&sql_lower) {
                if let Some(func_call) = captures.get(0) {
                    // 从函数调用中提取可能的列名
                    let func_content = func_call.as_str();
                    columns.extend(Self::extract_identifiers_from_expression(func_content));
                }
            }
        }
    }

    // 辅助方法：去除标识符中的引号
    fn normalize_identifier(s: &str) -> String {
        let s = s.trim();
        // 去除常见的标识符引号
        let s = s.trim_matches('`'); // MySQL反引号
        let s = s.trim_matches('"'); // PostgreSQL双引号
        let s = s.trim_matches('['); // SQL Server方括号
        let s = s.trim_matches(']'); // SQL Server方括号
        s.to_string()
    }

    // 辅助方法：检查是否为SQL关键字
    fn is_keyword(s: &str) -> bool {
        let keywords = [
            "select", "from", "where", "and", "or", "group", "by", "order", "having",
            "join", "inner", "outer", "left", "right", "full", "on", "as", "distinct",
            "limit", "offset", "fetch", "rows", "only", "first", "next", "current",
            "date", "time", "timestamp", "interval", "year", "month", "day", "hour",
            "minute", "second", "cast", "convert", "as", "null", "not", "exists",
            "in", "between", "like", "ilike", "glob", "regexp", "match", "case",
            "when", "then", "else", "end", "if", "elseif", "for", "update", "set",
            "delete", "insert", "into", "values", "create", "table", "view", "index",
            "alter", "drop", "truncate", "rename", "add", "modify", "change", "drop",
            "primary", "key", "foreign", "references", "unique", "check", "constraint",
            "default", "nulls", "asc", "desc", "using", "with", "window", "partition",
            "row", "rows", "range", "preceding", "following", "current", "row", "unbounded",
            "all", "any", "some", "exists", "distinct", "except", "intersect", "union",
            "minus", "true", "false", "is", "not", "null"
        ];
        keywords.contains(&s.to_lowercase().as_str())
    }

    // 辅助方法：从表达式中提取标识符
    fn extract_identifiers_from_expression(expr: &str) -> Vec<String> {
        let mut identifiers = Vec::new();
        
        // 1. 先处理GaussDB特有的POINT函数参数格式
        // 检查是否包含类似 (COI7D9074467256A3C9959469 8F887397705D1287A176928 1122334455667788) 的格式
        let gaussdb_pattern = Regex::new(r"\(([A-Z0-9]+)\s+([A-Z0-9]+)\s+([A-Z0-9]+)\)").unwrap();
        for captures in gaussdb_pattern.captures_iter(expr) {
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
        let identifier_re = Regex::new(r#"(?:`([^`]+)`|"([^"]+)"|\[([^\]]+)\]|\b([a-zA-Z0-9_]+)\b)"#).unwrap();
        
        for captures in identifier_re.captures_iter(expr) {
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

    // 辅助方法：安全地分割SQL部分，考虑嵌套括号
    // 性能优化：减少内存分配，避免不必要的trim操作
    fn split_sql_parts(sql: &str, delimiter: char) -> Vec<String> {
        // 预分配合理容量，减少动态扩容
        let capacity = sql.matches(delimiter).count() + 1;
        let mut parts = Vec::with_capacity(capacity);
        
        let mut start_idx = 0;
        let mut paren_count: i32 = 0;
        let mut in_quote = None;
        
        // 转换为字符迭代器并跟踪索引
        let chars: Vec<char> = sql.chars().collect();
        
        for (i, c) in chars.iter().enumerate() {
            // 处理引号
            if in_quote.is_none() && (*c == '\'' || *c == '"' || *c == '`') {
                in_quote = Some(*c);
            } else if in_quote == Some(*c) {
                in_quote = None;
            }
            
            // 处理括号
            if in_quote.is_none() && *c == '(' {
                paren_count += 1;
            } else if in_quote.is_none() && *c == ')' {
                paren_count = paren_count.saturating_sub(1);
            }
            
            // 如果是分隔符且不在括号/引号内，提取当前部分
            if *c == delimiter && paren_count == 0 && in_quote.is_none() {
                // 提取子字符串并trim
                let part = &sql[start_idx..i];
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    parts.push(trimmed.to_string());
                }
                start_idx = i + 1;
            }
        }
        
        // 添加最后一个部分
        let part = &sql[start_idx..];
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
        
        parts
    }

    // 辅助方法：分割AS关键字，考虑嵌套括号
    // 性能优化：避免多次字符转换和索引操作
    fn split_on_as(sql: &str) -> (String, String) {
        // 预先检查是否包含" as "，避免不必要的处理
        if !sql.to_lowercase().contains(" as ") {
            return (sql.to_string(), String::new());
        }
        
        let mut as_pos = None;
        let mut paren_count: i32 = 0;
        let mut in_quote = None;
        
        // 转换为字符数组便于快速访问
        let chars: Vec<char> = sql.chars().collect();
        let len = chars.len();
        
        let mut i = 0;
        while i < len - 2 {
            // 检查是否是 " as "
            if i + 3 <= len && 
               chars[i] == 'a' && chars[i+1] == 's' && chars[i+2] == ' ' &&
               // 确保as前有空格或边界
               (i == 0 || chars[i-1] == ' ') {
                
                // 检查是否在括号或引号内
                if paren_count == 0 && in_quote.is_none() {
                    as_pos = Some(i);
                    break;
                }
            }
            
            // 更新括号计数和引号状态
            match chars[i] {
                '(' if in_quote.is_none() => paren_count += 1,
                ')' if in_quote.is_none() => paren_count = paren_count.saturating_sub(1),
                '\'' | '"' | '`' => {
                    if in_quote.is_none() {
                        in_quote = Some(chars[i]);
                    } else if in_quote == Some(chars[i]) {
                        in_quote = None;
                    }
                }
                _ => {}
            }
            
            i += 1;
        }
        
        if let Some(pos) = as_pos {
            // 提取AS前面的部分（去除前后空格）
            let left_part = &sql[..pos].trim();
            // 提取AS后面的部分（去除前后空格）
            let right_part = sql[pos+3..].trim();
            
            (left_part.to_string(), right_part.to_string())
        } else {
            (sql.to_string(), String::new())
        }
    }

    // 辅助方法：移除注释
    fn remove_comments(sql: &str) -> String {
        let mut result = String::new();
        let mut in_single_line_comment = false;
        let mut in_multi_line_comment = false;
        let mut in_quote = None;
        let chars: Vec<char> = sql.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            if in_quote.is_none() {
                if !in_single_line_comment && !in_multi_line_comment && i < chars.len() - 1 {
                    // 检查单行注释
                    if chars[i] == '-' && chars[i+1] == '-' {
                        in_single_line_comment = true;
                        i += 2;
                        continue;
                    }
                    // 检查多行注释
                    if chars[i] == '/' && chars[i+1] == '*' {
                        in_multi_line_comment = true;
                        i += 2;
                        continue;
                    }
                }
                // 检查单行注释结束
                if in_single_line_comment && chars[i] == '\n' {
                    in_single_line_comment = false;
                    result.push(chars[i]);
                    i += 1;
                    continue;
                }
                // 检查多行注释结束
                if in_multi_line_comment && i < chars.len() - 1 && chars[i] == '*' && chars[i+1] == '/' {
                    in_multi_line_comment = false;
                    i += 2;
                    continue;
                }
            }
            
            if !in_single_line_comment && !in_multi_line_comment {
                // 处理引号
                if in_quote.is_none() && (chars[i] == '\'' || chars[i] == '"' || chars[i] == '`') {
                    in_quote = Some(chars[i]);
                } else if in_quote == Some(chars[i]) {
                    in_quote = None;
                }
                result.push(chars[i]);
            }
            
            i += 1;
        }
        
        result
    }

    // 辅助方法：规范化特殊字符
    fn normalize_special_chars(sql: &str) -> String {
        let mut result = sql.to_string();
        
        // 规范化制表符和换行符
        result = result.replace(&['\t', '\r'][..], " ");
        // 压缩多余的空格
        while result.contains("  ") {
            result = result.replace("  ", " ");
        }
        
        result
    }

    // 辅助方法：规范化引号
    fn normalize_quotes(sql: &str) -> String {
        let result = sql.to_string();
        
        // 处理双引号中的双引号（MySQL风格）
        // 例如：""AS"" 应该被转换为 `AS`
        let mut i = 0;
        let chars: Vec<char> = result.chars().collect();
        let mut new_result = String::new();
        let mut in_double_quotes = false;
        
        while i < chars.len() {
            if chars[i] == '"' {
                // 检查是否是双引号对
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    // 双引号对 - 转换为反引号
                    new_result.push('`');
                    i += 2;
                    in_double_quotes = true;
                } else {
                    // 单个双引号 - 如果在标识符上下文中，转换为反引号
                    if in_double_quotes {
                        new_result.push('`');
                        in_double_quotes = false;
                    } else {
                        new_result.push('`');
                        in_double_quotes = true;
                    }
                    i += 1;
                }
            } else {
                new_result.push(chars[i]);
                i += 1;
            }
        }
        
        new_result
    }

    // 辅助方法：处理方言特定的语法
    fn process_dialect_specific(dialect: &str, sql: &str) -> String {
        let mut processed = sql.to_string();
        
        match dialect.to_lowercase().as_str() {
            "mysql" => {
                // MySQL特定处理
                // 对于MySQL，双引号可能被用作标识符引号
                let mut in_quote = false;
                let mut result = String::new();
                for c in processed.chars() {
                    if c == '"' {
                        if in_quote {
                            result.push('`');
                            in_quote = false;
                        } else {
                            result.push('`');
                            in_quote = true;
                        }
                    } else {
                        result.push(c);
                    }
                }
                processed = result;
            },
            "postgresql" => {
                // PostgreSQL特定处理
                processed = processed.replace("::", " CAST( AS ");
                processed = processed.replace("||", " + ");
            },
            "sqlserver" => {
                // SQL Server特定处理
                processed = processed.replace("[", "`");
                processed = processed.replace("]", "`");
            },
            _ => {
                // 默认处理
            }
        }
        
        processed
    }
}