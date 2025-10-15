use sqlparser::dialect::Dialect;
use std::fmt::Debug; 

/// 增强的MySQL方言，支持更多MySQL特有的语法特性 
pub struct EnhancedMySqlDialect {}

impl EnhancedMySqlDialect {
    pub fn new() -> Self {
        Self {}
    }
}

impl Dialect for EnhancedMySqlDialect {
    /// 检查是否是标识符的开始
    fn is_identifier_start(&self, ch: char) -> bool {
        // MySQL支持字母、下划线、$开头
        ch.is_alphabetic() || ch == '_' || ch == '$'
    }

    /// 检查是否是标识符的一部分
    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ch.is_numeric()
    }

    /// 检查是否是分隔标识符的开始字符
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        // MySQL使用反引号作为分隔标识符
        ch == '`'
    }
}

impl Debug for EnhancedMySqlDialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnhancedMySqlDialect").finish()
    }
}