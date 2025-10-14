use sqlparser::dialect::Dialect;

/// Oracle数据库方言实现
#[derive(Debug, Clone)]
pub struct OracleDialect;

impl Dialect for OracleDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        // Oracle标识符规则：字母、下划线、美元符号、井号
        ch.is_alphabetic() || ch == '_' || ch == '$' || ch == '#'
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ch.is_numeric()
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
}