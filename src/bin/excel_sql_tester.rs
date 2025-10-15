use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use calamine::{open_workbook, Xlsx, Range, DataType, Reader};
use sqlop::core::types::DatabaseType;
use sqlop::SqlopEngine;

fn main() {
    // 初始化SQL解析引擎
    let mut engine = SqlopEngine::default();
    
    // 统计信息
    let mut per_database_stats = HashMap::new();
    let mut total_success = 0;
    let mut total_all = 0;
    let mut result_output = String::new();
    
    // 获取Excel文件路径
    let excel_files = vec![
        "/Volumes/Macintosh HD/Users/zhushuai/rust/src/sqlop/tests/安恒词法解析（复杂查询） (1).xlsx",
        "/Volumes/Macintosh HD/Users/zhushuai/rust/src/sqlop/tests/DSP解析与策略能力列表 (1).xlsx"
    ];
    
    // 处理每个Excel文件
    for file_path in &excel_files {
        let file_info = format!("\n=== Processing file: {} ===\n", file_path);
        println!("{}", file_info);
        result_output.push_str(&file_info);
        
        let (success, total, file_output) = process_excel_file(file_path, &mut engine, &mut per_database_stats);
        result_output.push_str(&file_output);
        
        total_success += success;
        total_all += total;
        
        let file_result = format!("File results: {} / {} ({}%)\n\n", success, total, 
                               if total > 0 { (success * 100) / total } else { 0 });
        println!("{}", file_result);
        result_output.push_str(&file_result);
    }
    
    // 显示总体统计信息
    let overall_result = format!("=== Overall Results ===\nTotal SQL statements parsed: {}/{} ({}%)\n", 
                             total_success, total_all, 
                             if total_all > 0 { (total_success * 100) / total_all } else { 0 });
    println!("{}", overall_result);
    result_output.push_str(&overall_result);
    
    // 显示按数据库类型的统计信息
    result_output.push_str("\n=== Statistics by Database Type ===\n");
    println!("\n=== Statistics by Database Type ===");
    for (db_type, (success, total)) in &per_database_stats {
        let db_stat = format!("{}: {}/{} ({}%)\n", 
                          db_type, success, total, 
                          if *total > 0 { (success * 100) / total } else { 0 });
        println!("{}", db_stat);
        result_output.push_str(&db_stat);
    }
    
    // 条件运行性能测试
    if total_success >= 100 {
        println!("\n=== Performance Test ===");
        let performance_info = run_performance_test(&mut engine);
        result_output.push_str("\n=== Performance Test ===\n");
        result_output.push_str(&performance_info);
    }
    
    // 写入结果到文件
    match File::create("result.txt") {
        Ok(mut file) => {
            if let Err(e) = file.write_all(result_output.as_bytes()) {
                println!("Error writing to result.txt: {:?}", e);
            } else {
                println!("\nResults successfully written to result.txt");
            }
        },
        Err(e) => {
            println!("Error creating result.txt: {:?}", e);
        }
    }
}

