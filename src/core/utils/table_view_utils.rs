// 表和视图相关的工具函数

/// 过滤掉子查询表名
pub fn filter_tables(tables: &[String]) -> Vec<String> {
    tables.iter()
        .filter(|&table| {
            // 过滤掉典型的子查询表名
            !table.starts_with("subquery") && 
            !table.contains("_") && 
            !table.chars().all(char::is_numeric) &&
            table != "t"
        })
        .cloned()
        .collect()
}

/// 从SQL语句中直接提取CREATE VIEW中的表和列（作为补充）
pub fn extract_view_info(sql: &str) -> (Vec<String>, Vec<String>) {
    // 简化实现，未来可以扩展为更复杂的解析
    (Vec::new(), Vec::new())
}