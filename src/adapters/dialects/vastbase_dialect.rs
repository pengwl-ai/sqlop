use sqlparser::dialect::Dialect;

/// Vastbase数据库方言实现
#[derive(Debug, Clone)]
pub struct VastbaseDialect;

impl Dialect for VastbaseDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        // Vastbase标识符规则：字母、下划线
        ch.is_alphabetic() || ch == '_'
    }
    
    fn is_identifier_part(&self, ch: char) -> bool {
        self.is_identifier_start(ch) || ch.is_numeric() || ch == '$'
    }
    
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        ch == '"'
    }
}