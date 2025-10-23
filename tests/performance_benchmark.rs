use std::collections::HashSet;
use std::time::{Instant, Duration};
use std::thread::available_parallelism;
use std::num::NonZeroUsize;
use sqlop::core::enhanced_parser_improved_optimized::EnhancedSqlParserImprovedOptimized;

// 测试SQL语句示例
const TEST_SQL_QUERIES: &[&str] = &[
    // 简单查询
    "SELECT * FROM users WHERE id = 1",
    "SELECT name, email FROM customers",
    "INSERT INTO products (name, price) VALUES ('Laptop', 999.99)",
    "UPDATE orders SET status = 'completed' WHERE id = 10",
    "DELETE FROM logs WHERE timestamp < '2023-01-01'",
    
    // 复杂查询
    "SELECT u.id, u.name, o.order_id, o.total_amount 
     FROM users u 
     JOIN orders o ON u.id = o.user_id 
     WHERE o.total_amount > 1000 
     ORDER BY o.order_date DESC 
     LIMIT 10",
    
    // 子查询
    "SELECT c.name, 
            (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.id) as order_count
     FROM customers c
     WHERE (SELECT COUNT(*) FROM orders o WHERE o.customer_id = c.id) > 5",
    
    // 聚合函数
    "SELECT department_id, AVG(salary) as avg_salary, MAX(salary) as max_salary
     FROM employees
     GROUP BY department_id
     HAVING AVG(salary) > 5000",
    
    // 带引号标识符
    "SELECT `user-name`, \"email-address\", [full_name] FROM \"user_table\"", 
    
    // 数据库特定语法
    "SELECT /*+ INDEX(users idx_user_id) */ * FROM users",
];

fn main() {
    println!("=== SQL解析引擎性能基准测试 ===");
    println!("可用CPU核心数: {}", available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap()).get());
    
    // 测试参数
    let test_iterations = 10000; // 测试迭代次数
    let batch_size = 1000;      // 批次大小
    
    // 执行基础性能测试
    run_basic_performance_test(test_iterations, batch_size);
    
    // 执行复杂SQL测试
    run_complex_sql_test(1000);
    
    // 执行方言特定测试
    run_dialect_specific_test(1000);
}

fn run_basic_performance_test(iterations: usize, batch_size: usize) {
    println!("\n基础性能测试 ({}次迭代，批次大小: {})
{}", 
             iterations, batch_size, "-".repeat(50));
    
    let parser = EnhancedSqlParserImprovedOptimized::new(None);
    let mut total_duration = Duration::new(0, 0);
    let mut success_count = 0;
    
    // 预热阶段
    println!("预热解析器...");
    for _ in 0..100 {
        for &sql in TEST_SQL_QUERIES {
            let mut databases = Vec::new();
            let mut schemas = Vec::new();
            let mut tables = Vec::new();
            let mut columns = Vec::new();
            parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns);
        }
    }
    
    // 正式测试
    println!("开始测试...");
    let start_time = Instant::now();
    
    for batch in 0..(iterations / batch_size) {
        let batch_start = Instant::now();
        
        for i in 0..batch_size {
            let sql_index = (batch * batch_size + i) % TEST_SQL_QUERIES.len();
            let sql = TEST_SQL_QUERIES[sql_index];
            
            let mut databases = Vec::new();
            let mut schemas = Vec::new();
            let mut tables = Vec::new();
            let mut columns = Vec::new();
            
            if parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
                success_count += 1;
            }
        }
        
        let batch_duration = batch_start.elapsed();
        total_duration += batch_duration;
        
        println!("批次 {}/{}: {}ms (EPS: {:.2})
", 
                batch + 1, iterations / batch_size,
                batch_duration.as_millis(),
                batch_size as f64 / batch_duration.as_secs_f64());
    }
    
    let elapsed = start_time.elapsed();
    let eps = iterations as f64 / elapsed.as_secs_f64();
    
    println!("测试完成!");
    println!("总耗时: {}ms", elapsed.as_millis());
    println!("总解析数: {}", success_count);
    println!("吞吐量: {:.2} EPS (每秒解析数)", eps);
}

