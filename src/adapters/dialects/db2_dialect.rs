use sqlparser::dialect::Dialect;

/// DB2数据库方言实现
#[derive(Debug, Clone)]
pub struct DB2Dialect;

impl Dialect for DB2Dialect {
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