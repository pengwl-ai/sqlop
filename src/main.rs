use sqlop::core::{SqlopEngine, DatabaseType};
use sqlop::adapters::AdapterManager;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("SQL 解析引擎启动...");
    
    // 创建引擎实例
    let mut engine = SqlopEngine::default();
    
    // 创建适配器管理器
    let adapter_manager = AdapterManager::new();
    
    println!("支持的数据库类型:");
    for db_type in adapter_manager.list_supported_databases() {
        println!("- {}", db_type);
    }
    
    // 测试示例
    let test_cases = vec![
        (
            "SELECT name, user FROM dsp.api LIMIT 10;",
            DatabaseType::MySQL,
        ),
        (
            "\\d testDb; SELECT name, user FROM dsp.api LIMIT 10;",
            DatabaseType::PostgreSQL,
        ),
        (
            "INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com');",
            DatabaseType::MySQL,
        ),
        (
            "UPDATE employees SET salary = salary * 1.1 WHERE department = 'IT';",
            DatabaseType::SQLServer,
        ),
    ];
    
    println!("\n开始解析测试...");
    
    for (sql, db_type) in test_cases {
        println!("\n解析 SQL ({}):", db_type);
        println!("SQL: {}", sql);
        
        let start_time = Instant::now();
        match engine.parse_sql(sql, db_type) {
            Ok(result) => {
                let parse_time = start_time.elapsed();
                println!("解析成功 (耗时: {:?})", parse_time);
                println!("数据库: {:?}", result.databases);
                println!("Schema: {:?}", result.schemas);
                println!("表: {:?}", result.tables);
                println!("列: {:?}", result.columns);
                println!("操作类型: {:?}", result.operation_type);
            }
            Err(e) => {
                println!("解析失败: {}", e);
            }
        }
    }
    
    // 性能测试
    println!("\n开始性能测试...");
    let performance_sql = "SELECT id, name, email FROM users WHERE status = 'active' ORDER BY created_at DESC LIMIT 100;";
    let iterations = 1000;
    
    let start_time = Instant::now();
    for _ in 0..iterations {
        if let Err(e) = engine.parse_sql(performance_sql, DatabaseType::MySQL) {
            println!("性能测试失败: {}", e);
            break;
        }
    }
    let total_time = start_time.elapsed();
    let eps = iterations as f64 / total_time.as_secs_f64();
    
    println!("性能测试结果:");
    println!("迭代次数: {}", iterations);
    println!("总耗时: {:?}", total_time);
    println!("EPS: {:.2}", eps);
    
    // 显示引擎统计信息
    let stats = engine.get_performance_stats();
    println!("\n引擎统计信息:");
    println!("缓存大小: {}", stats.cache_size);
    println!("最大解析时间: {}ms", stats.config.max_parse_time_ms);
    println!("启用缓存: {}", stats.config.enable_cache);
    println!("启用并行: {}", stats.config.enable_parallel);
    
    Ok(())
}