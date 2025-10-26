// JMH基准测试模块
// 实现SQL解析器的性能基准测试功能

use std::time::{Instant, Duration};
use std::collections::HashMap;

use crate::SqlopEngine;
use crate::core::types::{DatabaseType, ParserConfig};

/// SQL基准测试配置
#[derive(Clone)]
pub struct BenchmarkConfig {
    pub iterations: u32,
    pub warmup_iterations: u32,
    pub threads: u32,
    pub timeout_ms: u64,
    pub enable_parallel: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 10,
            warmup_iterations: 5,
            threads: 4,
            timeout_ms: 30000,
            enable_parallel: true,
        }
    }
}

pub struct SqlBenchmark {
    config: BenchmarkConfig,
    parser_config: ParserConfig,
    results: HashMap<String, BenchmarkResult>,
}

impl SqlBenchmark {
    pub fn new(config: BenchmarkConfig, parser_config: ParserConfig) -> Self {
        Self {
            config,
            parser_config,
            results: HashMap::new(),
        }
    }
    
    pub fn run_single_query_benchmark(&mut self, name: &str, sql: &str, db_type: &DatabaseType) -> BenchmarkResult {
        // 预热阶段
        for _ in 0..self.config.warmup_iterations {
            if let Ok(mut engine) = SqlopEngine::new(Some(self.parser_config.clone())) {
                let _ = engine.parse_sql(sql, db_type.clone().clone());
            }
        }
        
        // 计时开始
        let mut successful_runs = 0;
        let mut total_time = Duration::new(0, 0);
        // Removed performance_stats as it's not used with ParseResult
        
        for _ in 0..self.config.iterations {
            let start_time = Instant::now();
            
            // 解析SQL
            let result = match SqlopEngine::new(Some(self.parser_config.clone())) {
                Ok(mut engine) => engine.parse_sql(sql, db_type.clone().clone()),
                  Err(e) => Err(e)
            };
            
            let elapsed = start_time.elapsed();
            total_time += elapsed;
            
            // 记录性能统计信息
            if result.is_ok() {
                successful_runs += 1;
                // ParseResult doesn't have performance_stats field, skip updating statistics
            }
            
            // 检查超时
            if total_time.as_millis() > self.config.timeout_ms as u128 {
                break;
            }
        }
        
        // 计算平均解析时间
        let avg_parse_time = if successful_runs > 0 {
            total_time / successful_runs as u32
        } else {
            Duration::new(0, 0)
        };
        
        // 创建基准测试结果
        let result = BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            successful_runs,
            total_time,
            avg_parse_time,
            error: None,
            // No performance_stats field
        };
        
        // 保存结果
        self.results.insert(name.to_string(), result.clone());
        
