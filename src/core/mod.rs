pub mod error;
pub mod types;
pub mod ast_visitor;
pub mod piped_sql;
pub mod sql_transpiler;
pub mod layered_parser;
pub mod database_specific;
pub mod ast_parser;
pub mod regex_parser;

pub use error::{ParseError, Result};
pub use layered_parser::LayeredSqlParser;
pub use ast_parser::AstSqlParser;
pub use regex_parser::RegexSqlParser;
pub use database_specific::DatabaseSpecificHandler;
pub use types::{
    AuditLog, DatabaseConfig, DatabaseType, EnhancedParseResult, OperationType, ParseResult, ParserConfig,
    PerformanceStats, SensitivePattern, SqlObject,
};
pub use ast_visitor::{ObjectExtractor, SqlAstVisitor};
pub use piped_sql::PipedSqlParser;
pub use sql_transpiler::SqlTranspiler;

use std::fs;
use std::path::Path;

// 导出表和视图相关的工具函数
pub use crate::utils::{TableReferenceCollector, TableNameFilter};

pub struct SqlopEngine {
    layered_parser: LayeredSqlParser,
    config: ParserConfig,
}

impl SqlopEngine {
    pub fn new(config: Option<ParserConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let layered_parser = LayeredSqlParser::new(config.clone());
        Ok(Self { 
            layered_parser,
            config
        })
    }

    pub fn from_config_file(config_path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| ParseError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: ParserConfig = toml::from_str(&config_content)
            .map_err(|e| ParseError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        Self::new(Some(config))
    }

    pub fn parse_sql(&mut self, sql: &str, db_type: DatabaseType) -> Result<ParseResult> {
        // 使用LayeredSqlParser，它会首先尝试AST解析器，然后再尝试正则解析器
        self.layered_parser.parse_sql(sql, &db_type)
    }
    
    /// 解析SQL并返回增强的解析结果，包含更丰富的AST相关信息
    pub fn parse_sql_enhanced(&mut self, sql: &str, db_type: &DatabaseType) -> Result<EnhancedParseResult> {
        // 使用parse_sql方法（内部使用LayeredSqlParser）
        let base_result = self.parse_sql(sql, db_type.clone())?;
        
        // 构建增强的解析结果
        let enhanced_result = EnhancedParseResult {
            base_result,
            lineages: vec![],
            expressions: vec![],
            warnings: vec![],
            ast_node_count: 0,
            raw_sql: sql.to_string(),
            processed_sql: None,
            simplified_sql: sql.to_string(),
            is_enhanced_parsing: true,
            error_message: None,
        };
        
        Ok(enhanced_result)
    }

    // 使用LayeredSqlParser进行批量SQL解析
    pub fn parse_batch_sql(&mut self, sql_list: &[(String, DatabaseType)]) -> Vec<Result<ParseResult>> {
        // 对每个SQL单独解析
        sql_list
            .iter()
            .map(|(sql, db_type)| self.parse_sql(sql, db_type.clone()))
            .collect()
    }

    pub fn get_performance_stats(&self) -> EngineStats {
        // 由于LayeredSqlParser内部使用了AstSqlParser，我们可以直接返回基本统计信息
        // 注意：这里简化了性能统计，实际应用中可能需要扩展LayeredSqlParser以提供完整的统计
        EngineStats {
            cache_size: 0, // LayeredSqlParser暂时不直接暴露缓存大小
            config: self.config.clone(),
            parse_count: 0, // 简化实现
            total_parse_time_ms: 0, // 简化实现
            cache_hits: 0, // 简化实现
            cache_misses: 0, // 简化实现
        }
    }
    
    /// 解析Piped SQL并转换为标准SQL
    pub fn parse_piped_sql(&self, sql: &str) -> Result<String> {
        let mut piped_parser = PipedSqlParser::new(DatabaseType::MySQL)?;
        piped_parser.convert_to_standard_sql(sql)
            .map_err(|e| ParseError::SqlParseError(format!("Piped SQL解析失败: {}", e)))
    }
    
    /// 在不同数据库方言间转换SQL
    pub fn transpile_sql(&self, sql: &str, source_type: DatabaseType, target_type: DatabaseType) -> Result<String> {
        let transpiler = SqlTranspiler::new(source_type, target_type);
        transpiler.transpile(sql)
            .map_err(|e| ParseError::SqlParseError(format!("SQL转换失败: {}", e)))
    }
    
    /// 获取支持的目标数据库类型
    pub fn get_supported_target_types(&self, source_type: DatabaseType) -> Vec<DatabaseType> {
        SqlTranspiler::get_supported_target_types(source_type)
    }
    
    /// 执行高级安全检查
    pub fn perform_security_check(&self, sql: &str, user: &str, database_objects: &[SqlObject]) -> crate::utils::advanced_security::SecurityCheckResult {
        let security_engine = crate::utils::SecurityRuleEngine::new();
        security_engine.check_security(sql, user, database_objects)
    }
    
    /// 应用性能优化
    pub fn optimize_performance(&self, sql: &str, db_type: DatabaseType) -> crate::utils::performance_optimization::LookaheadInfo {
        let mut optimizer = crate::utils::LookaheadOptimizer::new();
        optimizer.lookahead(sql, &db_type)
    }
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub cache_size: usize,
    pub config: ParserConfig,
    pub parse_count: u64,
    pub total_parse_time_ms: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Default for SqlopEngine {
    fn default() -> Self {
        let config = ParserConfig::default();
        Self {
            layered_parser: LayeredSqlParser::new(config.clone()),
            config
        }
    }
}