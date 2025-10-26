use crate::core::error::{ParseError, Result};
use crate::core::types::{
    AuditLog, DatabaseType, OperationType, ParseResult, ParserConfig, SqlObject,
    PerformanceStats,
};
use crate::core::ast_visitor::{ObjectExtractor, SqlAstVisitor};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, MsSqlDialect};
use crate::adapters::dialects::{HiveDialect, SQLiteDialect, DB2Dialect, DamengDialect};

use crate::adapters::dialects::enhanced_mysql_dialect::EnhancedMySqlDialect;
use crate::utils::{TableReferenceCollector, TableNameFilter};
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use lru::LruCache;
use std::sync::Arc;

/// 基于AST的SQL解析器
/// 使用sqlparser库进行标准的SQL语法解析和AST生成
pub struct AstSqlParser {
    config: ParserConfig,
    // 使用更高效的LRU缓存实现
    cache: LruCache<String, Arc<ParseResult>>,
    // 添加性能统计
    parse_count: AtomicU64,
    total_parse_time_ms: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl AstSqlParser {
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
                        let parser = AstSqlParser::new(config.clone());
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
                // 使用adapters中定义的Hive方言
                Ok(Box::new(HiveDialect))
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
                // 使用adapters中定义的DB2方言
                Ok(Box::new(DB2Dialect))
            }
            DatabaseType::Dameng => {
                // 使用adapters中定义的Dameng方言
                Ok(Box::new(DamengDialect))
            }
            DatabaseType::SQLite => {
                // 使用adapters中定义的SQLite方言
                Ok(Box::new(SQLiteDialect))
            }
        }
    }

    pub fn parse_sql(&self, sql: &str, database_type: &DatabaseType) -> Result<ParseResult> {
        let dialect = self.get_dialect(database_type)?;
        self.parse_sql_with_dialect(sql, database_type, dialect)
    }

    fn parse_sql_with_dialect(
        &self,
        sql: &str,
        db_type: &DatabaseType,
        dialect: Box<dyn Dialect>,
    ) -> Result<ParseResult> {
        // 创建Vec类型的变量用于提取结果
        let mut databases = HashSet::new();
        let mut schemas = HashSet::new();
        let mut tables = HashSet::new();
        let mut columns = HashSet::new();
        let mut objects = Vec::new();
        let mut operation_type = OperationType::OTHER;
        
        // 尝试解析SQL - 使用正确的API参数顺序
        let statements = match sqlparser::parser::Parser::parse_sql(&*dialect, sql) {
            Ok(statements) => statements,
            Err(e) => {
                return Err(ParseError::SqlParseError(format!("AST解析失败: {}", e)));
            }
        };
        
        // 处理解析出的语句
        for statement in statements {
            self.extract_objects_from_statement(
                &statement,
                &mut databases,
                &mut schemas,
                &mut tables,
                &mut columns,
                &mut objects,
                &mut operation_type,
            );
        }
        
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
        
        // 如果没有推断出操作类型，使用简单的前缀判断
        if operation_type == OperationType::OTHER {
            operation_type = self.infer_operation_type(sql);
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
}

// 使用adapters/dialects中定义的方言，不再在ast_parser中重复定义