use crate::core::error::{ParseError, Result};
use crate::EnhancedSqlParser;
use crate::core::enhanced_parser_improved::EnhancedSqlParserImproved;
use crate::core::enhanced_parser_improved_optimized::EnhancedSqlParserImprovedOptimized;
use crate::core::types::{
    AuditLog, DatabaseType, OperationType, ParseResult, ParserConfig, SqlObject,
    PerformanceStats,
};
use crate::core::ast_visitor::{ObjectExtractor, SqlAstVisitor};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, MsSqlDialect};

use crate::adapters::dialects::enhanced_mysql_dialect::EnhancedMySqlDialect;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use lru::LruCache;
use std::sync::Arc;

pub struct SqlParser {
    config: ParserConfig,
    // 使用更高效的LRU缓存实现
    cache: LruCache<String, Arc<ParseResult>>,
    // 添加性能统计
    parse_count: AtomicU64,
    total_parse_time_ms: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl SqlParser {
    pub fn new(config: ParserConfig) -> Self {
        // 确保缓存大小至少为1
        let cache_size = if config.cache_size < 1 {
            1
        } else {
            config.cache_size
        };
        
        Self {
            config,
            cache: LruCache::new(NonZeroUsize::new(cache_size).unwrap()),
            parse_count: AtomicU64::new(0),
            total_parse_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
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

        // 获取对应的方言
        let dialect = self.get_dialect(&audit_log.database_type)?;
        
        // 解析 SQL
        let mut parse_result = self.parse_sql_with_dialect(
            &audit_log.sql_text,
            &audit_log.database_type,
            dialect,
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
                
                // 根据可用核心数设置并行度
                rayon::ThreadPoolBuilder::new()
                    .num_threads(available_cores)
                    .build_global()
                    .ok();
                
                // 并行处理未缓存的SQL
                let uncached_results: Vec<Result<ParseResult>> = uncached_logs
                    .par_iter()
                    .with_min_len(batch_size)
                    .map(|log| {
                        let parser = SqlParser::new(config.clone());
                        parser.parse_sql_without_cache(log)
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
    fn parse_sql_without_cache(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        let start_time = Instant::now();
        
        // 获取对应的方言
        let dialect = self.get_dialect(&audit_log.database_type)?;
        
        // 解析 SQL
        let mut parse_result = self.parse_sql_with_dialect(
            &audit_log.sql_text,
            &audit_log.database_type,
            dialect,
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

    fn get_dialect(&self, db_type: &DatabaseType) -> Result<Box<dyn Dialect>> {
        match db_type {
            DatabaseType::MySQL => Ok(Box::new(EnhancedMySqlDialect::new())),
            DatabaseType::PostgreSQL => Ok(Box::new(PostgreSqlDialect {})),
            DatabaseType::SQLServer => Ok(Box::new(MsSqlDialect {})),
            DatabaseType::Oracle => {
                // Oracle 使用 PostgreSQL 方言作为基础，后续可以优化
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Hive => {
                // Hive 使用自定义方言
                Ok(Box::new(HiveDialect {}))
            }
            DatabaseType::GaussDB => {
                // GaussDB 使用 PostgreSQL 方言
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Kingbase => {
                // Kingbase 使用 PostgreSQL 方言
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Highgo => {
                // Highgo 使用 PostgreSQL 方言
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Greenplum => {
                // Greenplum 使用 PostgreSQL 方言
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Vastbase => {
                // Vastbase 使用 PostgreSQL 方言
                Ok(Box::new(PostgreSqlDialect {}))
            }
            DatabaseType::Sybase => {
                // Sybase 使用 MySQL 方言
                Ok(Box::new(MySqlDialect {}))
            }
            DatabaseType::DB2 => {
                // DB2 使用自定义方言
                Ok(Box::new(DB2Dialect {}))
            }
            DatabaseType::Dameng => {
                // Dameng 使用自定义方言
                Ok(Box::new(DamengDialect {}))
            }
            DatabaseType::SQLite => {
                // SQLite 使用自定义方言
                Ok(Box::new(SQLiteDialect {}))
            }
        }
    }

    fn parse_sql_with_dialect(
        &self,
        sql: &str,
        db_type: &DatabaseType,
        dialect: Box<dyn Dialect>,
    ) -> Result<ParseResult> {
        // 先获取dialect_name，因为dialect会在调用parse_sql_enhanced时被移动
        let dialect_name = self.get_dialect_name(&*dialect);
        
        // 尝试使用优化版增强解析器
    let mut databases = HashSet::new();
    let mut schemas = HashSet::new();
    let mut tables = HashSet::new();
    let mut columns = HashSet::new();
    
    // 先尝试优化版增强解析器
    let optimized_parser = EnhancedSqlParserImprovedOptimized::new(Some(dialect_name.clone()));
    if optimized_parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
        // 构建解析结果
        let result = ParseResult {
            database_type: db_type.clone(),
            original_sql: sql.to_string(),
            databases,
            schemas,
            tables,
            columns,
            objects: Vec::new(),
            operation_type: self.infer_operation_type(sql),
            parse_time_ms: 0
        };
        
        return Ok(result);
    }
    
    // 如果优化版解析失败，尝试使用增强版解析器
    match EnhancedSqlParser::parse_sql_enhanced(sql, db_type, dialect) {
        Ok(enhanced_result) => Ok(enhanced_result.base_result),
        Err(_) => {
            // 如果增强版解析失败，尝试使用最新的改进版增强解析器
            databases.clear();
            schemas.clear();
            tables.clear();
            columns.clear();
            
            // 使用已经预先保存的dialect_name
            let improved_parser = EnhancedSqlParserImproved::new(Some(dialect_name));
            
            if improved_parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
                    // 构建解析结果
                    let result = ParseResult {
                        database_type: db_type.clone(),
                        original_sql: sql.to_string(),
                        databases,
                        schemas,
                        tables,
                        columns,
                        objects: Vec::new(),
                        operation_type: self.infer_operation_type(sql),
                        parse_time_ms: 0
                    };
                    
                    Ok(result)
                } else {
                    Err(ParseError::SqlParseError("Failed to parse SQL".to_string()))
                }
            }
        }
    }
    
    // 获取方言名称用于改进版解析器
    fn get_dialect_name(&self, dialect: &dyn Dialect) -> String {
        let dialect_type = std::any::type_name_of_val(dialect);
        
        if dialect_type.contains("MySql") {
            "mysql".to_string()
        } else if dialect_type.contains("Postgre") || dialect_type.contains("GaussDB") {
            "postgresql".to_string()
        } else if dialect_type.contains("Hive") {
            "hive".to_string()
        } else if dialect_type.contains("SQLite") {
            "sqlite".to_string()
        } else if dialect_type.contains("Snowflake") {
            "snowflake".to_string()
        } else {
            "generic".to_string()
        }
    }
    
    // 推断操作类型
    fn infer_operation_type(&self, sql: &str) -> OperationType {
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

    fn extract_objects_from_statement(
        &self,
        statement: &Statement,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
        _objects: &mut Vec<SqlObject>,
        _operation_type: &mut OperationType,
    ) {
        // 使用新的ObjectExtractor来提取对象信息
        let mut extractor = ObjectExtractor::new();
        
        // 调用访问者模式的visit_statement方法
        extractor.visit_statement(statement);
        
        // 将提取的结果提交回原有的集合
        let (extracted_dbs, extracted_schemas, extracted_tbls, extracted_cols) = extractor.commit();
        databases.extend(extracted_dbs);
        schemas.extend(extracted_schemas);
        tables.extend(extracted_tbls);
        columns.extend(extracted_cols);
    }

    // 注意：原来的extract_from_query, extract_table_name等方法已经被ObjectExtractor替代
    // 现在这些功能通过访问者模式在ast_visitor模块中实现
}

// 自定义方言实现
#[derive(Debug)]
struct HiveDialect;
impl Dialect for HiveDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_start(ch)
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_part(ch)
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '`'
    }
    
    // 移除identifier_quote_style方法，因为它不是Dialect trait的成员
}

#[derive(Debug)]
struct SQLiteDialect;
impl Dialect for SQLiteDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_start(ch)
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_part(ch)
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"' || ch == '['
    }
    
    // 移除identifier_quote_style方法，因为它不是Dialect trait的成员
}

#[derive(Debug)]
struct DB2Dialect;
impl Dialect for DB2Dialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_start(ch)
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_part(ch)
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
    
    // 移除identifier_quote_style方法，因为它不是Dialect trait的成员
}

#[derive(Debug)]
struct DamengDialect;
impl Dialect for DamengDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_start(ch)
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        sqlparser::dialect::PostgreSqlDialect {}.is_identifier_part(ch)
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
    
    // 移除identifier_quote_style方法，因为它不是Dialect trait的成员
}