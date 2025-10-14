use sqlop::SqlopEngine;
use std::time::Instant;

fn main() {
    // 初始化日志
    sqlop::init_logger();
    
    // 创建引擎实例
    let mut engine = SqlopEngine::default();
    let test_sql = "SELECT id, name, email FROM users WHERE status = 'active'";
    let db_type = sqlop::core::types::DatabaseType::MySQL;
    
    // 预热
    println!("预热中...");
    for _ in 0..10000 {
        let _ = engine.parse_sql(test_sql, db_type.clone());
    }
    
    // 测量EPS
    println!("开始测量性能...");
    let duration_seconds = 10; // 测量10秒钟
    let start = Instant::now();
    let mut count = 0;
    
    while start.elapsed().as_secs() < duration_seconds {
        if engine.parse_sql(test_sql, db_type.clone()).is_ok() {
            count += 1;
        }
    }
    
    let elapsed = start.elapsed().as_secs_f64();
    let eps = count as f64 / elapsed;
    
    println!("测试结果:");
    println!("- 总解析次数: {}", count);
    println!("- 总耗时: {:.2} 秒", elapsed);
    println!("- 每秒解析数(EPS): {:.2}", eps);
    
    // 与目标要求比较
    let target_eps = 6000.0;
    if eps >= target_eps {
        println!("✅ 性能满足要求！当前EPS({:.2}) >= 目标EPS({})", eps, target_eps);
    } else {
        println!("❌ 性能未满足要求。当前EPS({:.2}) < 目标EPS({})", eps, target_eps);
    }
}