fn process_excel_file(
    file_path: &str,
    engine: &mut SqlopEngine,
    per_database_stats: &mut HashMap<String, (usize, usize)>
) -> (usize, usize, String) {
    let mut success_count = 0;
    let mut total_count = 0;
    let mut output = String::new();
    
    // Try to open Excel file
    match open_workbook::<Xlsx<_>, _>(file_path) {
        Ok(mut workbook) => {
            // Process all sheets
            for sheet_name in workbook.sheet_names().into_iter() {
                let sheet_name = sheet_name.to_string();
                let sheet_info = format!("\nProcessing Sheet: {}", sheet_name);
                println!("{}", sheet_info);
                output.push_str(&sheet_info);
                output.push_str("\n");
                
                match workbook.worksheet_range(&sheet_name) {
                    Some(Ok(range)) => {
                        // 精准读取指定列的SQL语句
                        let sql_columns = if file_path.contains("安恒词法解析") {
                            // 安恒词法解析文件：C列是SQL（索引为2）
                            vec![2]
                        } else if file_path.contains("DSP解析与策略能力列表") {
                            // DSP解析与策略能力列表文件：D列是SQL（索引为3）
                            vec![3]
                        } else {
                            // 其他文件使用自动检测
                            find_all_sql_columns(&range)
                        };
                        
                        if !sql_columns.is_empty() {
                            let columns_info = format!("Found {} SQL columns: {:?}", sql_columns.len(), sql_columns);
                            println!("{}", columns_info);
                            output.push_str(&columns_info);
                            output.push_str("\n");
                            
                            // Process each row and each found SQL column
                            for (row_idx, row) in range.rows().enumerate().skip(1) { // Skip header row
                                for &col_idx in &sql_columns {
                                    if let Some(cell) = row.get(col_idx) {
                                        match cell {
                                            DataType::String(s) => {
                                                let trimmed_sql = s.trim();
                                                // Filter empty rows and comments, but not too strict to ensure capturing all possible SQL
                                                if !trimmed_sql.is_empty() {
                                                    total_count += 1;
                                                    
                                                    // Extended database type attempt range
                                                    let mut database_types = get_database_types_for_sheet(&sheet_name);
                                                    // If specific types fail, try all supported database types
                                                    let all_db_types = vec![
                                                        DatabaseType::MySQL,
                                                        DatabaseType::PostgreSQL,
                                                        DatabaseType::Oracle,
                                                        DatabaseType::SQLServer,
                                                        DatabaseType::Hive,
                                                        DatabaseType::GaussDB,
                                                        DatabaseType::Kingbase,
                                                        DatabaseType::DB2,
                                                        DatabaseType::Dameng,
                                                        DatabaseType::Sybase,
                                                        DatabaseType::Highgo,
                                                        DatabaseType::Greenplum,
                                                    ];
                                                    
                                                    let mut parsed = false;
                                                    let mut parsed_info = String::new();
                                                    
                                                    // First try database types inferred from sheet name
                                                    for db_type in &database_types {
                                                        match engine.parse_sql(trimmed_sql, db_type.clone()) {
                                                            Ok(result) => {
                                                                success_count += 1;
                                                                parsed = true;
                                                                 
                                                                // Extract and store parsed information
                                                                parsed_info = format!(
                                                                    "DB: {:?}, Schema: {:?}, Tables: {:?}, Columns: {:?}",
                                                                    result.databases,
                                                                    result.schemas,
                                                                    result.tables,
                                                                    result.columns
                                                                );
                                                                
                                                                // Add parsed information to output with complete SQL, each on a new line with db_type marked
                                                                output.push_str(&format!("{}\n", trimmed_sql));
                                                                output.push_str(&format!("db_type: {:?}\n", db_type));
                                                                output.push_str(&format!("{}\n", parsed_info));
                                                                                                                              // Update database type statistics
                                                                let db_type_str = format!("{:?}", db_type);
                                                                let entry = per_database_stats.entry(db_type_str.clone()).or_insert((0, 0));
                                                                entry.0 += 1;
                                                                entry.1 += 1;
                                                                
                                                                break;
                                                            },
                                                            Err(_) => {
                                                                // Parse failed, try next
                                                                continue;
                                                            }
                                                        }
                                                    }
                                                    
                                                    // If specific types fail, try all supported database types
                                                    if !parsed {
                                                        for db_type in &all_db_types {
                                                            // Skip already tried types
                                                            if database_types.contains(db_type) {
                                                                continue;
                                                            }
                                                            
                                                            match engine.parse_sql(trimmed_sql, db_type.clone()) {
                                                                Ok(result) => {
                                                                    success_count += 1;
                                                                    parsed = true;
                                                                    
                                                                    // Extract and store parsed information
                                                                    parsed_info = format!(
                                                                        "DB: {:?}, Schema: {:?}, Tables: {:?}, Columns: {:?}",
                                                                        result.databases,
                                                                        result.schemas,
                                                                        result.tables,
                                                                        result.columns
                                                                    );
                                                                    
                                                                    // Add parsed information to output with complete SQL, each on a new line with db_type marked
                                                                    output.push_str(&format!("{}\n", trimmed_sql));
                                                                    output.push_str(&format!("db_type: {:?}\n", db_type));
                                                                    output.push_str(&format!("{}\n", parsed_info));
                                                                    
                                                                    // Update database type statistics
                                                                    let db_type_str = format!("{:?}", db_type);
                                                                    let entry = per_database_stats.entry(db_type_str.clone()).or_insert((0, 0));
                                                                    entry.0 += 1;
                                                                    entry.1 += 1;
                                                                    
                                                                    break;
                                                                },
                                                                Err(_) => {
                                                                    // Parse failed, try next
                                                                    continue;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    
                                                    if !parsed {
                                                        // Update unknown type statistics
                                                        let entry = per_database_stats.entry("Unknown".to_string()).or_insert((0, 0));
                                                        entry.1 += 1;
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        } else {
                            let scan_info = format!("No SQL columns found in Sheet {}, trying full table scan...", sheet_name);
                            println!("{}", scan_info);
                            output.push_str(&scan_info);
                            output.push_str("\n");
                            
                            // Full table scan as last resort
                            for (row_idx, row) in range.rows().enumerate().skip(1) { // Skip header row
                                for (col_idx, cell) in row.iter().enumerate() {
                                    match cell {
                                        DataType::String(s) => {
                                            let trimmed_sql = s.trim();
                                            // Try to identify longer SQL statements
                                            if trimmed_sql.len() > 20 && 
                                               (trimmed_sql.to_lowercase().contains("select") ||
                                                trimmed_sql.to_lowercase().contains("from") ||
                                                trimmed_sql.to_lowercase().contains("where") ||
                                                trimmed_sql.to_lowercase().contains("insert") ||
                                                trimmed_sql.to_lowercase().contains("update") ||
                                                trimmed_sql.to_lowercase().contains("delete")) {
                                                total_count += 1;
                                                
                                                // Try all database types
                                                let all_db_types = vec![
                                                    DatabaseType::MySQL,
                                                    DatabaseType::PostgreSQL,
                                                    DatabaseType::Oracle,
                                                    DatabaseType::SQLServer,
                                                    DatabaseType::Hive,
                                                    DatabaseType::GaussDB,
                                                ];
                                                
                                                let mut parsed = false;
                                                for db_type in &all_db_types {
                                                    if engine.parse_sql(trimmed_sql, db_type.clone()).is_ok() {
                                                        success_count += 1;
                                                        parsed = true;
                                                        
                                                        let db_type_str = format!("{:?}", db_type);
                                                        let entry = per_database_stats.entry(db_type_str).or_insert((0, 0));
                                                        entry.0 += 1;
                                                        entry.1 += 1;
                                                        
                                                        break;
                                                    }
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => {
                        let error_info = format!("Error reading Sheet {}: {:?}", &sheet_name, e);
                        println!("{}", error_info);
                        output.push_str(&error_info);
                        output.push_str("\n");
                    },
                    None => {
                        let error_info = format!("Sheet {} does not exist", &sheet_name);
                        println!("{}", error_info);
                        output.push_str(&error_info);
                        output.push_str("\n");
                    }
                }
            }
        },
        Err(e) => {
            let error_info = format!("Error opening file: {:?}", e);
            println!("{}", error_info);
            output.push_str(&error_info);
            output.push_str("\n");
        }
    }
    
    (success_count, total_count, output)
}

fn find_all_sql_columns(range: &Range<DataType>) -> Vec<usize> {
    let mut sql_columns = Vec::new();
    
    // 1. First find SQL columns by header row
    if let Some(header_row) = range.rows().next() {
        for (col_idx, cell) in header_row.iter().enumerate() {
            match cell {
                DataType::String(header) => {
                    let header_lower = header.to_lowercase();
                    // Extended header keyword matching
                    if header_lower.contains("sql") || header_lower.contains("语句") || 
                       header_lower.contains("查询") || header_lower.contains("select") ||
                       header_lower.contains("表名") || header_lower.contains("语句内容") ||
                       header_lower.contains("示例") || header_lower.contains("demo") {
                        sql_columns.push(col_idx);
                    }
                },
                _ => {}
            }
        }
    }
    
    // 2. If not enough columns found by header, try SQL keyword statistical matching
    if sql_columns.is_empty() || sql_columns.len() < 3 { // Assume Excel might have multiple SQL columns
        // Count SQL keyword matches for each column
        for col_idx in 0..range.width() {
            // Skip already identified columns
            if sql_columns.contains(&col_idx) {
                continue;
            }
            
            let mut sql_keyword_count = 0;
            let mut total_string_cells = 0;
            
            // Count SQL keyword matches in the entire column
            for row in range.rows().skip(1) { // Skip header row
                if let Some(cell) = row.get(col_idx) {
                    match cell {
                        DataType::String(content) => {
                            total_string_cells += 1;
                            let content_lower = content.to_lowercase();
                            
                            // Extended SQL keyword matching
                            if content_lower.contains("select") || 
                               content_lower.contains("insert") ||
                               content_lower.contains("update") ||
                               content_lower.contains("delete") ||
                               content_lower.contains("from") ||
                               content_lower.contains("where") ||
                               content_lower.contains("join") ||
                               content_lower.contains("create") ||
                               content_lower.contains("drop") ||
                               content_lower.contains("alter") ||
                               content_lower.contains("group by") ||
                               content_lower.contains("order by") ||
                               content_lower.contains("having") ||
                               // Check for table name patterns
                               (content_lower.contains(".") && 
                                (content_lower.contains('"') || 
                                 content_lower.contains('[') || 
                                 content_lower.contains('`')) ||
                                content_lower.contains("table ")) ||
                               // Check for SQL comments
                               content_lower.contains("--") ||
                               content_lower.contains("/*") {
                                sql_keyword_count += 1;
                            }
                        },
                        _ => {}
                    }
                }
            }
            
            // Consider it a SQL column if more than 20% of cells contain SQL keywords
            if total_string_cells > 0 && sql_keyword_count >= 1 && 
               (sql_keyword_count as f64 / total_string_cells as f64) >= 0.2 {
                sql_columns.push(col_idx);
            }
        }
    }
    
    // 3. If still no columns found, try non-empty string columns as fallback
    if sql_columns.is_empty() {
        for col_idx in 0..range.width() {
            let mut non_empty_count = 0;
            for row in range.rows().skip(1) { // Skip header row
                if let Some(cell) = row.get(col_idx) {
                    match cell {
                        DataType::String(content) if !content.trim().is_empty() => {
                            non_empty_count += 1;
                        },
                        _ => {}
                    }
                }
            }
            
            // Consider it a potential SQL column if it has many non-empty strings
            if non_empty_count > 5 {
                sql_columns.push(col_idx);
            }
        }
    }
    
    sql_columns
}

fn get_database_types_for_sheet(sheet_name: &str) -> Vec<DatabaseType> {
    let mut result = Vec::new();
    let sheet_name_lower = sheet_name.to_lowercase();
    
    // Support more database type inference
    if sheet_name_lower.contains("mysql") || sheet_name_lower.contains("maria") {
        result.push(DatabaseType::MySQL);
    }
    if sheet_name_lower.contains("postgres") || sheet_name_lower.contains("postgresql") || sheet_name_lower.contains("pg") {
        result.push(DatabaseType::PostgreSQL);
    }
    if sheet_name_lower.contains("oracle") {
        result.push(DatabaseType::Oracle);
    }
    if sheet_name_lower.contains("sqlserver") || sheet_name_lower.contains("mssql") || sheet_name_lower.contains("t-sql") {
        result.push(DatabaseType::SQLServer);
    }
    if sheet_name_lower.contains("hive") {
        result.push(DatabaseType::Hive);
    }
    if sheet_name_lower.contains("gauss") {
        result.push(DatabaseType::GaussDB);
    }
    if sheet_name_lower.contains("kingbase") {
        result.push(DatabaseType::Kingbase);
    }
    if sheet_name_lower.contains("db2") {
        result.push(DatabaseType::DB2);
    }
    if sheet_name_lower.contains("dameng") || sheet_name_lower.contains("dm") {
        result.push(DatabaseType::Dameng);
    }
    
    // If no specific type matched, add MySQL as default
    if result.is_empty() {
        result.push(DatabaseType::MySQL);
        result.push(DatabaseType::PostgreSQL);
        result.push(DatabaseType::Oracle);
    }
    
    result
}

fn run_performance_test(engine: &mut SqlopEngine) -> String {
    let mut output = String::new();
    
    // Use simple SQL statements without quotes
    let test_sqls = [
        "SELECT id FROM users",
        "SELECT * FROM products",
        "SELECT count(*) FROM orders",
        "SELECT 1 + 1 AS result"
    ];
    
    let db_type = DatabaseType::MySQL;
    let iterations = 100000;
    
    println!("Preheating parser...");
    output.push_str("Preheating parser...\n");
    
    // Preheat
    for _ in 0..1000 {
        for sql in &test_sqls {
            let _ = engine.parse_sql(sql, db_type.clone());
        }
    }
    
    println!("Starting performance test ({} iterations)...", iterations);
    output.push_str(&format!("Starting performance test ({} iterations)...\n", iterations));
    
    let start_time = Instant::now();
    let mut count = 0;
    
    for i in 0..iterations {
        let sql = &test_sqls[i % test_sqls.len()];
        if engine.parse_sql(sql, db_type.clone()).is_ok() {
            count += 1;
        }
        
        if (i + 1) % 20000 == 0 {
            let iteration_msg = format!("Completed {} iterations\n", i + 1);
            println!("{}", iteration_msg.trim());
            output.push_str(&iteration_msg);
        }
    }
    
    let elapsed = start_time.elapsed();
    let eps = count as f64 / elapsed.as_secs_f64();
    
    output.push_str("\nPerformance test results:\n");
    output.push_str(&format!("- Total parsed: {}\n", count));
    output.push_str(&format!("- Total time: {:.2} seconds\n", elapsed.as_secs_f64()));
    output.push_str(&format!("- EPS: {:.2}\n", eps));
    
    println!("\nPerformance test results:");
    println!("- Total parsed: {}", count);
    println!("- Total time: {:.2} seconds", elapsed.as_secs_f64());
    println!("- EPS: {:.2}", eps);
    
    // Compare with target
    let target_eps = 6000.0;
    let result_msg = if eps >= target_eps {
        format!("Performance meets requirements: EPS({:.2}) >= target({})\n", eps, target_eps)
    } else {
        format!("Performance does not meet requirements: EPS({:.2}) < target({})\n", eps, target_eps)
    };
    
    println!("{}", result_msg.trim());
    output.push_str(&result_msg);
    
    output
}