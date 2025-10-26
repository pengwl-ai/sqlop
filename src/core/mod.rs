pub mod error;
pub mod parser;
pub mod types;
pub mod ast_visitor;
pub mod piped_sql;
pub mod sql_transpiler;
pub mod enhanced_parser;
pub mod enhanced_parser_improved;
pub mod enhanced_parser_improved_optimized;
pub mod database_specific;

pub use error::{ParseError, Result};
pub use parser::SqlParser;
pub use enhanced_parser::EnhancedSqlParser;
pub use enhanced_parser_improved::EnhancedSqlParserImproved;
pub use enhanced_parser_improved_optimized::EnhancedSqlParserImprovedOptimized;
pub use database_specific::DatabaseSpecificHandler;
pub use types::{
    AuditLog, DatabaseConfig, DatabaseType, EnhancedParseResult, OperationType, ParseResult, ParserConfig,
    PerformanceStats, SensitivePattern, SqlObject,
};
pub use ast_visitor::{ObjectExtractor, SqlAstVisitor};
pub use piped_sql::PipedSqlParser;
pub use sql_transpiler::SqlTranspiler;

pub mod utils;

use std::fs;
use std::path::Path;
use uuid::Uuid;

// 导出表和视图相关的工具函数
pub use utils::{filter_tables, extract_view_info};

pub struct SqlopEngine {
    parser: SqlParser,
}

impl SqlopEngine {
    pub fn new(config: Option<ParserConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        let parser = SqlParser::new(config);
        Ok(Self { parser })
    }

    pub fn from_config_file(config_path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(config_path)
            .map_err(|e| ParseError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: ParserConfig = toml::from_str(&config_content)
            .map_err(|e| ParseError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        Self::new(Some(config))
    }

    pub fn parse_sql(&mut self, sql: &str, db_type: DatabaseType) -> Result<ParseResult> {
        let audit_log = AuditLog {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            database_type: db_type,
            user: None,
            client_ip: None,
            database_name: None,
            sql_text: sql.to_string(),
            execution_time_ms: None,
            rows_affected: None,
            status: "success".to_string(),
        };
        
        self.parser.parse_audit_log(&audit_log)
    }
    
    /// 解析SQL并返回增强的解析结果，包含更丰富的AST相关信息
    pub fn parse_sql_enhanced(&mut self, sql: &str, db_type: &DatabaseType) -> Result<EnhancedParseResult> {
        let base_result = self.parse_sql(sql, db_type.clone())?;
        
        // 构建增强的解析结果
        let enhanced_result = EnhancedParseResult {
            base_result,
            lineages: vec![],  // 后续将实现血缘关系分析
            expressions: vec![],  // 后续将实现表达式分析
            warnings: vec![],
            ast_node_count: 0,  // 后续将实现AST节点计数
            raw_sql: sql.to_string(),
            processed_sql: None,
            simplified_sql: sql.to_string(),
            is_enhanced_parsing: false,
            error_message: None,
        };
        
        Ok(enhanced_result)
    }

    // 修改回原始实现，因为测试文件和示例文件已经被修改为使用String类型
    pub fn parse_batch_sql(&mut self, sql_list: &[(String, DatabaseType)]) -> Vec<Result<ParseResult>> {
        let audit_logs: Vec<AuditLog> = sql_list
            .iter()
            .map(|(sql, db_type)| AuditLog {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                database_type: db_type.clone(),
                user: None,
                client_ip: None,
                database_name: None,
                sql_text: sql.clone(),  // 直接克隆String
                execution_time_ms: None,
                rows_affected: None,
                status: "success".to_string(),
            })
            .collect();
        
        self.parser.parse_batch(&audit_logs)
    }

    pub fn get_performance_stats(&self) -> EngineStats {
        let parser_stats = self.parser.get_performance_stats();
        EngineStats {
            cache_size: parser_stats.cache_size,
            config: self.parser.get_config().clone(),
            parse_count: parser_stats.parse_count,
            total_parse_time_ms: parser_stats.total_parse_time_ms,
            cache_hits: parser_stats.cache_hits,
            cache_misses: parser_stats.cache_misses,
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
    pub fn perform_security_check(&self, sql: &str, user: &str, database_objects: &[SqlObject]) -> crate::core::utils::advanced_security::SecurityCheckResult {
        let security_engine = crate::core::utils::advanced_security::SecurityRuleEngine::new();
        security_engine.check_security(sql, user, database_objects)
    }
    
    /// 应用性能优化
    pub fn optimize_performance(&self, sql: &str, db_type: DatabaseType) -> crate::core::utils::performance_optimization::LookaheadInfo {
        let mut optimizer = crate::core::utils::performance_optimization::LookaheadOptimizer::new();
        optimizer.lookahead(sql, db_type)
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
        Self::new(None).expect("Failed to create default engine")
    }
}