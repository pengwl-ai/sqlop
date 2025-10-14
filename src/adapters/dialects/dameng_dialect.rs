use sqlparser::dialect::Dialect;

/// Dameng数据库方言实现
#[derive(Debug, Clone)]
pub struct DamengDialect;

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
}