        result
    }
    
    pub fn run_batch_query_benchmark(&mut self, name: &str, sql_queries: &[(&str, &DatabaseType)]) -> BenchmarkResult {
        // 预热阶段
        for _ in 0..self.config.warmup_iterations {
            for (sql, db_type) in sql_queries {
                if let Ok(mut engine) = SqlopEngine::new(Some(self.parser_config.clone())) {
                    let _ = engine.parse_sql(sql, db_type.clone().clone());
                }
            }
        }
        
        // 计时开始
        let mut successful_runs = 0;
        let mut total_time = Duration::new(0, 0);
        // Removed performance_stats as it's not used with ParseResult
        
        for _ in 0..self.config.iterations {
            let start_time = Instant::now();
            
            // 批量解析SQL
                let mut batch_success = true;
                for (sql, db_type) in sql_queries {
                    let result = match SqlopEngine::new(Some(self.parser_config.clone())) {
                        Ok(mut engine) => engine.parse_sql(sql, db_type.clone().clone()),
                          Err(e) => Err(e)
                    };
                
                if result.is_ok() {
                    // ParseResult doesn't have performance_stats field, skip updating statistics
                } else {
                    batch_success = false;
                    break;
                }
            }
            
            let elapsed = start_time.elapsed();
            total_time += elapsed;
            
            if batch_success {
                successful_runs += 1;
            }
            
            // 检查超时
            if total_time.as_millis() > self.config.timeout_ms as u128 {
                break;
            }
        }
        
        // 计算平均解析时间
        let avg_parse_time = if successful_runs > 0 {
            total_time / successful_runs as u32
        } else {
            Duration::new(0, 0)
        };
        
        // 创建基准测试结果
        let result = BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            successful_runs,
            total_time,
            avg_parse_time,
            error: None,
            // No performance_stats field
        };
        
        // 保存结果
        self.results.insert(name.to_string(), result.clone());
        
        result
    }
    
    pub fn run_cache_size_benchmark(&mut self, name: &str, sql: &str, db_type: DatabaseType) -> Vec<BenchmarkResult> {
        let cache_sizes = [10, 50, 100, 500, 1000];
        let mut results = Vec::new();
        
        for &cache_size in &cache_sizes {
            // 创建具有特定缓存大小的配置
            let mut config = self.parser_config.clone();
            config.cache_size = cache_size;
            
            // 创建新的基准测试实例
            let mut benchmark = SqlBenchmark::new(self.config.clone(), config);
            
            // 运行测试
            let result_name = format!("{}_cache_{}", name, cache_size);
            let result = benchmark.run_single_query_benchmark(&result_name, sql, &db_type);
            
            // 保存结果
            results.push(result);
            self.results.insert(result_name, results.last().unwrap().clone());
        }
        
        results
    }
    
    pub fn run_parallel_benchmark(&mut self, name: &str, sql_queries: &[(&str, &DatabaseType)]) -> Vec<BenchmarkResult> {
        if !self.config.enable_parallel {
            // 如果未启用并行，则回退到单线程测试
            let result = self.run_batch_query_benchmark(name, sql_queries);
            return vec![result];
        }
        
        let thread_counts = [1, 2, 4, 8];
        let mut results = Vec::new();
        
        for &thread_count in &thread_counts {
            // 设置线程数
            let config = BenchmarkConfig {
                threads: thread_count,
                ..self.config.clone()
            };
            
            // 创建新的基准测试实例
            let mut benchmark = SqlBenchmark::new(config, self.parser_config.clone());
            
            // 运行测试
            let result_name = format!("{}_threads_{}", name, thread_count);
            let result = benchmark.run_batch_query_benchmark(&result_name, sql_queries);
            
            // 保存结果
            results.push(result);
            self.results.insert(result_name, results.last().unwrap().clone());
        }
        
        results
    }
    
    pub fn get_results(&self) -> &HashMap<String, BenchmarkResult> {
        &self.results
    }
    
    pub fn print_results(&self) {
        println!("\n=== SQL Parser Benchmark Results ===");
        println!("Configuration: {}", self.config.format());
        println!("{:<30} {:>15} {:>15} {:>15} {:>15}", 
                 "Benchmark", "Iterations", "Successful", "Total Time", "Avg Time");
        println!("{:-<30} {:-<15} {:-<15} {:-<15} {:-<15}", 
                 "", "", "", "", "");
        
        // 按名称排序并打印结果
        let mut sorted_results: Vec<_> = self.results.iter().collect();
        sorted_results.sort_by_key(|&(name, _)| name);
        
        for (name, result) in sorted_results {
            println!("{:<30} {:>15} {:>15} {:>15} {:>15.2?}", 
                     name,
                     result.iterations,
                     result.successful_runs,
                     format!("{:.2?}", result.total_time),
                     result.avg_parse_time);
            
            // Removed performance statistics printing
        }
        
        println!("=====================================\n");
    }
}

#[derive(Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u32,
    pub successful_runs: u32,
    pub total_time: Duration,
    pub avg_parse_time: Duration,
    pub error: Option<String>,
}

impl BenchmarkResult {
    pub fn error(message: String) -> Self {
        Self {
            name: "error".to_string(),
            iterations: 0,
            successful_runs: 0,
            total_time: Duration::new(0, 0),
            avg_parse_time: Duration::new(0, 0),
            error: Some(message),
            // No performance_stats field
        }
    }
    
    pub fn format(&self) -> String {
        let mut output = format!(
            "Benchmark: {}\n", self.name
        );
        
        output.push_str(&format!("  Iterations: {}\n", self.iterations));
        output.push_str(&format!("  Successful runs: {}\n", self.successful_runs));
        output.push_str(&format!("  Total time: {:.2?}\n", self.total_time));
        output.push_str(&format!("  Average parse time: {:.2?}\n", self.avg_parse_time));
        
        if let Some(error) = &self.error {
            output.push_str(&format!("  Error: {}\n", error));
        }
        
        // Removed performance statistics output

        
        output
    }
}

impl BenchmarkConfig {
    pub fn format(&self) -> String {
        format!(
            "iterations={}, warmup={}, threads={}, timeout={}ms, parallel={}",
            self.iterations,
            self.warmup_iterations,
            self.threads,
            self.timeout_ms,
            self.enable_parallel
        )
    }
}

