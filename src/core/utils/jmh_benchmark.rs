// JMH基准测试模块
// 实现SQL解析器的性能基准测试功能

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::parser::SqlParser;
use crate::core::types::{AuditLog, DatabaseType, ParserConfig, PerformanceStats};

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

/// SQL基准测试工具
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
    
    /// 运行单个SQL查询的基准测试
    pub fn run_single_query_benchmark(&mut self, name: &str, sql: &str, db_type: DatabaseType) -> BenchmarkResult {
        let mut parser = SqlParser::new(self.parser_config.clone());
        
        // 预热阶段
        println!("预热阶段: {}", name);
        for _ in 0..self.config.warmup_iterations {
            let audit_log = AuditLog {
                sql_text: sql.to_string(),
                database_type: db_type.clone(),
                ..Default::default()
            };
            
            if parser.parse_audit_log(&audit_log).is_err() {
                return BenchmarkResult::error("解析失败".to_string());
            }
        }
        
        // 重置性能统计
        parser = SqlParser::new(self.parser_config.clone());
        
        // 测试阶段
        println!("测试阶段: {}", name);
        let start_time = Instant::now();
        let mut total_parse_time = Duration::from_nanos(0);
        let mut successful_runs = 0;
        
        for _ in 0..self.config.iterations {
            let audit_log = AuditLog {
                sql_text: sql.to_string(),
                database_type: db_type.clone(),
                ..Default::default()
            };
            
            let parse_start = Instant::now();
            let result = parser.parse_audit_log(&audit_log);
            let parse_time = parse_start.elapsed();
            
            if result.is_ok() {
                total_parse_time += parse_time;
                successful_runs += 1;
            }
            
            // 检查超时
            if start_time.elapsed().as_millis() > self.config.timeout_ms.into() {
                return BenchmarkResult::error("基准测试超时".to_string());
            }
        }
        
        // 计算统计数据
        let total_time = start_time.elapsed();
        let avg_parse_time = if successful_runs > 0 {
            total_parse_time / successful_runs
        } else {
            Duration::from_nanos(0)
        };
        
        let result = BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            successful_runs,
            total_time,
            avg_parse_time,
            error: None,
            performance_stats: parser.get_performance_stats(),
        };
        
        // 保存结果
        self.results.insert(name.to_string(), result.clone());
        
        result
    }
    
    /// 运行批量SQL查询的基准测试
    pub fn run_batch_query_benchmark(&mut self, name: &str, sql_queries: &[(&str, DatabaseType)]) -> BenchmarkResult {
        let mut parser = SqlParser::new(self.parser_config.clone());
        
        // 准备审计日志
        let audit_logs: Vec<AuditLog> = sql_queries.iter()
            .map(|(sql, db_type)| AuditLog {
                sql_text: sql.to_string(),
                database_type: db_type.clone(),
                ..Default::default()
            })
            .collect();
        
        // 预热阶段
        println!("批量预热阶段: {}", name);
        for _ in 0..self.config.warmup_iterations {
            let _ = parser.parse_batch(&audit_logs);
        }
        
        // 重置解析器
        parser = SqlParser::new(self.parser_config.clone());
        
        // 测试阶段
        println!("批量测试阶段: {}", name);
        let start_time = Instant::now();
        
        let mut total_parse_time = Duration::from_nanos(0);
        let mut successful_runs = 0;
        
        for _ in 0..self.config.iterations {
            let parse_start = Instant::now();
            let results = parser.parse_batch(&audit_logs);
            let parse_time = parse_start.elapsed();
            
            // 检查是否所有解析都成功
            if results.iter().all(|r| r.is_ok()) {
                total_parse_time += parse_time;
                successful_runs += 1;
            }
            
            // 检查超时
            if start_time.elapsed().as_millis() > self.config.timeout_ms.into() {
                return BenchmarkResult::error("基准测试超时".to_string());
            }
        }
        
        // 计算统计数据
        let total_time = start_time.elapsed();
        let avg_parse_time = if successful_runs > 0 {
            total_parse_time / successful_runs
        } else {
            Duration::from_nanos(0)
        };
        
        let result = BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.iterations,
            successful_runs,
            total_time,
            avg_parse_time,
            error: None,
            performance_stats: parser.get_performance_stats(),
        };
        
        // 保存结果
        self.results.insert(name.to_string(), result.clone());
        
        result
    }
    
    /// 运行不同缓存大小的基准测试
    pub fn run_cache_size_benchmark(&mut self, name: &str, sql: &str, db_type: DatabaseType) -> Vec<BenchmarkResult> {
        let cache_sizes = vec![0, 10, 100, 500, 1000, 5000];
        let mut results = Vec::new();
        
        for cache_size in cache_sizes {
            let mut config = self.parser_config.clone();
            config.cache_size = cache_size;
            config.enable_cache = cache_size > 0;
            
            let benchmark_name = format!("{}_cache_{}", name, cache_size);
            println!("运行缓存大小基准测试: {}", benchmark_name);
            
            let mut benchmark = SqlBenchmark::new(self.config.clone(), config);
            let result = benchmark.run_single_query_benchmark(&benchmark_name, sql, db_type.clone());
            results.push(result);
        }
        
        results
    }
    
    /// 运行并行性能基准测试
    pub fn run_parallel_benchmark(&mut self, name: &str, sql_queries: &[(&str, DatabaseType)]) -> Vec<BenchmarkResult> {
        let parallel_options = vec![false, true];
        let mut results = Vec::new();
        
        for parallel in parallel_options {
            let mut config = self.parser_config.clone();
            config.enable_parallel = parallel;
            
            let benchmark_name = format!("{}_parallel_{}", name, parallel);
            println!("运行并行基准测试: {}", benchmark_name);
            
            let mut benchmark = SqlBenchmark::new(self.config.clone(), config);
            let result = benchmark.run_batch_query_benchmark(&benchmark_name, sql_queries);
            results.push(result);
        }
        
        results
    }
    
    /// 获取所有基准测试结果
    pub fn get_results(&self) -> &HashMap<String, BenchmarkResult> {
        &self.results
    }
    
    /// 打印所有基准测试结果
    pub fn print_results(&self) {
        println!("\n=== 基准测试结果 ===\n");
        
        for (name, result) in &self.results {
            println!("测试名称: {}", name);
            println!("总迭代次数: {}", result.iterations);
            println!("成功运行次数: {}", result.successful_runs);
            println!("总耗时: {:.2?}", result.total_time);
            println!("平均解析时间: {:.2?}", result.avg_parse_time);
            println!("性能统计:");
            println!("  总解析次数: {}", result.performance_stats.parse_count);
            println!("  总解析时间(ms): {}", result.performance_stats.total_parse_time_ms);
            println!("  缓存命中次数: {}", result.performance_stats.cache_hits);
            println!("  缓存未命中次数: {}", result.performance_stats.cache_misses);
            println!("  当前缓存大小: {}", result.performance_stats.cache_size);
            
            if let Some(error) = &result.error {
                println!("错误: {}", error);
            }
            
            println!("------------------------------");
        }
    }
}

