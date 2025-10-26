use sqlop::core::{LayeredSqlParser, ParserConfig}; use sqlop::core::types::{DatabaseType}; use std::collections::HashMap;

#[test]
fn test_layered_parser_mysql() {
    // 创建默认配置
    let config = ParserConfig::default();
    
    // 创建分层解析器实例
    let parser = LayeredSqlParser::new(config);
    
    // MySQL示例SQL
    let sql = "SELECT id, name, email FROM users WHERE age > 18";
    
    // 解析SQL
    match parser.parse_sql(sql, &DatabaseType::MySQL) {
        Ok(result) => {
            println!("MySQL解析成功!");
            println!("原始SQL: {}", result.original_sql);
            println!("表: {:?}", result.tables);
            println!("解析时间: {}ms", result.parse_time_ms);
            
            // 验证解析结果
            assert!(result.tables_contains("users"));
            assert_eq!(result.operation_type, sqlop::core::types::OperationType::SELECT);
        },
        Err(e) => {
            panic!("MySQL解析失败: {:?}", e);
        }
    }
}

#[test]
fn test_layered_parser_postgresql() {
    // 创建默认配置
    let config = ParserConfig::default();
    
    // 创建分层解析器实例
    let parser = LayeredSqlParser::new(config);
    
    // PostgreSQL示例SQL
    let sql = "SELECT u.id, u.name, o.order_date FROM public.users u JOIN public.orders o ON u.id = o.user_id";
    
    // 解析SQL
    match parser.parse_sql(sql, &DatabaseType::PostgreSQL) {
        Ok(result) => {
            println!("PostgreSQL解析成功!");
            println!("原始SQL: {}", result.original_sql);
            println!("表: {:?}", result.tables);
            println!("模式: {:?}", result.schemas);
            println!("解析时间: {}ms", result.parse_time_ms);
            
            // 验证解析结果
            assert!(result.tables_contains("users") || result.tables_contains("u") || result.tables_contains("orders") || result.tables_contains("o"));
            assert_eq!(result.operation_type, sqlop::core::types::OperationType::SELECT);
        },
        Err(e) => {
            panic!("PostgreSQL解析失败: {:?}", e);
        }
    }
}

#[test]
fn test_layered_parser_configuration() {
    // 创建默认配置
    let config = ParserConfig::default();
    
    // 创建配置选项
    let mut options = HashMap::new();
    options.insert("disable_ast_parsing".to_string(), "true".to_string());
    
    // 创建并配置分层解析器实例
    let parser = LayeredSqlParser::new(config).configure(&options);
    
    // 测试SQL
    let sql = "SELECT id FROM test_table";
    
    // 即使禁用了AST解析，应该还能通过正则降级解析
    match parser.parse_sql(sql, &DatabaseType::MySQL) {
        Ok(result) => {
            println!("禁用AST解析后依然解析成功!");
            println!("表: {:?}", result.tables);
        },
        Err(e) => {
            panic!("禁用AST解析后解析失败: {:?}", e);
        }
    }
}

#[test]
fn test_layered_parser_regex_fallback() {
    println!("测试正则表达式降级解析...");
    
    // 使用默认配置创建分层解析器
    let mut config = ParserConfig::default();
    let mut layered_parser = LayeredSqlParser::new(config);
    
    // 使用一些复杂SQL进行测试
    let complex_sql = "INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com');";
    
    let result = layered_parser.parse_sql(complex_sql, &DatabaseType::MySQL);
    assert!(result.is_ok(), "正则表达式解析失败: {:?}", result);
    
    let parse_result = result.unwrap();
    println!("正则表达式解析结果 - 表: {:?}", parse_result.tables);
    
    // 验证结果中包含users表
    assert!(parse_result.tables.contains(&"users".to_string()));
}