pub fn generate_standard_benchmark_sql() -> Vec<(&'static str, &'static DatabaseType)> {
    vec![
        // 简单查询
        (
            "SELECT id, name, email FROM users WHERE age > 18",
            &DatabaseType::MySQL
        ),
        // 带有JOIN的查询
        (
            "SELECT u.id, u.name, o.order_id, o.total FROM users u JOIN orders o ON u.id = o.user_id WHERE o.status = 'completed'",
            &DatabaseType::PostgreSQL
        ),
        // 带有子查询的查询
        (
            "SELECT name, (SELECT COUNT(*) FROM orders WHERE orders.user_id = users.id) as order_count FROM users",
            &DatabaseType::SQLServer
        ),
        // 复杂聚合查询
        (
            "SELECT department, COUNT(*) as employee_count, AVG(salary) as avg_salary, MAX(salary) as max_salary FROM employees GROUP BY department HAVING COUNT(*) > 5 ORDER BY avg_salary DESC",
            &DatabaseType::Oracle
        ),
        // 使用窗口函数的查询
        (
            "SELECT id, name, salary, RANK() OVER (PARTITION BY department ORDER BY salary DESC) as rank FROM employees",
            &DatabaseType::PostgreSQL
        ),
        // 带CTE的查询
        (
            "WITH sales_summary AS (SELECT department, SUM(amount) as total_sales FROM sales GROUP BY department) SELECT * FROM sales_summary WHERE total_sales > 10000",
            &DatabaseType::PostgreSQL
        ),
        // INSERT语句
        (
            "INSERT INTO users (name, email, age) VALUES ('John Doe', 'john@example.com', 30), ('Jane Smith', 'jane@example.com', 25)",
            &DatabaseType::MySQL
        ),
        // UPDATE语句
        (
            "UPDATE products SET price = price * 1.1, last_updated = CURRENT_DATE WHERE category = 'electronics' AND in_stock = true",
            &DatabaseType::Oracle
        ),
        // DELETE语句
        (
            "DELETE FROM logs WHERE log_date < DATEADD(month, -6, CURRENT_TIMESTAMP)",
            &DatabaseType::SQLServer
        ),
        // CREATE TABLE语句
        (
            "CREATE TABLE new_employees (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(100) NOT NULL, email VARCHAR(100) UNIQUE, hire_date DATE, salary DECIMAL(10, 2))",
            &DatabaseType::MySQL
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::DatabaseType;
    
    #[test]
    fn test_single_query_benchmark() {
        let config = BenchmarkConfig {
            iterations: 3,  // 减少测试迭代次数以加快测试速度
            warmup_iterations: 1,
            threads: 1,
            timeout_ms: 5000,
            enable_parallel: false,
        };
        
        let parser_config = ParserConfig::default();
        let mut benchmark = SqlBenchmark::new(config, parser_config);
        
        // 运行简单的基准测试
        let result = benchmark.run_single_query_benchmark(
            "test_query",
            "SELECT id, name FROM users WHERE age > 18",
            &DatabaseType::MySQL
        );
        
        // 验证结果
        assert_eq!(result.name, "test_query");
        assert_eq!(result.iterations, 3);
        assert!(result.successful_runs > 0); // 至少有一次成功运行
    }
    
    #[test]
    fn test_batch_query_benchmark() {
        let config = BenchmarkConfig {
            iterations: 2,  // 减少测试迭代次数以加快测试速度
            warmup_iterations: 1,
            threads: 1,
            timeout_ms: 5000,
            enable_parallel: false,
        };
        
        let parser_config = ParserConfig::default();
        let mut benchmark = SqlBenchmark::new(config, parser_config);
        
        // 准备批量查询
        let queries = [
            ("SELECT id, name FROM users", &DatabaseType::MySQL),
            ("SELECT id, title FROM posts", &DatabaseType::PostgreSQL),
        ];
        
        // 运行批量基准测试
        let result = benchmark.run_batch_query_benchmark("test_batch", &queries);
        
        // 验证结果
        assert_eq!(result.name, "test_batch");
        assert_eq!(result.iterations, 2);
        assert!(result.successful_runs > 0); // 至少有一次成功运行
    }
    
    #[test]
    fn test_standard_benchmark_sql() {
        let queries = generate_standard_benchmark_sql();
        
        // 验证生成了足够的测试查询
        assert!(queries.len() >= 5, "Should generate at least 5 test queries");
        
        // 验证每个查询都是有效的
        for (sql, _) in queries {
            assert!(!sql.is_empty(), "SQL query should not be empty");
        }
    }
}