use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlop::core::{SqlopEngine, DatabaseType};
use sqlop::utils::PerformanceMonitor;
use std::time::Duration;

fn benchmark_sql_parsing(c: &mut Criterion) {
    let mut engine = SqlopEngine::default();
    let mut monitor = PerformanceMonitor::new();
    
    // 测试不同复杂度的 SQL 语句
    let test_cases = vec![
        ("简单查询", "SELECT * FROM users", DatabaseType::MySQL),
        ("中等查询", "SELECT u.name, u.email, COUNT(o.id) as order_count FROM users u LEFT JOIN orders o ON u.id = o.user_id WHERE u.status = 'active' GROUP BY u.id, u.name, u.email", DatabaseType::PostgreSQL),
        ("复杂查询", "SELECT u.name, u.email, o.order_date, p.product_name, oi.quantity, oi.price FROM users u JOIN orders o ON u.id = o.user_id JOIN order_items oi ON o.id = oi.order_id JOIN products p ON oi.product_id = p.id WHERE u.status = 'active' AND o.order_date > '2023-01-01' AND p.category = 'electronics' ORDER BY o.order_date DESC, u.name LIMIT 100", DatabaseType::SQLServer),
        ("插入语句", "INSERT INTO users (id, name, email, created_at) VALUES (1, 'John Doe', 'john@example.com', NOW())", DatabaseType::MySQL),
        ("更新语句", "UPDATE users SET email = 'newemail@example.com', updated_at = NOW() WHERE id = 1 AND status = 'active'", DatabaseType::PostgreSQL),
        ("删除语句", "DELETE FROM users WHERE id IN (SELECT user_id FROM inactive_users WHERE last_login < '2023-01-01')", DatabaseType::SQLServer),
    ];
    
    let mut group = c.benchmark_group("sql_parsing");
    
    for (name, sql, db_type) in test_cases {
        group.bench_with_input(BenchmarkId::new("parse", name), sql, |b, sql| {
            b.iter(|| {
                let timer = monitor.start_operation().with_database_type(&db_type.to_string());
                let result = engine.parse_sql(black_box(sql), black_box(db_type.clone()));
                timer.finish(result.is_ok());
                result
            })
        });
    }
    
    group.finish();
}

fn benchmark_batch_parsing(c: &mut Criterion) {
    let mut engine = SqlopEngine::default();
    let mut monitor = PerformanceMonitor::new();
    
    // 准备批量测试数据
    let batch_sizes = vec![10, 50, 100, 500, 1000];
    let base_sql = "SELECT * FROM users WHERE id = ";
    
    let mut group = c.benchmark_group("batch_parsing");
    
    for size in batch_sizes {
        let batch_sql: Vec<(String, DatabaseType)> = (0..size)
            .map(|i| (format!("{}{}", base_sql, i), DatabaseType::MySQL))
            .collect();
        
        group.bench_with_input(BenchmarkId::new("batch", size), &size, |b, _| {
            b.iter(|| {
                let timer = monitor.start_operation().with_database_type("batch");
                let results = engine.parse_batch_sql(black_box(&batch_sql));
                let success_count = results.iter().filter(|r| r.is_ok()).count();
                timer.finish(success_count == batch_sql.len());
                results
            })
        });
    }
    
    group.finish();
}

fn benchmark_database_types(c: &mut Criterion) {
    let mut engine = SqlopEngine::default();
    let mut monitor = PerformanceMonitor::new();
    
    let test_sql = "SELECT u.name, u.email FROM users u WHERE u.status = 'active' ORDER BY u.created_at DESC LIMIT 100";
    let database_types = vec![
        ("MySQL", DatabaseType::MySQL),
        ("PostgreSQL", DatabaseType::PostgreSQL),
        ("SQLServer", DatabaseType::SQLServer),
        ("Oracle", DatabaseType::Oracle),
        ("Hive", DatabaseType::Hive),
    ];
    
    let mut group = c.benchmark_group("database_types");
    
    for (name, db_type) in database_types {
        group.bench_with_input(BenchmarkId::new("db_type", name), &db_type, |b, db_type| {
            b.iter(|| {
                let timer = monitor.start_operation().with_database_type(&name);
                let result = engine.parse_sql(black_box(test_sql), black_box(db_type.clone()));
                timer.finish(result.is_ok());
                result
            })
        });
    }
    
    group.finish();
}

fn benchmark_performance_monitoring(c: &mut Criterion) {
    let mut monitor = PerformanceMonitor::new();
    
    let mut group = c.benchmark_group("performance_monitoring");
    
    group.bench_function("record_operation", |b| {
        b.iter(|| {
            let timer = monitor.start_operation();
            std::thread::sleep(Duration::from_millis(1));
            timer.finish(true);
        })
    });
    
    group.bench_function("get_metrics", |b| {
        b.iter(|| {
            black_box(monitor.get_metrics());
        })
    });
    
    group.bench_function("get_percentile", |b| {
        b.iter(|| {
            black_box(monitor.get_percentile(95.0));
        })
    });
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("engine_creation", |b| {
        b.iter(|| {
            let engine = SqlopEngine::default();
            black_box(engine);
        })
    });
    
    group.bench_function("large_sql_parsing", |b| {
        let mut engine = SqlopEngine::default();
        let large_sql = "SELECT ".to_string() + 
            &"column".to_string() + 
            &(0..1000).map(|i| format!(", col{}", i)).collect::<String>() + 
            " FROM large_table";
        
        b.iter(|| {
            let result = engine.parse_sql(black_box(&large_sql), black_box(DatabaseType::MySQL));
            black_box(result);
        })
    });
    
    group.finish();
}

fn benchmark_eps_performance(c: &mut Criterion) {
    let mut engine = SqlopEngine::default();
    let test_sql = "SELECT id, name, email FROM users WHERE status = 'active'";
    
    let mut group = c.benchmark_group("eps_performance");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("eps_test", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let mut count = 0;
            
            while start.elapsed().as_secs() < 1 {
                let result = engine.parse_sql(black_box(test_sql), black_box(DatabaseType::MySQL));
                if result.is_ok() {
                    count += 1;
                }
            }
            
            black_box(count);
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_sql_parsing,
    benchmark_batch_parsing,
    benchmark_database_types,
    benchmark_performance_monitoring,
    benchmark_memory_usage,
    benchmark_eps_performance
);
criterion_main!(benches);