/// 基准测试结果
#[derive(Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u32,
    pub successful_runs: u32,
    pub total_time: Duration,
    pub avg_parse_time: Duration,
    pub error: Option<String>,
    pub performance_stats: PerformanceStats,
}

impl BenchmarkResult {
    /// 创建错误结果
    pub fn error(message: String) -> Self {
        Self {
            name: "error".to_string(),
            iterations: 0,
            successful_runs: 0,
            total_time: Duration::from_nanos(0),
            avg_parse_time: Duration::from_nanos(0),
            error: Some(message),
            performance_stats: PerformanceStats::default(),
        }
    }
    
    /// 格式化输出结果
    pub fn format(&self) -> String {
        let mut output = format!("测试: {}\n", self.name);
        output.push_str(&format!("迭代次数: {}\n", self.iterations));
        output.push_str(&format!("成功次数: {}\n", self.successful_runs));
        output.push_str(&format!("总耗时: {:.2?}\n", self.total_time));
        output.push_str(&format!("平均时间: {:.2?}\n", self.avg_parse_time));
        
        if let Some(error) = &self.error {
            output.push_str(&format!("错误: {}\n", error));
        }
        
        output.push_str("性能统计:\n");
        output.push_str(&format!("  解析次数: {}\n", self.performance_stats.parse_count));
        output.push_str(&format!("  解析时间(ms): {}\n", self.performance_stats.total_parse_time_ms));
        output.push_str(&format!("  缓存命中率: {:.2}%\n", 
            if self.performance_stats.parse_count > 0 {
                (self.performance_stats.cache_hits as f64 / self.performance_stats.parse_count as f64) * 100.0
            } else { 0.0 }
        ));
        
        output
    }
}

