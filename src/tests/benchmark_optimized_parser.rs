use std::collections::HashSet;
use std::time::{Duration, Instant};
use sqlop::core::{EnhancedSqlParserImprovedOptimized, DatabaseType};
use std::thread::available_parallelism;

// 测试用的SQL示例
const TEST_SQLS: &[&str] = &[
    "SELECT id, name, email FROM users WHERE age > 18 AND status = 'active'",
    "INSERT INTO products (name, price, category) VALUES ('Laptop', 1299.99, 'Electronics')",
    "UPDATE orders SET status = 'shipped' WHERE order_id = 12345",
    "DELETE FROM log_entries WHERE created_at < CURRENT_DATE - INTERVAL '30 days'",
    "SELECT t1.id, t1.name, t2.order_date FROM customers t1 JOIN orders t2 ON t1.id = t2.customer_id WHERE t2.total > 1000",
    "SELECT MAX(salary), MIN(salary), AVG(salary) FROM employees GROUP BY department_id HAVING COUNT(*) > 5",
    "WITH recent_orders AS (SELECT * FROM orders WHERE order_date > CURRENT_DATE - INTERVAL '7 days') SELECT c.name, COUNT(ro.order_id) AS order_count FROM customers c JOIN recent_orders ro ON c.id = ro.customer_id GROUP BY c.name",
    "SELECT id, name FROM products WHERE price > (SELECT AVG(price) FROM products)",
    "SELECT * FROM (SELECT id, name, ROW_NUMBER() OVER (PARTITION BY department_id ORDER BY salary DESC) AS rn FROM employees) WHERE rn <= 3",
    "SELECT CONCAT(first_name, ' ', last_name) AS full_name, email FROM users WHERE status = 'active'",
    "-- 测试注释SELECT id, name FROM users; -- 行尾注释",
    "SELECT /* 这是一个多行注释 */ id, name FROM users",
    "SELECT `id`, `name`, `email` FROM `users` WHERE `age` > 18",
    "SELECT [id], [name], [email] FROM [users] WHERE [age] > 18",
    "SELECT \"id\", \"name\", \"email\" FROM \"users\" WHERE \"age\" > 18",
    "SELECT dbo.GetFullName(first_name, last_name) AS full_name FROM employees",
    "SELECT t1.id, t2.name FROM schema1.table1 t1 JOIN schema2.table2 t2 ON t1.id = t2.ref_id",
    "SELECT db1.schema1.table1.id, db2.schema2.table2.name FROM db1.schema1.table1 JOIN db2.schema2.table2 ON db1.schema1.table1.id = db2.schema2.table2.ref_id",
    "SELECT * FROM table1 WHERE id IN (SELECT id FROM table2 WHERE status = 'active')",
    "SELECT CASE WHEN status = 'active' THEN 'Active User' ELSE 'Inactive User' END AS user_status, COUNT(*) AS count FROM users GROUP BY status",
];

// 复杂SQL测试用例
const COMPLEX_SQLS: &[&str] = &[
    "SELECT \
        a.*, \
        b.order_count, \
        c.product_name, \
        ROW_NUMBER() OVER (PARTITION BY a.department_id ORDER BY a.salary DESC) AS rank_in_dept \
    FROM \
        employees a \
        JOIN (SELECT employee_id, COUNT(*) AS order_count FROM orders GROUP BY employee_id) b ON a.id = b.employee_id \
        LEFT JOIN products c ON a.favorite_product_id = c.id \
    WHERE \
        a.status = 'active' \
        AND a.salary > (SELECT AVG(salary) FROM employees WHERE department_id = a.department_id) \
        AND EXISTS (SELECT 1 FROM employee_skills WHERE employee_id = a.id AND skill_id = 123) \
    GROUP BY \
        a.id, a.name, a.department_id, a.salary, a.status, a.favorite_product_id, \
        b.order_count, c.product_name \
    HAVING \
        COUNT(DISTINCT b.order_id) > 5 \
    ORDER BY \
        a.department_id, rank_in_dept",
    "WITH \
        recent_sales AS (SELECT * FROM sales WHERE sale_date > CURRENT_DATE - INTERVAL '90 days'), \
        top_products AS (SELECT product_id, SUM(amount) AS total_sales FROM recent_sales GROUP BY product_id ORDER BY total_sales DESC LIMIT 10), \
        product_details AS (SELECT p.*, c.category_name, b.brand_name FROM products p JOIN categories c ON p.category_id = c.id JOIN brands b ON p.brand_id = b.id) \
    SELECT \
        pd.product_name, \
        pd.category_name, \
        pd.brand_name, \
        ts.total_sales, \
        LAG(ts.total_sales) OVER (ORDER BY ts.total_sales DESC) AS prev_sales, \
        LEAD(ts.total_sales) OVER (ORDER BY ts.total_sales DESC) AS next_sales, \
        ROUND((ts.total_sales - LAG(ts.total_sales) OVER (ORDER BY ts.total_sales DESC)) / LAG(ts.total_sales) OVER (ORDER BY ts.total_sales DESC) * 100, 2) AS growth_percentage \
    FROM \
        top_products ts \
        JOIN product_details pd ON ts.product_id = pd.id \
    ORDER BY \
        ts.total_sales DESC",
];

