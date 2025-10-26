use std::collections::{HashSet, HashMap};
use std::sync::Arc;
use std::time::Instant;

use crate::core::error::{ParseError, Result};
use crate::core::types::{DatabaseType, ParseResult, ParserConfig, SqlObject, OperationType};
use crate::core::ast_parser::AstSqlParser;
use crate::core::regex_parser::RegexSqlParser;

// 定义分层解析策略枚举
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseStrategy {
    AstParsing,      // 使用AstSqlParser（AST解析）
    RegexParsing,    // 使用RegexSqlParser（正则解析）
}

/// 分层SQL解析器实现
/// 作为协调者，在不同解析器之间切换：
/// 1. 首先尝试使用AstSqlParser（AST解析）
/// 2. 如果失败，尝试使用RegexSqlParser（正则解析）
/// 支持配置解析策略顺序和开关
pub struct LayeredSqlParser {
    // AST解析器实例
    ast_parser: AstSqlParser,
    // 正则解析器实例
    regex_parser: RegexSqlParser,
    // 解析策略配置
    parse_strategies: Vec<ParseStrategy>,
}

impl LayeredSqlParser {
    /// 创建新的分层SQL解析器实例
    pub fn new(config: ParserConfig) -> Self {
        // 初始化AST解析器
        let ast_parser = AstSqlParser::new(config.clone());
        
        // 初始化正则解析器
        let regex_parser = RegexSqlParser::new(config).unwrap_or_else(|_| {
            // 如果正则解析器初始化失败，使用默认配置重试
            RegexSqlParser::new(ParserConfig::default()).expect("Failed to initialize regex parser with default config")
        });
        
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
                self.ast_parser.parse_sql(sql, database_type)
            },
            ParseStrategy::RegexParsing => {
                // 使用正则解析器
                self.regex_parser.parse_sql(sql, database_type)
            },
        }
    }
}