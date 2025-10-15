use std::collections::HashMap;
use std::time::{Duration, Instant};


#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_duration: Duration::from_millis(0),
            average_duration: Duration::from_millis(0),
            min_duration: Duration::from_millis(u64::MAX),
            max_duration: Duration::from_millis(0),
            operations_per_second: 0.0,
        }
    }
}

pub struct PerformanceMonitor {
    metrics: PerformanceMetrics,
    operation_times: Vec<Duration>,
    start_time: Instant,
    database_metrics: HashMap<String, PerformanceMetrics>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn start_operation(&mut self) -> OperationTimer<'_> {
        OperationTimer::new(self)
    }
    
    pub fn record_operation(&mut self, duration: Duration, success: bool, database_type: Option<&str>) {
        // 更新总体指标
        self.metrics.total_operations += 1;
        if success {
            self.metrics.successful_operations += 1;
        } else {
            self.metrics.failed_operations += 1;
        }

        self.metrics.total_duration += duration;
        self.operation_times.push(duration);

        // 更新最小和最大时间
        if duration < self.metrics.min_duration {
            self.metrics.min_duration = duration;
        }
        if duration > self.metrics.max_duration {
            self.metrics.max_duration = duration;
        }

        // 计算平均时间
        if self.metrics.total_operations > 0 {
            self.metrics.average_duration = self.metrics.total_duration / self.metrics.total_operations as u32;
        }

        // 计算 EPS
        let elapsed = self.start_time.elapsed();
        if elapsed.as_secs() > 0 {
            self.metrics.operations_per_second = self.metrics.total_operations as f64 / elapsed.as_secs_f64();
        }

        // 更新数据库特定指标
        if let Some(db_type) = database_type {
            let db_metrics = self.database_metrics.entry(db_type.to_string()).or_default();
            db_metrics.total_operations += 1;
            if success {
                db_metrics.successful_operations += 1;
            } else {
                db_metrics.failed_operations += 1;
            }
            db_metrics.total_duration += duration;
            
            if duration < db_metrics.min_duration {
                db_metrics.min_duration = duration;
            }
            if duration > db_metrics.max_duration {
                db_metrics.max_duration = duration;
            }
            
            if db_metrics.total_operations > 0 {
                db_metrics.average_duration = db_metrics.total_duration / db_metrics.total_operations as u32;
            }
            
            if elapsed.as_secs() > 0 {
                db_metrics.operations_per_second = db_metrics.total_operations as f64 / elapsed.as_secs_f64();
            }
        }
    }
    
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    pub fn get_database_metrics(&self) -> &HashMap<String, PerformanceMetrics> {
        &self.database_metrics
    }
    
    pub fn reset(&mut self) {
        self.metrics = PerformanceMetrics::default();
        self.operation_times.clear();
        self.start_time = Instant::now();
        self.database_metrics.clear();
    }
    
    pub fn get_percentile(&self, percentile: f64) -> Option<Duration> {
        if self.operation_times.is_empty() {
            return None;
        }

        let mut times = self.operation_times.clone();
        times.sort();
        
        let index = (percentile / 100.0 * times.len() as f64) as usize;
        Some(times.get(index).cloned().unwrap_or(Duration::from_millis(0)))
    }

    
    pub fn get_summary(&self) -> String {
        let metrics = self.get_metrics();
        format!(
            "性能统计:\n\
            总操作数: {}\n\
            成功操作数: {}\n\
            失败操作数: {}\n\
            成功率: {:.2}%\n\
            总耗时: {:?}\n\
            平均耗时: {:?}\n\
            最小耗时: {:?}\n\
            最大耗时: {:?}\n\
            EPS: {:.2}\n\
            P95 耗时: {:?}\n\
            P99 耗时: {:?}",
            metrics.total_operations,
            metrics.successful_operations,
            metrics.failed_operations,
            if metrics.total_operations > 0 {
                (metrics.successful_operations as f64 / metrics.total_operations as f64) * 100.0
            } else {
                0.0
            },
            metrics.total_duration,
            metrics.average_duration,
            metrics.min_duration,
            metrics.max_duration,
            metrics.operations_per_second,
            self.get_percentile(95.0).unwrap_or(Duration::from_millis(0)),
            self.get_percentile(99.0).unwrap_or(Duration::from_millis(0))
        )
    }
    

}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self {
            metrics: PerformanceMetrics::default(),
            operation_times: Vec::new(),
            database_metrics: HashMap::new(),
            start_time: Instant::now(),
        }
    }
}

pub struct OperationTimer<'a> {
    monitor: &'a mut PerformanceMonitor,
    start_time: Instant,
    database_type: Option<String>,
    finished: bool,
}

impl<'a> OperationTimer<'a> {
    fn new(monitor: &'a mut PerformanceMonitor) -> Self {
        Self {
            monitor,
            start_time: Instant::now(),
            database_type: None,
            finished: false,
        }
    }

    pub fn with_database_type(mut self, db_type: &str) -> Self {
        self.database_type = Some(db_type.to_string());
        self
    }

    pub fn finish(mut self, success: bool) {
        if !self.finished {
            let duration = self.start_time.elapsed();
            self.monitor.record_operation(duration, success, self.database_type.as_deref());
            self.finished = true;
        }
    }
}

impl<'a> Drop for OperationTimer<'a> {
    fn drop(&mut self) {
        if !self.finished {
            let duration = self.start_time.elapsed();
            self.monitor.record_operation(duration, true, self.database_type.as_deref());
            self.finished = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        // 模拟一些操作
        for i in 0..10 {
            let timer = monitor.start_operation();
            thread::sleep(Duration::from_millis(1));
            timer.finish(i % 2 == 0); // 一半成功，一半失败
        }
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_operations, 10);
        assert_eq!(metrics.successful_operations, 5);
        assert_eq!(metrics.failed_operations, 5);
        assert!(metrics.total_duration > Duration::from_millis(0));
    }

    #[test]
    fn test_database_specific_metrics() {
        let mut monitor = PerformanceMonitor::new();
        
        // MySQL 操作
        let timer = monitor.start_operation().with_database_type("MySQL");
        thread::sleep(Duration::from_millis(1));
        timer.finish(true);
        
        // PostgreSQL 操作
        let timer = monitor.start_operation().with_database_type("PostgreSQL");
        thread::sleep(Duration::from_millis(2));
        timer.finish(true);
        
        let db_metrics = monitor.get_database_metrics();
        assert_eq!(db_metrics.len(), 2);
        assert!(db_metrics.contains_key("MySQL"));
        assert!(db_metrics.contains_key("PostgreSQL"));
    }

    #[test]
    fn test_percentile_calculation() {
        let mut monitor = PerformanceMonitor::new();
        
        // 添加一些已知时间的操作
        for i in 0..100 {
            let timer = monitor.start_operation();
            thread::sleep(Duration::from_millis(i));
            timer.finish(true);
        }
        
        let p95 = monitor.get_percentile(95.0);
        let p99 = monitor.get_percentile(99.0);
        
        assert!(p95.is_some());
        assert!(p99.is_some());
        assert!(p99.unwrap() >= p95.unwrap());
    }
}