// 方言特定的SQL测试用例
const DIALECT_SPECIFIC_SQLS: &[(&str, &str)] = &[
    ("gaussdb", "SELECT id, ST_AsText(geom) FROM spatial_table WHERE ST_DWithin(geom, ST_MakePoint(121.4737, 31.2304), 1000)")
];

fn main() {
    println!("=== SQL解析引擎性能测试 ===");
    println!("可用CPU核心数: {}", available_parallelism().unwrap());
    
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

fn run_basic_performance_test(iterations: u64, batch_size: u64) {
    println!("\n=== 基础SQL性能测试 ===");
    println!("总测试次数: {}", iterations);
    
    let parser = EnhancedSqlParserImprovedOptimized::new(None);
    let total_sqls = TEST_SQLS.len() as u64;
    let total_queries = iterations;
    
    let start_time = Instant::now();
    
    let mut databases = HashSet::new();
    let mut schemas = HashSet::new();
    let mut tables = HashSet::new();
    let mut columns = HashSet::new();
    
    let mut success_count = 0;
    let mut batch = 0;
    
    for i in 0..total_queries {
        // 循环使用测试SQL
        let sql = TEST_SQLS[(i % total_sqls) as usize];
        
        // 清空集合
        databases.clear();
        schemas.clear();
        tables.clear();
        columns.clear();
        
        // 解析SQL
        if parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
            success_count += 1;
        }
        
        // 打印批次信息
        if (i + 1) % batch_size == 0 {
            batch += 1;
            let elapsed = start_time.elapsed().as_secs_f64();
            let processed = i + 1;
            let eps = processed as f64 / elapsed;
            println!("批次 {}: 已处理 {} 条SQL, EPS: {:.2}", batch, processed, eps);
        }
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let eps = total_queries as f64 / elapsed_secs;
    
    println!("\n测试完成!");
    println!("总耗时: {:.2} 秒", elapsed_secs);
    println!("成功解析: {}/{} ({:.2}%)", success_count, total_queries, 
             (success_count as f64 / total_queries as f64) * 100.0);
    println!("性能: {:.2} EPS", eps);
}

fn run_complex_sql_test(iterations: u64) {
    println!("\n=== 复杂SQL性能测试 ===");
    println!("总测试次数: {}", iterations);
    
    let parser = EnhancedSqlParserImprovedOptimized::new(None);
    let total_sqls = COMPLEX_SQLS.len() as u64;
    let total_queries = iterations;
    
    let start_time = Instant::now();
    
    let mut databases = HashSet::new();
    let mut schemas = HashSet::new();
    let mut tables = HashSet::new();
    let mut columns = HashSet::new();
    
    let mut success_count = 0;
    
    for i in 0..total_queries {
        // 循环使用复杂测试SQL
        let sql = COMPLEX_SQLS[(i % total_sqls) as usize];
        
        // 清空集合
        databases.clear();
        schemas.clear();
        tables.clear();
        columns.clear();
        
        // 解析SQL
        if parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
            success_count += 1;
        }
        
        // 打印进度
        if (i + 1) % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let processed = i + 1;
            let eps = processed as f64 / elapsed;
            println!("已处理 {} 条复杂SQL, EPS: {:.2}", processed, eps);
        }
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let eps = total_queries as f64 / elapsed_secs;
    
    println!("\n测试完成!");
    println!("总耗时: {:.2} 秒", elapsed_secs);
    println!("成功解析: {}/{} ({:.2}%)", success_count, total_queries, 
             (success_count as f64 / total_queries as f64) * 100.0);
    println!("性能: {:.2} EPS", eps);
}

fn run_dialect_specific_test(iterations: u64) {
    println!("\n=== 方言特定SQL性能测试 ===");
    println!("总测试次数: {}", iterations);
    
    let total_dialect_sqls = DIALECT_SPECIFIC_SQLS.len() as u64;
    let total_queries = iterations;
    
    let start_time = Instant::now();
    
    let mut success_count = 0;
    
    for i in 0..total_queries {
        // 循环使用方言特定测试SQL
        let (dialect, sql) = DIALECT_SPECIFIC_SQLS[(i % total_dialect_sqls) as usize];
        let parser = EnhancedSqlParserImprovedOptimized::new(Some(dialect.to_string()));
        
        let mut databases = HashSet::new();
        let mut schemas = HashSet::new();
        let mut tables = HashSet::new();
        let mut columns = HashSet::new();
        
        // 解析SQL
        if parser.parse_sql(sql, &mut databases, &mut schemas, &mut tables, &mut columns) {
            success_count += 1;
        }
        
        // 打印进度
        if (i + 1) % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let processed = i + 1;
            let eps = processed as f64 / elapsed;
            println!("已处理 {} 条方言特定SQL, EPS: {:.2}", processed, eps);
        }
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let eps = total_queries as f64 / elapsed_secs;
    
    println!("\n测试完成!");
    println!("总耗时: {:.2} 秒", elapsed_secs);
    println!("成功解析: {}/{} ({:.2}%)", success_count, total_queries, 
             (success_count as f64 / total_queries as f64) * 100.0);
    println!("性能: {:.2} EPS", eps);
}