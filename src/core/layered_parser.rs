use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{ParseError, Result};
use crate::core::types::{DatabaseType, ParseResult, ParserConfig, SqlObject, OperationType};
use crate::core::parser::SqlParser;
use crate::core::enhanced_parser_improved_optimized::EnhancedSqlParserImprovedOptimized;

// 定义分层解析策略枚举
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseStrategy {
    AstParsing,      // 使用SqlParser（AST解析）
    RegexParsing,    // 使用EnhancedSqlParserImprovedOptimized（正则解析）
}

/// 分层SQL解析器实现
/// 作为协调者，在不同解析器之间切换：
/// 1. 首先尝试使用SqlParser（AST解析）
/// 2. 如果失败，尝试使用EnhancedSqlParserImprovedOptimized（正则解析）
/// 支持配置解析策略顺序和开关
pub struct LayeredSqlParser {
    // AST解析器实例
    ast_parser: SqlParser,
    // 正则解析器实例
    regex_parser: EnhancedSqlParserImprovedOptimized,
    // 解析策略配置
    parse_strategies: Vec<ParseStrategy>,
}

impl LayeredSqlParser {
    /// 创建新的分层SQL解析器实例
    pub fn new(config: ParserConfig) -> Self {
        // 初始化AST解析器
        let ast_parser = SqlParser::new(config.clone());
        
        // 初始化正则解析器
        let regex_parser = EnhancedSqlParserImprovedOptimized::new(None);
        
        // 定义默认解析策略顺序
        let parse_strategies = vec![
            ParseStrategy::AstParsing,
            ParseStrategy::RegexParsing,
        ];
        
        Self {
            ast_parser,
            regex_parser,
            parse_strategies,
        }
    }
    
    /// 配置解析器选项
    pub fn configure(mut self, options: &HashMap<String, String>) -> Self {
        // 处理配置选项
        for (key, value) in options {
            match key.as_str() {
                "disable_ast_parsing" if value == "true" => {
                    self.parse_strategies.retain(|s| *s != ParseStrategy::AstParsing);
                },
                "disable_regex_parsing" if value == "true" => {
                    self.parse_strategies.retain(|s| *s != ParseStrategy::RegexParsing);
                },
                _ => {},
            }
        }
        
        self
    }
    
    /// 解析SQL文本
    pub fn parse_sql(&self, sql: &str, database_type: &DatabaseType) -> Result<ParseResult> {
        let start_time = Instant::now();
        
        // 尝试按配置的策略顺序进行解析
        for strategy in &self.parse_strategies {
            match self.try_parse_with_strategy(sql, database_type, strategy) {
                Ok(mut result) => {
                    // 计算解析时间
                    let parse_time_ms = start_time.elapsed().as_millis() as u64;
                    result.parse_time_ms = parse_time_ms;
                    
                    // 返回解析结果
                    return Ok(result);
                },
                Err(_) => {
                    // 当前策略失败，尝试下一个策略
                    continue;
                },
            }
        }
        
        // 所有策略都失败
        Err(ParseError::SqlParseError("所有解析策略均失败".to_string()))
    }
    
    /// 使用指定策略尝试解析SQL
    fn try_parse_with_strategy(&self, sql: &str, database_type: &DatabaseType, strategy: &ParseStrategy) -> Result<ParseResult> {
        match strategy {
            ParseStrategy::AstParsing => {
                // 使用AST解析器
                self.parse_with_ast(sql, database_type)
            },
            ParseStrategy::RegexParsing => {
                // 使用正则解析器
                self.parse_with_regex(sql, database_type)
            },
        }
    }
    
    /// 使用AST解析器解析SQL
    fn parse_with_ast(&self, sql: &str, database_type: &DatabaseType) -> Result<ParseResult> {
        // 创建一个审计日志结构用于调用SqlParser
        let audit_log = crate::core::types::AuditLog {
            id: "".to_string(),
            timestamp: "".to_string(),
            database_type: database_type.clone(),
            user: None,
            client_ip: None,
            database_name: None,
            sql_text: sql.to_string(),
            execution_time_ms: None,
            rows_affected: None,
            status: "success".to_string(),
        };
        
        // 由于SqlParser没有实现Clone，我们重新创建一个实例
        let mut parser = SqlParser::new(self.ast_parser.get_config().clone());
        parser.parse_audit_log(&audit_log)
    }
    
    /// 使用正则解析器解析SQL
    fn parse_with_regex(&self, sql: &str, database_type: &DatabaseType) -> Result<ParseResult> {
        // 初始化结果集合
        let mut databases = Vec::new();
        let mut schemas = Vec::new();
        let mut tables = Vec::new();
        let mut columns = Vec::new();
        
        // 调用正则解析器
        let success = self.regex_parser.parse_sql(
            sql, 
            &mut databases, 
            &mut schemas, 
            &mut tables, 
            &mut columns
        );
        
        if !success {
            return Err(ParseError::SqlParseError("正则解析失败".to_string()));
        }
        
        // 转换Vec为HashSet以符合ParseResult的类型要求
        let databases_set: HashSet<String> = databases.into_iter().collect();
        let schemas_set: HashSet<String> = schemas.into_iter().collect();
        let tables_set: HashSet<String> = tables.into_iter().collect();
        let columns_set: HashSet<String> = columns.into_iter().collect();
        
        // 构建SqlObject列表
        let objects: Vec<SqlObject> = tables_set.iter().map(|table| SqlObject {
            database: None,
            schema: None,
            table: table.clone(),
            column: None,
            alias: None,
        }).collect();
        
        // 推断操作类型
        let operation_type = self.infer_operation_type(sql);
        
        // 创建并返回解析结果
        Ok(ParseResult {
            database_type: database_type.clone(),
            original_sql: sql.to_string(),
            databases: databases_set,
            schemas: schemas_set,
            tables: tables_set,
            columns: columns_set,
            objects,
            operation_type,
            parse_time_ms: 0, // 将在外部设置
        })
    }
    
    /// 推断SQL操作类型
    fn infer_operation_type(&self, sql: &str) -> OperationType {
        let lower_sql = sql.trim().to_lowercase();
        
        if lower_sql.starts_with("select") {
            OperationType::SELECT
        } else if lower_sql.starts_with("insert") {
            OperationType::INSERT
        } else if lower_sql.starts_with("update") {
            OperationType::UPDATE
        } else if lower_sql.starts_with("delete") {
            OperationType::DELETE
        } else if lower_sql.starts_with("create") {
            OperationType::CREATE
        } else if lower_sql.starts_with("drop") {
            OperationType::DROP
        } else if lower_sql.starts_with("alter") {
            OperationType::ALTER
        } else if lower_sql.starts_with("truncate") {
            OperationType::TRUNCATE
        } else {
            OperationType::OTHER
        }
    }
}