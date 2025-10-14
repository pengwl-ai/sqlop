use sqlop::core::{SqlopEngine, DatabaseType};
use sqlop::utils::{SqlValidator, SqlFormatter, PerformanceMonitor};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    sqlop::init_logger();
    
    println!("=== SQL 解析引擎基本使用示例 ===\n");
    
    // 创建引擎实例
    let mut engine = SqlopEngine::default();
    
    // 创建性能监控器
    let mut monitor = PerformanceMonitor::new();
    
    // 示例 SQL 语句
    let test_cases = vec![
        (
            "SELECT name, user FROM dsp.api LIMIT 10;",
            DatabaseType::MySQL,
            "MySQL 查询示例"
        ),
        (
            "\\d testDb; SELECT name, user FROM dsp.api LIMIT 10;",
            DatabaseType::PostgreSQL,
            "PostgreSQL 查询示例"
        ),
        (
            "INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com');",
            DatabaseType::MySQL,
            "MySQL 插入示例"
        ),
        (
            "UPDATE employees SET salary = salary * 1.1 WHERE department = 'IT';",
            DatabaseType::SQLServer,
            "SQL Server 更新示例"
        ),
        (
            "SELECT u.name, u.email, COUNT(o.id) as order_count \
             FROM users u \
             LEFT JOIN orders o ON u.id = o.user_id \
             WHERE u.status = 'active' \
             GROUP BY u.id, u.name, u.email \
             HAVING COUNT(o.id) > 5 \
             ORDER BY order_count DESC \
             LIMIT 100;",
            DatabaseType::PostgreSQL,
            "复杂查询示例"
        ),
    ];
    
    // 解析示例
    println!("1. 基本解析功能演示：");
    println!("{}", "=".repeat(50));
    
    for (sql, db_type, description) in test_cases {
        println!("\n【{}】", description);
        println!("SQL: {}", sql);
        println!("数据库类型: {}", db_type);
        
        // 开始性能监控
        let timer = monitor.start_operation().with_database_type(&db_type.to_string());
        let start_time = Instant::now();
        
        match engine.parse_sql(sql, db_type) {
            Ok(result) => {
                let parse_time = start_time.elapsed();
                timer.finish(true);
                
                println!("✅ 解析成功 (耗时: {:?})", parse_time);
                println!("   数据库: {:?}", result.databases);
                println!("   Schema: {:?}", result.schemas);
                println!("   表: {:?}", result.tables);
                println!("   列: {:?}", result.columns);
                println!("   操作类型: {:?}", result.operation_type);
                println!("   对象数量: {}", result.objects.len());
                
                // 显示对象详情
                if !result.objects.is_empty() {
                    println!("   对象详情:");
                    for obj in &result.objects {
                        println!("     - 表: {}.{}{}", 
                            obj.schema.as_deref().unwrap_or(""),
                            obj.table,
                            obj.alias.as_ref().map(|a| format!(" (别名: {})", a)).unwrap_or_default()
                        );
                    }
                }
            }
            Err(e) => {
                timer.finish(false);
                println!("❌ 解析失败: {}", e);
            }
        }
    }
    
    // 性能统计
    println!("\n\n2. 性能统计：");
    println!("{}", "=".repeat(50));
    println!("{}", monitor.get_summary());
    
    // 验证功能演示
    println!("\n\n3. SQL 验证功能演示：");
    println!("{}", "=".repeat(50));
    
    let validator = SqlValidator::new(DatabaseType::MySQL);
    let test_sql = "SELECT * FROM users WHERE id = 1 OR 1=1";
    
    match validator.validate(test_sql) {
        Ok(result) => {
            println!("验证 SQL: {}", test_sql);
            println!("验证结果: {}", if result.is_valid { "✅ 有效" } else { "❌ 无效" });
            
            if !result.errors.is_empty() {
                println!("错误:");
                for error in &result.errors {
                    println!("  - {}: {}", error.rule_name, error.message);
                }
            }
            
            if !result.warnings.is_empty() {
                println!("警告:");
                for warning in &result.warnings {
                    println!("  - {}: {}", warning.rule_name, warning.message);
                }
            }
            
            if !result.info.is_empty() {
                println!("信息:");
                for info in &result.info {
                    println!("  - {}: {}", info.rule_name, info.message);
                }
            }
        }
        Err(e) => {
            println!("验证失败: {}", e);
        }
    }
    
    // 格式化功能演示
    println!("\n\n4. SQL 格式化功能演示：");
    println!("{}", "=".repeat(50));
    
    let formatter = SqlFormatter::new(DatabaseType::MySQL);
    let unformatted_sql = "select u.name,u.email,count(o.id) as order_count from users u left join orders o on u.id=o.user_id where u.status='active' group by u.id,u.name,u.email having count(o.id)>5 order by order_count desc limit 100";
    
    println!("原始 SQL:");
    println!("{}", unformatted_sql);
    
    match formatter.format(unformatted_sql) {
        Ok(formatted) => {
            println!("\n格式化后 SQL:");
            println!("{}", formatted);
        }
        Err(e) => {
            println!("格式化失败: {}", e);
        }
    }
    
    // 压缩功能演示
    println!("\n\n5. SQL 压缩功能演示：");
    println!("{}", "=".repeat(50));
    
    let verbose_sql = "SELECT   name,  email FROM   users WHERE   status = 'active'   AND   created_at > '2023-01-01'";
    
    println!("原始 SQL:");
    println!("{}", verbose_sql);
    
    match formatter.minify(verbose_sql) {
        Ok(minified) => {
            println!("\n压缩后 SQL:");
            println!("{}", minified);
        }
        Err(e) => {
            println!("压缩失败: {}", e);
        }
    }
    
    // 批量处理演示
    println!("\n\n6. 批量处理演示：");
    println!("{}", "=".repeat(50));
    
    let batch_sql = vec![
        ("SELECT * FROM users".to_string(), DatabaseType::MySQL),
        ("SELECT name FROM products".to_string(), DatabaseType::PostgreSQL),
        ("SELECT id FROM orders".to_string(), DatabaseType::SQLServer),
    ];
    
    let start_time = Instant::now();
    let results = engine.parse_batch_sql(&batch_sql);
    let batch_time = start_time.elapsed();
    
    println!("批量处理 {} 条 SQL，耗时: {:?}", batch_sql.len(), batch_time);
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(parse_result) => {
                println!("  {}. ✅ {} - 表: {:?}", i + 1, parse_result.database_type, parse_result.tables);
            }
            Err(e) => {
                println!("  {}. ❌ 解析失败: {}", i + 1, e);
            }
        }
    }
    
    // 敏感信息检测演示
    println!("\n\n7. 敏感信息检测演示：");
    println!("{}", "=".repeat(50));
    
    let sensitive_sqls = vec![
        "SELECT id_card, phone FROM users WHERE name = '张三'",
        "INSERT INTO users (name, email, password) VALUES ('李四', 'lisi@example.com', '123456')",
        "UPDATE users SET id_card = '110101199001011234' WHERE id = 1",
    ];
    
    for sql in sensitive_sqls {
        println!("检测 SQL: {}", sql);
        
        // 使用配置中的敏感模式进行检测
        let contains_sensitive = sqlop::utils::contains_sensitive_info(sql, &[
            sqlop::core::types::SensitivePattern {
                name: "身份证号".to_string(),
                pattern: r"\\b\\d{17}[\\dXx]\\b".to_string(),
                description: "匹配中国身份证号码".to_string(),
                column_names: vec!["id_card".to_string()],
                table_names: vec!["users".to_string()],
            },
            sqlop::core::types::SensitivePattern {
                name: "手机号".to_string(),
                pattern: r"\\b1[3-9]\\d{9}\\b".to_string(),
                description: "匹配中国手机号码".to_string(),
                column_names: vec!["phone".to_string()],
                table_names: vec!["users".to_string()],
            },
            sqlop::core::types::SensitivePattern {
                name: "密码".to_string(),
                pattern: r"(?i)\\bpassword\\b".to_string(),
                description: "匹配密码相关字段".to_string(),
                column_names: vec!["password".to_string()],
                table_names: vec!["users".to_string()],
            },
        ]);
        
        if contains_sensitive {
            println!("  ⚠️  检测到敏感信息");
        } else {
            println!("  ✅ 未检测到敏感信息");
        }
    }
    
    println!("\n=== 示例演示完成 ===");
    Ok(())
}