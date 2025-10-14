use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("SQL 解析错误: {0}")]
    SqlParseError(String),
    
    #[error("不支持的数据库类型: {0}")]
    UnsupportedDatabase(String),
    
    #[error("审计日志格式错误: {0}")]
    AuditLogFormatError(String),
    
    #[error("正则表达式错误: {0}")]
    RegexError(String),
    
    #[error("IO 错误: {0}")]
    IoError(String),
    
    #[error("JSON 解析错误: {0}")]
    JsonError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("性能超时: {0}")]
    TimeoutError(String),
}

impl From<sqlparser::parser::ParserError> for ParseError {
    fn from(error: sqlparser::parser::ParserError) -> Self {
        ParseError::SqlParseError(error.to_string())
    }
}

impl From<regex::Error> for ParseError {
    fn from(error: regex::Error) -> Self {
        ParseError::RegexError(error.to_string())
    }
}

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(error: serde_json::Error) -> Self {
        ParseError::JsonError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;