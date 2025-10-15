use sqlop::{SqlopEngine, DatabaseType};
use calamine::{open_workbook, Reader, Xlsx};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

fn main() {
    println!("=== SQL解析测试工具 ===\n");
    
    // 创建SQL解析引擎
    sqlop::init_logger();
    let mut engine = SqlopEngine::default();
    
    // 定义验收条件中的Excel文件路径
    let excel_files = [
        "/Volumes/Macintosh HD/Users/zhushuai/rust/src/sqlop/tests/安恒词法解析（复杂查询） (1).xlsx",
        "/Volumes/Macintosh HD/Users/zhushuai/rust/src/sqlop/tests/DSP解析与策略能力列表 (1).xlsx",
    ];
    
    let mut overall_success = 0;
    let mut overall_total = 0;
    let mut per_database_stats: HashMap<String, (usize, usize)> = HashMap::new();
    let mut per_file_stats: Vec<(String, usize, usize)> = Vec::new();
    
    // 处理每个Excel文件
    for file_path in &excel_files {
        println!("\n===========================================");
        println!("处理文件: {}", file_path);
        println!("===========================================");
        
        let (file_success, file_total) = process_excel_file(file_path, &mut engine, &mut per_database_stats);
        
        per_file_stats.push((file_path.to_string(), file_success, file_total));
        overall_success += file_success;
        overall_total += file_total;
        
        println!("\n文件统计: 成功 {}/{}, 失败 {}/{} (成功率: {:.1}%)", 
                 file_success, file_total, 
                 file_total - file_success, file_total,
                 (file_success as f64 / file_total as f64) * 100.0);
    }
    
    // 打印总体统计
    println!("\n===========================================");
    println!("=== 总体测试结果 ===");
    println!("总SQL语句数: {}", overall_total);
    println!("成功解析数: {}", overall_success);
    println!("失败解析数: {}", overall_total - overall_success);
    println!("总体成功率: {:.1}%", 
             (overall_success as f64 / overall_total as f64) * 100.0);
    
    // 按文件统计
    println!("\n=== 按文件统计 ===");
    for (file_path, success, total) in &per_file_stats {
        println!("{}: 成功 {}/{}, 失败 {}/{} (成功率: {:.1}%)", 
                 file_path,
                 success, total,
                 total - success, total,
                 (success as f64 / *total as f64) * 100.0);
    }
    
    // 按数据库类型统计
    println!("\n=== 按数据库类型统计 ===");
    let mut sorted_types: Vec<_> = per_database_stats.iter().collect();
    sorted_types.sort_by_key(|&(_, (_, total))| std::cmp::Reverse(*total));
    
    for (db_type, &(success, total)) in sorted_types {
        println!("{}: 成功 {}/{}, 失败 {}/{} (成功率: {:.1}%)", 
                 db_type,
                 success, total,
                 total - success, total,
                 (success as f64 / total as f64) * 100.0);
    }
    
    // 如果所有SQL都成功解析，运行性能测试
    if overall_success == overall_total && overall_total > 0 {
        println!("\n===========================================");
        println!("所有SQL语句解析成功！开始性能测试...");
        run_performance_test(&mut engine);
    } else if overall_total == 0 {
        println!("\n⚠️  未找到可测试的SQL语句");
    } else {
        println!("\n❌ 部分SQL语句解析失败，跳过性能测试");
    }
}