fn run_complex_sql_test(iterations: usize) {
    println!("\n复杂SQL性能测试 ({}次迭代)
{}", 
             iterations, "-".repeat(50));
    
    let complex_sql = r#"WITH temp_orders AS (
        SELECT o.order_id, o.customer_id, SUM(oi.quantity * oi.unit_price) as total_amount
        FROM orders o
        JOIN order_items oi ON o.order_id = oi.order_id
        WHERE o.order_date BETWEEN '2023-01-01' AND '2023-12-31'
        GROUP BY o.order_id, o.customer_id
    )
    SELECT 
        c.customer_id, 
        c.customer_name, 
        c.email, 
        c.phone, 
        COUNT(to.order_id) as order_count,
        SUM(to.total_amount) as total_spent,
        AVG(to.total_amount) as avg_order_value,
        MAX(to.total_amount) as max_order_value
    FROM customers c
    JOIN temp_orders to ON c.customer_id = to.customer_id
    WHERE c.country = 'China'
    GROUP BY c.customer_id, c.customer_name, c.email, c.phone
    HAVING COUNT(to.order_id) > 5
    ORDER BY total_spent DESC
    LIMIT 100 OFFSET 0"#;
    
    let parser = EnhancedSqlParserImprovedOptimized::new(None);
    
    // 预热
      for _ in 0..10 {
         let mut databases = Vec::new();
         let mut schemas = Vec::new();
         let mut tables = Vec::new();
         let mut columns = Vec::new();
         parser.parse_sql(complex_sql, &mut databases, &mut schemas, &mut tables, &mut columns);
      }
    
    // 测试
    let start_time = Instant::now();
    
    for _ in 0..iterations {
        let mut databases = Vec::new();
        let mut schemas = Vec::new();
        let mut tables = Vec::new();
        let mut columns = Vec::new();
        parser.parse_sql(complex_sql, &mut databases, &mut schemas, &mut tables, &mut columns);
    }
    
    let elapsed = start_time.elapsed();
    let eps = iterations as f64 / elapsed.as_secs_f64();
    
    println!("复杂SQL测试完成!");
    println!("总耗时: {}ms", elapsed.as_millis());
    println!("吞吐量: {:.2} EPS", eps);
}

fn run_dialect_specific_test(iterations: usize) {
    println!("\n方言特定SQL性能测试 ({}次迭代)
{}", 
             iterations, "-".repeat(50));
    
    let dialect_specific_queries = [
        ("oracle", "SELECT /*+ INDEX(employees emp_idx) */ * FROM employees emp WHERE emp.empno = 7369"),
        ("mysql", "SELECT * FROM `user_data` WHERE `user_id` = 1 FOR UPDATE"),
        ("postgresql", "SELECT * FROM users u JOIN LATERAL (SELECT * FROM orders o WHERE o.user_id = u.id) AS o ON true"),
        ("sqlserver", "SELECT TOP 10 * FROM [dbo].[customers] WHERE [status] = 'active'"),
    ];
    
    // 测试每种方言
    for (dialect_name, sql) in dialect_specific_queries.iter() {
        println!("\n测试方言: {}", dialect_name);
        let parser = EnhancedSqlParserImprovedOptimized::new(Some(dialect_name.to_string()));
        
        // 预热
        for _ in 0..10 {
            let mut databases = Vec::new();
            let mut schemas = Vec::new();
            let mut tables = Vec::new();
            let mut columns = Vec::new();
            parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns);
        }
        
        // 测试
        let start_time = Instant::now();
        
        for _ in 0..iterations {
            let mut databases = Vec::new();
            let mut schemas = Vec::new();
            let mut tables = Vec::new();
            let mut columns = Vec::new();
            parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns);
        }
        
        let elapsed = start_time.elapsed();
        let eps = iterations as f64 / elapsed.as_secs_f64();
        
        println!("方言 {} 测试完成: {}ms, {:.2} EPS", dialect_name, elapsed.as_millis(), eps);
    }
}