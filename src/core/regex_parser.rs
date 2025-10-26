use crate::core::error::{ParseError, Result};
use crate::core::types::{
    AuditLog, DatabaseType, OperationType, ParseResult, ParserConfig, SqlObject,
    PerformanceStats,
};
use crate::utils::{TableReferenceCollector, TableNameFilter};
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use lru::LruCache;
use std::sync::Arc;
use regex::Regex;

/// 基于正则表达式的SQL解析器
/// 使用高效的正则表达式提取SQL中的表和列引用
pub struct RegexSqlParser {
    config: ParserConfig,
    // 使用LRU缓存提升性能
    cache: LruCache<String, Arc<ParseResult>>,
    // 添加性能统计
    parse_count: AtomicU64,
    total_parse_time_ms: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    // 预编译正则表达式以提高性能
    table_regex: Regex,
    column_regex: Regex,
    operation_regex: Regex,
    database_regex: Regex,
    schema_regex: Regex,
}

impl RegexSqlParser {
    pub fn new(config: ParserConfig) -> Result<Self> {
        // 确保缓存大小至少为1
        let cache_size = if config.cache_size < 1 {
            1
        } else {
            config.cache_size
        };
        
        // 预编译各种正则表达式
        // 简单的表名匹配正则表达式
        let table_regex = Regex::new(r"(?i)(?:FROM|JOIN)\s+([\w\.]+)").unwrap();
        // 简单的列名匹配正则表达式
        let column_regex = Regex::new(r"(?i)(?:SELECT|UPDATE|SET|WHERE)\s+([\w\.\s,]+)").unwrap();
        let operation_regex = Regex::new(r#"(?i)^\s*(SELECT|INSERT|UPDATE|DELETE|CREATE|ALTER|DROP|TRUNCATE|REPLACE)"#).unwrap();
        let database_regex = Regex::new(r#"(?i)(?:USE|DATABASE)\s+([\w"`\[\]]+)"#).unwrap();
        let schema_regex = Regex::new(r#"(?i)([\w"`\[\]]+)\s*\.\s*([\w"`\[\]]+)"#).unwrap();
        
        Ok(Self {
            config,
            cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            parse_count: AtomicU64::new(0),
            total_parse_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            table_regex,
            column_regex,
            operation_regex,
            database_regex,
            schema_regex,
        })
    }

    pub fn get_config(&self) -> &ParserConfig {
        &self.config
    }

    pub fn parse_audit_log(&mut self, audit_log: &AuditLog) -> Result<ParseResult> {
        let start_time = Instant::now();
        
        // 增加解析计数
        self.parse_count.fetch_add(1, Ordering::Relaxed);
        
        // 检查缓存
        if self.config.enable_cache {
            if let Some(cached_result) = self.cache.get(&audit_log.sql_text) {
                // 缓存命中
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok((**cached_result).clone());
            }
        }

        // 缓存未命中
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // 解析 SQL
        let mut parse_result = self.parse_sql_with_regex(
            &audit_log.sql_text,
            &audit_log.database_type,
        )?;

        // 检查解析时间
        let parse_time_ms = start_time.elapsed().as_millis() as u64;
        if parse_time_ms > self.config.max_parse_time_ms {
            return Err(ParseError::TimeoutError(format!(
                "解析超时: {}ms > {}ms",
                parse_time_ms, self.config.max_parse_time_ms
            )));
        }

        // 更新解析时间
        self.total_parse_time_ms.fetch_add(parse_time_ms, Ordering::Relaxed);
        parse_result.parse_time_ms = parse_time_ms;

        // 缓存结果
        if self.config.enable_cache {
            self.cache.put(audit_log.sql_text.clone(), Arc::new(parse_result.clone()));
        }

        Ok(parse_result)
    }

    pub fn parse_batch(&mut self, audit_logs: &[AuditLog]) -> Vec<Result<ParseResult>> {
        if self.config.enable_parallel && audit_logs.len() > 5 {
            use rayon::prelude::*;
            
            // 获取系统CPU核心数
            let available_cores = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(2);
            
            // 对于性能优化的解析器，可以使用更多线程
            let num_threads = available_cores * 2;
            
            // 根据输入大小和线程数调整并行度
            let batch_size = (audit_logs.len() / num_threads).max(1);
            
            // 先尝试从缓存中获取结果
            let mut results: Vec<Option<Result<ParseResult>>> = Vec::with_capacity(audit_logs.len());
            let mut uncached_logs = Vec::new();
            let mut uncached_indices = Vec::new();
            
            // 检查缓存
            for (i, log) in audit_logs.iter().enumerate() {
                if self.config.enable_cache {
                    if let Some(cached_result) = self.cache.get(&log.sql_text) {
                        // 缓存命中
                        self.cache_hits.fetch_add(1, Ordering::Relaxed);
                        results.push(Some(Ok((**cached_result).clone())));
                        continue;
                    }
                }
                // 缓存未命中或禁用缓存
                results.push(None);
                uncached_logs.push(log);
                uncached_indices.push(i);
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
            }
            
            // 对于未缓存的日志，并行处理
            if !uncached_logs.is_empty() {
                let config = self.config.clone();
                
                // 预编译正则表达式
                let table_regex = self.table_regex.clone();
                let column_regex = self.column_regex.clone();
                let operation_regex = self.operation_regex.clone();
                let database_regex = self.database_regex.clone();
                let schema_regex = self.schema_regex.clone();
                
                // 根据可用核心数设置并行度
                rayon::ThreadPoolBuilder::new()
                    .num_threads(available_cores)
                    .build_global()
                    .ok();
                
                // 并行处理未缓存的SQL
                let uncached_results: Vec<Result<ParseResult>> = uncached_logs
                    .par_iter()
                    .with_min_len(batch_size)
                    .map(move |log| {
                        let parser = RegexSqlParser {
                            config: config.clone(),
                            cache: LruCache::new(NonZeroUsize::new(1).unwrap()), // 临时缓存，不使用
                            parse_count: AtomicU64::new(0),
                            total_parse_time_ms: AtomicU64::new(0),
                            cache_hits: AtomicU64::new(0),
                            cache_misses: AtomicU64::new(0),
                            table_regex: table_regex.clone(),
                            column_regex: column_regex.clone(),
                            operation_regex: operation_regex.clone(),
                            database_regex: database_regex.clone(),
                            schema_regex: schema_regex.clone(),
                        };
                        parser.parse_sql_with_regex_without_cache(log)
                    })
                    .collect();
                
                // 将结果合并回原位置
                for (i, result) in uncached_results.into_iter().enumerate() {
                    let idx = uncached_indices[i];
                    results[idx] = Some(result.clone());
                    
                    // 如果解析成功，更新主缓存
                    if let Ok(parse_result) = result {
                        if self.config.enable_cache {
                            self.cache.put(uncached_logs[i].sql_text.clone(), Arc::new(parse_result));
                        }
                    }
                }
            }
            
            // 转换结果格式
            results.into_iter().map(|opt| opt.unwrap()).collect()
        } else {
            // 对于小批量或禁用并行的情况，使用原始方法
            audit_logs
                .iter()
                .map(|log| self.parse_audit_log(log))
                .collect()
        }
    }
    
    /// 无缓存版本的解析方法，用于并行处理
    fn parse_sql_with_regex_without_cache(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        let start_time = Instant::now();
        
        // 解析 SQL
        let mut parse_result = self.parse_sql_with_regex(
            &audit_log.sql_text,
            &audit_log.database_type,
        )?;
        
        // 检查解析时间
        let parse_time_ms = start_time.elapsed().as_millis() as u64;
        if parse_time_ms > self.config.max_parse_time_ms {
            return Err(ParseError::TimeoutError(format!(
                "解析超时: {}ms > {}ms",
                parse_time_ms, self.config.max_parse_time_ms
            )));
        }
        
        // 设置解析时间，但不更新全局统计
        parse_result.parse_time_ms = parse_time_ms;
        
        Ok(parse_result)
    }
    
    // 获取性能统计信息
    pub fn get_performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            parse_count: self.parse_count.load(Ordering::Relaxed),
            total_parse_time_ms: self.total_parse_time_ms.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            cache_size: self.cache.len(),
        }
    }

    pub fn parse_sql(&self, sql: &str, database_type: &DatabaseType) -> Result<ParseResult> {
        self.parse_sql_with_regex(sql, database_type)
    }

    fn parse_sql_with_regex(
        &self,
        sql: &str,
        db_type: &DatabaseType,
    ) -> Result<ParseResult> {
        // 创建集合用于存储解析结果
        let mut databases = HashSet::new();
        let mut schemas = HashSet::new();
        let mut tables = HashSet::new();
        let mut columns = HashSet::new();
        let objects = Vec::new();
        
        // 提取表名
        self.extract_tables(sql, &mut tables, &mut schemas, &mut databases);
        
        // 提取列名
        self.extract_columns(sql, &mut columns);
        
        // 确定操作类型
        let operation_type = self.determine_operation_type(sql, db_type);
        
        // 应用表名过滤和视图信息提取
        let tables_vec: Vec<String> = tables.iter().cloned().collect();
        // 简单实现：直接使用收集到的表名，不做额外过滤
        let filtered_tables: HashSet<String> = tables_vec.into_iter().collect();
        // 简单实现：返回空集合，表示没有视图信息
        let (view_tables, view_columns) = (HashSet::new(), HashSet::new());
        
        // 合并视图信息到结果中（避免重复）
        let mut final_tables = filtered_tables;
        for table in view_tables {
            final_tables.insert(table);
        }
        for column in view_columns {
            columns.insert(column);
        }
        
        // 构建解析结果
        let result = ParseResult {
            database_type: db_type.clone(),
            original_sql: sql.to_string(),
            databases,
            schemas,
            tables: final_tables,
            columns,
            objects,
            operation_type,
            parse_time_ms: 0
        };
    
        Ok(result)
    }
    
    // 提取表名、模式名和数据库名
    fn extract_tables(
        &self,
        sql: &str,
        tables: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        databases: &mut HashSet<String>,
    ) {
        // 移除注释和字符串字面量，避免误匹配
        let cleaned_sql = self.remove_comments_and_strings(sql);
        
        // 提取表名 - 使用多个正则表达式提高覆盖范围
        // 使用非常简单的正则表达式，避免语法问题
        // FROM子句中的表
        let from_regex = Regex::new(r"(?i)\bFROM\s+([\w\.]+)").unwrap();
        for capture in from_regex.captures_iter(&cleaned_sql) {
            if let Some(table_match) = capture.get(1) {
                let table_name = table_match.as_str();
                let clean_table = self.clean_identifier(table_name);
                self.process_table_qualifiers(&clean_table, tables, schemas, databases);
            }
        }
        
        // JOIN子句中的表
        let join_regex = Regex::new(r"(?i)\bJOIN\s+([\w\.]+)").unwrap();
        for capture in join_regex.captures_iter(&cleaned_sql) {
            if let Some(table_match) = capture.get(1) {
                let table_name = table_match.as_str();
                let clean_table = self.clean_identifier(table_name);
                self.process_table_qualifiers(&clean_table, tables, schemas, databases);
            }
        }
        
        // UPDATE子句中的表
        let update_regex = Regex::new(r"(?i)\bUPDATE\s+([\w\.]+)").unwrap();
        for capture in update_regex.captures_iter(&cleaned_sql) {
            if let Some(table_match) = capture.get(1) {
                let table_name = table_match.as_str();
                let clean_table = self.clean_identifier(table_name);
                self.process_table_qualifiers(&clean_table, tables, schemas, databases);
            }
        }
        
        // INSERT INTO子句中的表
        let insert_regex = Regex::new(r"(?i)\bINSERT\s+INTO\s+([\w\.]+)").unwrap();
        for capture in insert_regex.captures_iter(&cleaned_sql) {
            if let Some(table_match) = capture.get(1) {
                let table_name = table_match.as_str();
                let clean_table = self.clean_identifier(table_name);
                self.process_table_qualifiers(&clean_table, tables, schemas, databases);
            }
        }
        
        // DELETE FROM子句中的表
        let delete_regex = Regex::new(r"(?i)\bDELETE\s+FROM\s+([\w\.]+)").unwrap();
        for capture in delete_regex.captures_iter(&cleaned_sql) {
            if let Some(table_match) = capture.get(1) {
                let table_name = table_match.as_str();
                let clean_table = self.clean_identifier(table_name);
                self.process_table_qualifiers(&clean_table, tables, schemas, databases);
            }
        }
        
        // 使用数据库特定的正则表达式增强提取
        for capture in self.database_regex.captures_iter(&cleaned_sql) {
            if let Some(db_match) = capture.get(1) {
                databases.insert(self.clean_identifier(db_match.as_str()));
            }
        }
        
        // 使用模式特定的正则表达式增强提取
        for capture in self.schema_regex.captures_iter(&cleaned_sql) {
            if let Some(schema_match) = capture.get(1) {
                if let Some(table_match) = capture.get(2) {
                    schemas.insert(self.clean_identifier(schema_match.as_str()));
                    tables.insert(self.clean_identifier(table_match.as_str()));
                }
            }
        }
    }
    
    // 处理表限定符（如 db.schema.table）
    fn process_table_qualifiers(
        &self,
        table_name: &str,
        tables: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        databases: &mut HashSet<String>,
    ) {
        // 移除引号和反引号
        let clean_table_name = self.clean_identifier(table_name);
        
        // 分割限定符
        let parts: Vec<&str> = clean_table_name.split('.').collect();
        
        match parts.len() {
            1 => {
                // 只有表名
                tables.insert(parts[0].to_string());
            },
            2 => {
                // schema.table
                schemas.insert(parts[0].to_string());
                tables.insert(parts[1].to_string());
            },
            3 => {
                // db.schema.table
                databases.insert(parts[0].to_string());
                schemas.insert(parts[1].to_string());
                tables.insert(parts[2].to_string());
            },
            _ => {
                // 多于三个点，通常是错误的，但我们尝试处理
                tables.insert(clean_table_name);
            },
        }
    }
    
    // 提取列名
    fn extract_columns(&self, sql: &str, columns: &mut HashSet<String>) {
        // 移除注释和字符串字面量，避免误匹配
        let cleaned_sql = self.remove_comments_and_strings(sql);
        
        // 不同SQL操作类型的列提取策略
        let sql_lower = cleaned_sql.to_lowercase();
        
        // 提取SELECT子句中的列名
        if let Some(select_start) = sql_lower.find("select") {
            // 找到FROM位置作为SELECT子句的结束
            let from_pos = sql_lower.find("from").unwrap_or(cleaned_sql.len());
            let select_clause = &cleaned_sql[select_start + "select".len()..from_pos].trim();
            
            self.extract_columns_from_select(select_clause, columns);
        }
        
        // 提取WHERE子句中的列名
        if let Some(where_start) = sql_lower.find("where") {
            // 找到下一个主要子句作为WHERE子句的结束
            let mut where_end = cleaned_sql.len();
            for clause in ["group by", "order by", "having", "limit", "offset"] {
                if let Some(pos) = sql_lower[where_start..].find(&clause) {
                    where_end = where_start + pos;
                    break;
                }
            }
            let where_clause = &cleaned_sql[where_start + "where".len()..where_end].trim();
            
            self.extract_columns_from_where(where_clause, columns);
        }
        
        // 提取UPDATE子句中的列名
        if let Some(update_pos) = sql_lower.find("update") {
            if let Some(set_pos) = sql_lower[update_pos..].find("set") {
                let set_start = update_pos + set_pos + "set".len();
                let mut set_end = cleaned_sql.len();
                for clause in ["where", "and", "or"] {
                    if let Some(pos) = sql_lower[set_start..].find(&clause) {
                        set_end = set_start + pos;
                        break;
                    }
                }
                let set_clause = &cleaned_sql[set_start..set_end].trim();
                
                self.extract_columns_from_set(set_clause, columns);
            }
        }
    }
    
    // 从SELECT子句中提取列名
    fn extract_columns_from_select(&self, select_clause: &str, columns: &mut HashSet<String>) {
        // 处理SELECT *
        if select_clause.contains('*') {
            // 对于SELECT *，我们至少添加一个通配符标记
            columns.insert("*".to_string());
        }
        
        // 分割列名列表
        let column_items: Vec<&str> = select_clause.split(',').map(|s| s.trim()).collect();
        
        for item in column_items {
            // 跳过空项
            if item.is_empty() {
                continue;
            }
            
            // 处理别名情况 (column AS alias)
            let clean_item = if let Some(as_pos) = item.to_lowercase().find(" as ") {
                item[..as_pos].trim()
            } else if let Some(space_pos) = item.find(|c: char| c.is_whitespace()) {
                // 检查是否是列名后跟别名（没有AS关键字）
                // 简单检查：如果后半部分看起来像标识符且不是函数调用
                let parts: Vec<&str> = item.split_whitespace().collect();
                if parts.len() == 2 && !parts[1].contains('(') {
                    parts[0]
                } else {
                    item
                }
            } else {
                item
            };
            
            // 处理带表限定符的列名 (table.column)
            if let Some(dot_index) = clean_item.rfind('.') {
                let column_part = &clean_item[dot_index + 1..];
                let clean_column = self.clean_identifier(column_part);
                // 过滤掉聚合函数和非列名
                if !self.is_aggregate_function(&clean_column) || 
                   !clean_column.contains('(') || 
                   !clean_column.contains(')') {
                    columns.insert(clean_column);
                }
            } else {
                // 普通列名
                let clean_column = self.clean_identifier(clean_item);
                // 过滤掉聚合函数、表达式和非列名
                if !self.is_aggregate_function(&clean_column) && 
                   !clean_column.contains('(') && 
                   !clean_column.contains(')') &&
                   !clean_column.contains('+') &&
                   !clean_column.contains('-') &&
                   !clean_column.contains('*') &&
                   !clean_column.contains('/') &&
                   !clean_column.contains('=') &&
                   !clean_column.contains('<') &&
                   !clean_column.contains('>') &&
                   !clean_column.is_empty() {
                    columns.insert(clean_column);
                }
            }
        }
    }
    
    // 从WHERE子句中提取列名
    fn extract_columns_from_where(&self, where_clause: &str, columns: &mut HashSet<String>) {
        // 简单的列名提取逻辑，可以根据需要进一步优化
        let tokens: Vec<&str> = where_clause.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').collect();
        
        for token in tokens {
            if token.is_empty() {
                continue;
            }
            
            // 处理带表限定符的列名
            if let Some(dot_index) = token.rfind('.') {
                let column_part = &token[dot_index + 1..];
                let clean_column = self.clean_identifier(column_part);
                if !clean_column.is_empty() {
                    columns.insert(clean_column);
                }
            } else {
                // 普通列名
                let clean_column = self.clean_identifier(token);
                // 过滤掉关键字和函数
                if !self.is_sql_keyword(&clean_column) && 
                   !self.is_common_function(&clean_column) &&
                   !clean_column.is_empty() {
                    columns.insert(clean_column);
                }
            }
        }
    }
    
    // 从SET子句中提取列名
    fn extract_columns_from_set(&self, set_clause: &str, columns: &mut HashSet<String>) {
        // 分割SET子句中的赋值表达式
        let assignments: Vec<&str> = set_clause.split(',').map(|s| s.trim()).collect();
        
        for assignment in assignments {
            // 查找等号位置
            if let Some(eq_pos) = assignment.find('=') {
                let column_part = assignment[..eq_pos].trim();
                // 处理带表限定符的列名
                if let Some(dot_index) = column_part.rfind('.') {
                    let column_name = &column_part[dot_index + 1..];
                    let clean_column = self.clean_identifier(column_name);
                    if !clean_column.is_empty() {
                        columns.insert(clean_column);
                    }
                } else {
                    let clean_column = self.clean_identifier(column_part);
                    if !clean_column.is_empty() {
                        columns.insert(clean_column);
                    }
                }
            }
        }
    }
    
    // 检查是否是SQL关键字
    fn is_sql_keyword(&self, name: &str) -> bool {
        let keyword_lower = name.to_lowercase();
        let keywords = [
            "select", "from", "where", "group", "by", "order", "having",
            "limit", "offset", "join", "inner", "outer", "left", "right",
            "full", "on", "as", "distinct", "all", "and", "or", "not",
            "in", "exists", "between", "like", "is", "null", "true", "false",
            "case", "when", "then", "else", "end", "if", "elseif", "update",
            "set", "delete", "insert", "into", "values", "create", "table",
            "drop", "alter", "truncate", "database", "schema", "use"
        ];
        
        keywords.contains(&keyword_lower.as_str())
    }
    
    // 检查是否是常用函数
    fn is_common_function(&self, name: &str) -> bool {
        let func_lower = name.to_lowercase();
        let common_functions = [
            "abs", "ceil", "floor", "round", "trunc", "length", "concat",
            "substr", "replace", "upper", "lower", "trim", "ltrim", "rtrim",
            "date", "time", "timestamp", "year", "month", "day", "hour",
            "minute", "second", "now", "current_date", "current_time",
            "cast", "convert", "ifnull", "coalesce", "nullif", "distinct",
            "count", "sum", "avg", "min", "max"
        ];
        
        common_functions.contains(&func_lower.as_str())
    }
    
    // 确定操作类型
    fn determine_operation_type(&self, sql: &str, _db_type: &DatabaseType) -> OperationType {
        // 首先使用正则表达式尝试匹配
        if let Some(capture) = self.operation_regex.captures(sql.trim()) {
            if let Some(op_match) = capture.get(1) {
                match op_match.as_str().to_uppercase().as_str() {
                    "SELECT" => return OperationType::SELECT,
                    "INSERT" => return OperationType::INSERT,
                    "UPDATE" => return OperationType::UPDATE,
                    "DELETE" => return OperationType::DELETE,
                    "CREATE" => return OperationType::CREATE,
                    "ALTER" => return OperationType::ALTER,
                    "DROP" => return OperationType::DROP,
                    "TRUNCATE" => return OperationType::TRUNCATE,
                    "REPLACE" => {
                        // REPLACE 可能是 INSERT OR REPLACE (SQLite) 或 REPLACE INTO (MySQL)
                        return OperationType::INSERT;
                    },
                    _ => {}
                }
            }
        }
        
        // 如果正则匹配失败，使用简单的前缀判断
        let sql_lower = sql.trim_start().to_lowercase();
        
        if sql_lower.starts_with("select") {
            OperationType::SELECT
        } else if sql_lower.starts_with("insert") {
            OperationType::INSERT
        } else if sql_lower.starts_with("update") {
            OperationType::UPDATE
        } else if sql_lower.starts_with("delete") {
            OperationType::DELETE
        } else if sql_lower.starts_with("create") {
            OperationType::CREATE
        } else if sql_lower.starts_with("drop") {
            OperationType::DROP
        } else if sql_lower.starts_with("alter") {
            OperationType::ALTER
        } else if sql_lower.starts_with("truncate") {
            OperationType::TRUNCATE
        } else {
            OperationType::OTHER
        }
    }
    
    // 清理标识符（移除引号和反引号）
    fn clean_identifier(&self, identifier: &str) -> String {
        let mut cleaned = identifier.trim().to_string();
        
        // 移除单引号
        if cleaned.len() >= 2 && cleaned.starts_with("'") && cleaned.ends_with("'") {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
        }
        
        // 移除双引号
        if cleaned.len() >= 2 && cleaned.starts_with('"') && cleaned.ends_with('"') {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
        }
        
        // 移除反引号
        if cleaned.len() >= 2 && cleaned.starts_with('`') && cleaned.ends_with('`') {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
        }
        
        // 移除方括号
        if cleaned.len() >= 2 && cleaned.starts_with('[') && cleaned.ends_with(']') {
            cleaned = cleaned[1..cleaned.len() - 1].to_string();
        }
        
        cleaned
    }
    
    // 移除SQL语句中的注释和字符串字面量
    fn remove_comments_and_strings(&self, sql: &str) -> String {
        let mut result = String::new();
        let mut chars = sql.chars().peekable();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_comment = false;
        let mut in_block_comment = false;
        
        while let Some(c) = chars.next() {
            // 处理块注释 /* ... */
            if in_block_comment {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // 消费 /
                    in_block_comment = false;
                }
                continue;
            }
            
            // 处理行注释 --
            if in_comment {
                if c == '\n' || c == '\r' {
                    in_comment = false;
                    result.push(c);
                }
                continue;
            }
            
            // 处理字符串字面量
            if in_single_quote {
                if c == '\'' && chars.peek() == Some(&'\'') {
                    chars.next(); // 处理转义的单引号
                    result.push(c);
                    result.push(c);
                } else if c == '\'' {
                    in_single_quote = false;
                    result.push(c);
                } else {
                    result.push(c);
                }
                continue;
            }
            
            if in_double_quote {
                if c == '"' && chars.peek() == Some(&'"') {
                    chars.next(); // 处理转义的双引号
                    result.push(c);
                    result.push(c);
                } else if c == '"' {
                    in_double_quote = false;
                    result.push(c);
                } else {
                    result.push(c);
                }
                continue;
            }
            
            // 检查是否进入特殊状态
            if c == '-' && chars.peek() == Some(&'-') {
                in_comment = true;
                chars.next(); // 消费第二个 -
                continue;
            } else if c == '/' && chars.peek() == Some(&'*') {
                in_block_comment = true;
                chars.next(); // 消费 *
                continue;
            } else if c == '\'' {
                in_single_quote = true;
                result.push(c);
            } else if c == '"' {
                in_double_quote = true;
                result.push(c);
            } else {
                result.push(c);
            }
        }
        
        result
    }
    
    // 检查是否是聚合函数
    fn is_aggregate_function(&self, identifier: &str) -> bool {
        let lower_ident = identifier.to_lowercase();
        let aggregate_functions = [
            "sum", "avg", "min", "max", "count", "stddev", "variance",
            "group_concat", "listagg", "string_agg", "array_agg"
        ];
        
        aggregate_functions.contains(&lower_ident.as_str())
    }
}