/// 生成标准基准测试SQL集
pub fn generate_standard_benchmark_sql() -> Vec<(&'static str, DatabaseType)> {
    vec![
        // 简单查询
        (
            "SELECT id, name, email FROM users WHERE status = 'active'",
            DatabaseType::MySQL
        ),
        // 带JOIN的查询
        (
            "SELECT u.id, u.name, o.order_id, o.total_amount FROM users u JOIN orders o ON u.id = o.user_id WHERE u.status = 'active'",
            DatabaseType::PostgreSQL
        ),
        // 带聚合函数的查询
        (
            "SELECT department_id, COUNT(*) as employee_count, AVG(salary) as avg_salary FROM employees GROUP BY department_id HAVING COUNT(*) > 10",
            DatabaseType::SQLServer
        ),
        // 带子查询的查询
        (
            "SELECT id, name FROM products WHERE category_id IN (SELECT id FROM categories WHERE parent_id = 5) AND price > (SELECT AVG(price) FROM products)",
            DatabaseType::MySQL
        ),
        // 复杂的多表JOIN
        (
            "SELECT c.id, c.name, o.order_date, o.total_amount, p.name as product_name FROM customers c JOIN orders o ON c.id = o.customer_id JOIN order_items oi ON o.id = oi.order_id JOIN products p ON oi.product_id = p.id WHERE c.status = 'premium' AND o.order_date > '2023-01-01'",
            DatabaseType::PostgreSQL
        ),
        // 数据修改语句
        (
            "UPDATE users SET last_login = CURRENT_TIMESTAMP, login_count = login_count + 1 WHERE id = 123",
            DatabaseType::MySQL
        ),
        // 复杂的WHERE条件
        (
            "SELECT * FROM log_entries WHERE timestamp BETWEEN '2023-01-01' AND '2023-01-31' AND (status_code = 404 OR status_code = 500) AND user_agent LIKE '%Chrome%'",
            DatabaseType::SQLServer
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
            iterations: 3,
            warmup_iterations: 1,
            threads: 1,
            timeout_ms: 10000,
            enable_parallel: false,
        };
        
        let parser_config = ParserConfig::default();
        
        let mut benchmark = SqlBenchmark::new(config, parser_config);
        let sql = "SELECT id, name FROM users WHERE department_id = 10";
        
        let result = benchmark.run_single_query_benchmark("test_simple_query", sql, DatabaseType::MySQL);
        
        assert!(result.successful_runs > 0);
        assert!(result.error.is_none());
    }
    
    #[test]
    fn test_batch_query_benchmark() {
        let config = BenchmarkConfig {
            iterations: 2,
            warmup_iterations: 1,
            threads: 2,
            timeout_ms: 10000,
            enable_parallel: true,
        };
        
        let parser_config = ParserConfig::default();
        
        let mut benchmark = SqlBenchmark::new(config, parser_config);
        
        let queries = vec![
            ("SELECT id, name FROM users", DatabaseType::MySQL),
            ("SELECT id, title FROM products", DatabaseType::PostgreSQL),
        ];
        
        let result = benchmark.run_batch_query_benchmark("test_batch_query", &queries);
        
        assert!(result.successful_runs > 0);
        assert!(result.error.is_none());
    }
    
    #[test]
    fn test_standard_benchmark_sql() {
        let queries = generate_standard_benchmark_sql();
        assert!(!queries.is_empty());
        assert!(queries.len() >= 5);
    }
}