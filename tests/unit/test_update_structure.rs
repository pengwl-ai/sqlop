extern crate sqlparser;

use sqlparser::ast::Statement;
use sqlparser::dialect::MySqlDialect;
use sqlparser::parser::Parser;

fn main() {
    let sql = "UPDATE users SET email = 'new@example.com' WHERE id = 1";
    let dialect = MySqlDialect {};
    let mut parser = Parser::new(&dialect).try_with_sql(sql).unwrap();
    let statements = parser.parse_statements().unwrap();
    
    println!("Complete statement: {:?}", statements[0]);
    
    // 尝试匹配Update语句并打印其所有字段
    match &statements[0] {
        Statement::Update { .. } => {
            println!("This is an Update statement");
            // 这里我们可以查看UpdateStatement的具体结构
            // 但由于我们不知道具体字段名，我们只能打印整个结构
        },
        _ => println!("Not an Update statement")
    }
}