fn find_all_sql_columns(range: &calamine::Range<calamine::DataType>) -> Vec<usize> {
    let mut sql_columns = Vec::new();
    let mut processed_columns = HashSet::new();
    
    // 先尝试找到主要的SQL列
    if let Some(main_col) = find_sql_column(range) {
        sql_columns.push(main_col);
        processed_columns.insert(main_col);
    }
    
    // 然后尝试找到其他可能包含SQL的列
    for col_idx in 0..range.width() {
        if processed_columns.contains(&col_idx) {
            continue;
        }
        
        let mut sql_keyword_count = 0;
        let mut total_string_cells = 0;
        
        for row in range.rows() {
            if let Some(cell) = row.get(col_idx) {
                match cell {
                    calamine::DataType::String(content) => {
                        total_string_cells += 1;
                        let content_lower = content.to_lowercase();
                        
                        // 检查是否可能包含SQL语句
                        if content_lower.contains("select") || 
                           content_lower.contains("from") ||
                           content_lower.contains("where") ||
                           content_lower.contains("join") ||
                           content_lower.contains("insert") ||
                           content_lower.contains("update") ||
                           content_lower.contains("delete") ||
                           content_lower.contains("create") ||
                           content_lower.contains("table") {
                            sql_keyword_count += 1;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // 如果该列中有超过20%的单元格包含SQL关键字，就认为这也是SQL列
        if total_string_cells > 5 && sql_keyword_count >= 3 && 
           (sql_keyword_count as f64 / total_string_cells as f64) >= 0.2 {
            println!("在列 {} 中检测到额外的SQL列: {} 个SQL关键字匹配 / {} 个字符串单元格", 
                     col_idx, sql_keyword_count, total_string_cells);
            sql_columns.push(col_idx);
            processed_columns.insert(col_idx);
        }
    }
    
    // 如果没有找到任何SQL列，尝试检查所有非空字符串列
    if sql_columns.is_empty() {
        for col_idx in 0..range.width() {
            let mut non_empty_count = 0;
            let mut has_sql_keyword = false;
            
            for row in range.rows().skip(1) { // 跳过标题行
                if let Some(cell) = row.get(col_idx) {
                    match cell {
                        calamine::DataType::String(content) => {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                non_empty_count += 1;
                                let content_lower = trimmed.to_lowercase();
                                if content_lower.contains("select") || 
                                   content_lower.contains("from") ||
                                   content_lower.contains("where") {
                                    has_sql_keyword = true;
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
            
            // 如果该列有超过5个非空字符串并且至少有一个包含SQL关键字
            if non_empty_count > 5 && has_sql_keyword {
                println!("在列 {} 中检测到可能的SQL列: {} 个非空字符串，包含SQL关键字", 
                         col_idx, non_empty_count);
                sql_columns.push(col_idx);
                processed_columns.insert(col_idx);
            }
        }
    }
    
    sql_columns
}

fn process_excel_file(
    file_path: &str,
    engine: &mut SqlopEngine,
    per_database_stats: &mut HashMap<String, (usize, usize)>
) -> (usize, usize) {
    let mut success_count = 0;
    let mut total_count = 0;
    
    // 尝试打开Excel文件
    match open_workbook::<Xlsx<_>, _>(file_path) {
        Ok(mut workbook) => {
            // 遍历所有sheet
            for sheet_name in workbook.sheet_names() {
                println!("\n处理Sheet: {}", sheet_name);
                
                match workbook.worksheet_range(&sheet_name) {
                    Some(Ok(range)) => {
                        // 尝试找到所有可能的SQL列
                        let sql_columns = find_all_sql_columns(&range);
                        
                        if !sql_columns.is_empty() {
                            println!("找到SQL列在位置: {:?}", sql_columns);
                            
                            // 处理每一列
                            for col_idx in &sql_columns {
                                println!("\n处理列 {}:", col_idx);
                                
                                // 处理每一行
                                for (row_idx, row) in range.rows().enumerate().skip(1) { // 跳过标题行
                                    if let Some(cell) = row.get(*col_idx) {
                                        match cell {
                                            calamine::DataType::String(s) => {
                                                let trimmed_sql = s.trim();
                                                if !trimmed_sql.is_empty() && !trimmed_sql.starts_with('#') {
                                                    total_count += 1;
                                                    
                                                    // 尝试使用不同的数据库类型进行解析
                                                    let database_types = get_database_types_for_sheet(&sheet_name);
                                                    let mut parsed = false;
                                                    
                                                    for db_type in &database_types {
                                                        match engine.parse_sql(trimmed_sql, db_type.clone()) {
                                                            Ok(result) => {
                                                                success_count += 1;
                                                                parsed = true;
                                                                
                                                                // 更新数据库类型统计
                                                                let db_type_str = format!("{:?}", db_type);
                                                                let entry = per_database_stats.entry(db_type_str.clone()).or_insert((0, 0));
                                                                entry.0 += 1;
                                                                entry.1 += 1;
                                                                
                                                                println!("Row {} (列 {}): ✅ 成功 [{}] - 表: {:?}, 列: {:?}",
                                                                         row_idx, col_idx,
                                                                         db_type_str,
                                                                         result.tables,
                                                                         result.columns);
                                                                break;
                                                            },
                                                            Err(e) => {
                                                                // 解析失败，尝试下一个数据库类型
                                                                continue;
                                                            }
                                                        }
                                                    }
                                                    
                                                    if !parsed {
                                                        // 尝试所有数据库类型
                                                        let all_db_types = [
                                                            DatabaseType::MySQL,
                                                            DatabaseType::Oracle,
                                                            DatabaseType::PostgreSQL,
                                                            DatabaseType::SQLServer,
                                                            DatabaseType::Hive,
                                                            DatabaseType::GaussDB,
                                                            DatabaseType::Kingbase,
                                                            DatabaseType::Highgo,
                                                            DatabaseType::Greenplum,
                                                            DatabaseType::Dameng,
                                                            DatabaseType::DB2,
                                                            DatabaseType::Sybase,
                                                        ];
                                                        
                                                        for db_type in &all_db_types {
                                                            match engine.parse_sql(trimmed_sql, db_type.clone()) {
                                                                Ok(result) => {
                                                                    success_count += 1;
                                                                    parsed = true;
                                                                    
                                                                    let db_type_str = format!("{:?}", db_type);
                                                                    let entry = per_database_stats.entry(db_type_str.clone()).or_insert((0, 0));
                                                                    entry.0 += 1;
                                                                    entry.1 += 1;
                                                                    
                                                                    println!("Row {} (列 {}): ✅ 成功 (备用类型: [{}])",
                                                                             row_idx, col_idx,
                                                                             db_type_str);
                                                                    break;
                                                                },
                                                                Err(_) => {
                                                                    continue;
                                                                }
                                                            }
                                                        }
                                                        
                                                        if !parsed {
                                                            // 只在详细模式下打印失败信息
                                                            if total_count % 50 == 0 {
                                                                println!("Row {} (列 {}): ❌ 失败 - SQL 开头: {}", 
                                                                         row_idx, col_idx, 
                                                                         trimmed_sql.split('\n').next().unwrap_or("").chars().take(50).collect::<String>());
                                                            }
                                                            
                                                            // 更新未知类型统计
                                                            let entry = per_database_stats.entry("Unknown".to_string()).or_insert((0, 0));
                                                            entry.1 += 1;
                                                        }
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        } else {
                            // 即使没有找到SQL列，也尝试扫描整个表格
                            println!("⚠️ 在Sheet {} 中未明确找到SQL列，尝试扫描整个表格...", sheet_name);
                            
                            for col_idx in 0..range.width().min(10) { // 限制最多检查前10列
                                println!("尝试扫描列 {}", col_idx);
                                
                                for (row_idx, row) in range.rows().enumerate().skip(1) {
                                    if let Some(cell) = row.get(col_idx) {
                                        match cell {
                                            calamine::DataType::String(s) => {
                                                let trimmed_sql = s.trim();
                                                if !trimmed_sql.is_empty() && trimmed_sql.len() > 20 {
                                                    // 尝试解析看起来像SQL的文本
                                                    let all_db_types = [DatabaseType::MySQL, DatabaseType::Hive];
                                                    let mut tried = false;
                                                    
                                                    for db_type in &all_db_types {
                                                        if engine.parse_sql(trimmed_sql, db_type.clone()).is_ok() {
                                                            success_count += 1;
                                                            total_count += 1;
                                                            tried = true;
                                                            
                                                            let db_type_str = format!("{:?}", db_type);
                                                            let entry = per_database_stats.entry(db_type_str.clone()).or_insert((0, 0));
                                                            entry.0 += 1;
                                                            entry.1 += 1;
                                                            
                                                            println!("在列 {} 行 {} 发现SQL: ✅ 成功 [{}]", col_idx, row_idx, db_type_str);
                                                            break;
                                                        }
                                                    }
                                                    
                                                    if tried && total_count % 100 == 0 {
                                                        println!("已扫描 {} 条SQL语句", total_count);
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => {
                        println!("❌ 读取Sheet {} 失败: {:?}", sheet_name, e);
                    },
                    None => {
                        println!("❌ Sheet {} 不存在", sheet_name);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ 打开文件失败: {:?}", e);
        }
    }
    
    (success_count, total_count)
}

fn find_sql_column(range: &calamine::Range<calamine::DataType>) -> Option<usize> {
    // 检查标题行
    if let Some(header_row) = range.rows().next() {
        for (col_idx, cell) in header_row.iter().enumerate() {
            match cell {
                calamine::DataType::String(header) => {
                    let header_lower = header.to_lowercase();
                    if header_lower.contains("sql") || header_lower.contains("语句") || 
                       header_lower.contains("查询") || header_lower.contains("select") ||
                       header_lower.contains("表名") || header_lower.contains("语句内容") ||
                       header_lower.contains("示例") || header_lower.contains("demo") {
                        return Some(col_idx);
                    }
                },
                _ => {}
            }
        }
    }
    
    // 尝试更广泛的SQL关键字匹配 - 检查所有列的所有行
    for col_idx in 0..range.width() {
        // 统计包含SQL关键字的单元格数量
        let mut sql_keyword_count = 0;
        let mut total_string_cells = 0;
        
        for row in range.rows() {
            if let Some(cell) = row.get(col_idx) {
                match cell {
                    calamine::DataType::String(content) => {
                        total_string_cells += 1;
                        let content_lower = content.to_lowercase();
                        // 更宽松的SQL关键字匹配
                        if content_lower.contains("select") || 
                           content_lower.contains("from") ||
                           content_lower.contains("where") ||
                           content_lower.contains("join") ||
                           content_lower.contains("insert") ||
                           content_lower.contains("update") ||
                           content_lower.contains("delete") ||
                           content_lower.contains("create") ||
                           content_lower.contains("drop") ||
                           content_lower.contains("alter") ||
                           content_lower.contains("group by") ||
                           content_lower.contains("order by") ||
                           content_lower.contains("having") ||
                           // 检查是否包含表名模式
                           (content_lower.contains(".") && 
                            (content_lower.contains("\") || 
                             content_lower.contains("[") || 
                             content_lower.contains("`")) ||
                            content_lower.contains("table ")) ||
                           // 检查是否包含SQL注释
                           content_lower.contains("--") ||
                           content_lower.contains("/*") {
                            sql_keyword_count += 1;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // 如果该列中有超过30%的单元格包含SQL关键字，就认为这是SQL列
        if total_string_cells > 0 && sql_keyword_count >= 2 && 
           (sql_keyword_count as f64 / total_string_cells as f64) >= 0.3 {
            println!("在列 {} 中检测到可能的SQL列: {} 个SQL关键字匹配 / {} 个字符串单元格", 
                     col_idx, sql_keyword_count, total_string_cells);
            return Some(col_idx);
        }
    }
    
    // 最后的尝试：检查是否有任何非空字符串列
    for col_idx in 0..range.width() {
        let mut non_empty_count = 0;
        for row in range.rows().skip(1) { // 跳过标题行
            if let Some(cell) = row.get(col_idx) {
                match cell {
                    calamine::DataType::String(content) if !content.trim().is_empty() => {
                        non_empty_count += 1;
                    },
                    _ => {}
                }
            }
        }
        
        // 如果该列有超过10个非空字符串，可能是SQL列
        if non_empty_count > 10 {
            println!("在列 {} 中检测到可能的SQL列: {} 个非空字符串", col_idx, non_empty_count);
            return Some(col_idx);
        }
    }
    
    None
}

fn get_database_types_for_sheet(sheet_name: &str) -> Vec<DatabaseType> {
    let sheet_lower = sheet_name.to_lowercase();
    
    // 根据sheet名称推断可能的数据库类型
    if sheet_lower.contains("mysql") {
        return vec![DatabaseType::MySQL];
    } else if sheet_lower.contains("oracle") {
        return vec![DatabaseType::Oracle];
    } else if sheet_lower.contains("postgre") || sheet_lower.contains("pg") {
        return vec![DatabaseType::PostgreSQL];
    } else if sheet_lower.contains("sqlserver") {
        return vec![DatabaseType::SQLServer];
    } else if sheet_lower.contains("hive") {
        return vec![DatabaseType::Hive];
    } else if sheet_lower.contains("gauss") {
        return vec![DatabaseType::GaussDB];
    } else if sheet_lower.contains("kingbase") {
        return vec![DatabaseType::Kingbase];
    } else if sheet_lower.contains("highgo") {
        return vec![DatabaseType::Highgo];
    } else if sheet_lower.contains("greenplum") || sheet_lower.contains("gp") {
        return vec![DatabaseType::Greenplum];
    } else if sheet_lower.contains("dameng") || sheet_lower.contains("dm") {
        return vec![DatabaseType::Dameng];
    } else if sheet_lower.contains("db2") {
        return vec![DatabaseType::DB2];
    } else if sheet_lower.contains("sybase") {
        return vec![DatabaseType::Sybase];
    }
    
    // 默认尝试所有常见数据库类型
    vec![
        DatabaseType::MySQL,
        DatabaseType::PostgreSQL,
        DatabaseType::Oracle,
        DatabaseType::SQLServer,
        DatabaseType::Hive,
        DatabaseType::GaussDB,
    ]
}

// 保留空行，避免编译器警告

fn run_performance_test(engine: &mut SqlopEngine) {
    // 基本性能测试 - 使用常见的SQL语句
    let test_sqls = [
        "SELECT id, name, email FROM users WHERE status = 'active'",
        "INSERT INTO products (name, price) VALUES ('Laptop', 999.99)",
        "UPDATE orders SET status = 'completed' WHERE id = 10",
        "DELETE FROM logs WHERE timestamp < '2023-01-01'",
    ];
    
    let db_type = DatabaseType::MySQL;
    let iterations = 100000;
    
    println!("预热解析器...");
    // 预热
    for _ in 0..1000 {
        for sql in &test_sqls {
            let _ = engine.parse_sql(sql, db_type.clone());
        }
    }
    
    println!("开始性能测试 ({}次迭代)...", iterations);
    let start_time = Instant::now();
    let mut count = 0;
    
    for i in 0..iterations {
        let sql = &test_sqls[i % test_sqls.len()];
        if engine.parse_sql(sql, db_type.clone()).is_ok() {
            count += 1;
        }
        
        if (i + 1) % 20000 == 0 {
            println!("已完成 {} 次迭代", i + 1);
        }
    }
    
    let elapsed = start_time.elapsed();
    let eps = count as f64 / elapsed.as_secs_f64();
    
    println!("\n性能测试结果:");
    println!("- 总解析次数: {}", count);
    println!("- 总耗时: {:.2} 秒", elapsed.as_secs_f64());
    println!("- 每秒解析数(EPS): {:.2}", eps);
    
    // 与目标要求比较
    let target_eps = 6000.0;
    if eps >= target_eps {
        println!("✅ 性能满足要求！当前EPS({:.2}) >= 目标EPS({})", eps, target_eps);
    } else {
        println!("❌ 性能未满足要求。当前EPS({:.2}) < 目标EPS({})", eps, target_eps);
    }
}

// 所有必要的导入已在文件顶部声明