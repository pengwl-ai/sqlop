// Piped SQL支持模块
// 实现Piped SQL到标准SQL的转换功能

use std::collections::{HashMap, HashSet};
use crate::core::types::{DatabaseType, SqlObject};
use regex::Regex;

/// Piped SQL解析器，用于解析和转换Piped SQL语法
#[derive(Clone)]
pub struct PipedSqlParser {
    database_type: DatabaseType,
    
    // 用于识别Piped SQL模式的正则表达式
    pipe_pattern: Regex,
    temp_table_pattern: Regex,
    
    // 保存中间结果
    temp_tables: HashMap<String, String>,
}

impl PipedSqlParser {
    pub fn new(database_type: DatabaseType) -> Result<Self, regex::Error> {
        Ok(Self {
            database_type,
            pipe_pattern: Regex::new(r#"\|\s*(let|into|from|select|where|group|order|limit)"#)?,
            temp_table_pattern: Regex::new(r#"let\s+([a-zA-Z0-9_]+)\s*=\s*"#)?,
            temp_tables: HashMap::new(),
        })
    }
    
    /// 将Piped SQL转换为标准SQL
    pub fn convert_to_standard_sql(&mut self, piped_sql: &str) -> Result<String, String> {
        // 重置临时表存储
        self.temp_tables.clear();
        
        // 检查是否为Piped SQL
        if !self.is_piped_sql(piped_sql) {
            return Ok(piped_sql.to_string());
        }
        
        // 分割Piped SQL语句
        let statements = self.split_piped_statements(piped_sql);
        
        // 处理每个语句段
        let mut standard_sql = String::new();
        
        // 检查是否包含临时表定义
        let has_temp_table = statements.iter().any(|s| {
            let trimmed = s.trim();
            trimmed.starts_with("let ") && trimmed.contains("=")
        });
        
        // 特殊处理简单的管道SQL（没有临时表的情况）
        if !has_temp_table && !statements.is_empty() {
            let first_segment = statements[0].trim();
            if first_segment.starts_with("select ") {
                // 构建完整的SQL
                let mut complete_sql = first_segment.replacen("select ", "SELECT ", 1);
                complete_sql = complete_sql.replacen("from ", "FROM ", 1);
                complete_sql = complete_sql.replacen("where ", "WHERE ", 1);
                
                for segment in &statements[1..] {
                    let trimmed = segment.trim();
                    
                    if trimmed.starts_with("where ") {
                        let condition = &trimmed[6..].trim(); // 去掉 "where "
                        if complete_sql.to_uppercase().contains("WHERE") {
                            complete_sql.push_str(" AND ");
                            complete_sql.push_str(condition);
                        } else {
                            complete_sql.push_str(" WHERE ");
                            complete_sql.push_str(condition);
                        }
                    } else if trimmed.starts_with("group ") {
                        complete_sql.push_str(" GROUP BY ");
                        complete_sql.push_str(&trimmed[6..].trim());
                    } else if trimmed.starts_with("order ") {
                        complete_sql.push_str(" ORDER BY ");
                        complete_sql.push_str(&trimmed[6..].trim());
                    } else if trimmed.starts_with("limit ") {
                        complete_sql.push_str(" LIMIT ");
                        complete_sql.push_str(&trimmed[6..].trim());
                    } else {
                        // 对于其他类型的段，使用原有的处理逻辑
                        let processed = self.process_pipe_segment(segment, false)?;
                        if !processed.is_empty() {
                            complete_sql.push_str("; ");
                            complete_sql.push_str(&processed);
                        }
                    }
                }
                
                return Ok(complete_sql);
            }
        }
        
        // 原有逻辑（处理带临时表的情况）
        for (i, statement) in statements.iter().enumerate() {
            let processed = self.process_pipe_segment(statement, i == statements.len() - 1)?;
            
            if !processed.is_empty() {
                if !standard_sql.is_empty() {
                    standard_sql.push_str(";\n");
                }
                standard_sql.push_str(&processed);
            }
        }
        
        Ok(standard_sql)
    }
    
    /// 检查是否为Piped SQL
    fn is_piped_sql(&self, sql: &str) -> bool {
        self.pipe_pattern.is_match(sql)
    }
    
    /// 分割Piped SQL语句
    fn split_piped_statements(&self, piped_sql: &str) -> Vec<String> {
        let mut statements = Vec::new();
        let mut current = String::new();
        let mut in_quotes = None;
        let mut last_char = '\0';
        
        for c in piped_sql.chars() {
            // 处理引号
            if c == '"' || c == '\'' {
                if last_char != '\\' {
                    in_quotes = if in_quotes == Some(c) { None } else { Some(c) };
                }
            }
            
            // 处理管道符
            if c == '|' && in_quotes.is_none() && last_char != '\\' {
                if !current.trim().is_empty() {
                    statements.push(current.trim().to_string());
                }
                current.clear();
            } else {
                current.push(c);
            }
            
            last_char = c;
        }
        
        // 添加最后一个语句
        if !current.trim().is_empty() {
            statements.push(current.trim().to_string());
        }
        
        statements
    }
    
    /// 处理单个管道段
    fn process_pipe_segment(&mut self, segment: &str, is_last: bool) -> Result<String, String> {
        let trimmed = segment.trim();
        
        // 处理let语句（创建临时表）
        if let Some(captures) = self.temp_table_pattern.captures(trimmed) {
            if let Some(table_name) = captures.get(1) {
                let table_name = table_name.as_str();
                let sql_content = trimmed[captures.get(0).unwrap().as_str().len()..].trim();
                
                // 存储临时表定义
                self.temp_tables.insert(table_name.to_string(), sql_content.to_string());
                
                // 不是最后一个段时，返回创建临时表的SQL
                if !is_last {
                    let create_temp_sql = self.generate_temp_table_sql(table_name, sql_content);
                    return Ok(create_temp_sql);
                }
            }
            return Ok(String::new());
        }
        
        // 处理其他类型的管道段
        if trimmed.starts_with("from ") || trimmed.starts_with("select ") || 
           trimmed.starts_with("where ") || trimmed.starts_with("group ") || 
           trimmed.starts_with("order ") || trimmed.starts_with("limit ") {
            
            // 构建完整的SQL查询
            return self.build_complete_sql(trimmed);
        }
        
        // 对于不支持的语法，原样返回
        Ok(trimmed.to_string())
    }
    
    /// 生成创建临时表的SQL
    fn generate_temp_table_sql(&self, table_name: &str, sql_content: &str) -> String {
        match self.database_type {
            DatabaseType::MySQL => format!("CREATE TEMPORARY TABLE {} AS {}", table_name, sql_content),
            DatabaseType::PostgreSQL => format!("CREATE TEMP TABLE {} AS {}", table_name, sql_content),
            DatabaseType::SQLServer => format!("SELECT * INTO #{} FROM ({}) AS temp", table_name, sql_content),
            _ => format!("CREATE TEMP TABLE {} AS {}", table_name, sql_content),
        }
    }
    
    /// 构建完整的SQL查询
    fn build_complete_sql(&self, segment: &str) -> Result<String, String> {
        // 查找最后定义的临时表作为主表
        if let Some((last_table, _)) = self.temp_tables.iter().last() {
            let base_sql = format!("SELECT * FROM {}", last_table);
            
            // 根据当前段类型，将其附加到基础SQL
            if segment.starts_with("from ") {
                // 不常见的用法，但仍需支持
                return Ok(segment.to_string());
            } else if segment.starts_with("select ") {
                // 检查段内容是否已经包含FROM子句
                let segment_uppercase = segment.to_uppercase();
                if segment_uppercase.contains(" FROM ") {
                    // 如果已经包含FROM子句，将SQL关键字转换为大写
                    let mut result = String::new();
                    let parts: Vec<&str> = segment.split_whitespace().collect();
                    
                    for (i, part) in parts.iter().enumerate() {
                        let part_uppercase = part.to_uppercase();
                        // 检查是否为SQL关键字
                        if part_uppercase == "SELECT" || part_uppercase == "FROM" || 
                           part_uppercase == "WHERE" || part_uppercase == "AND" || 
                           part_uppercase == "OR" || part_uppercase == "LIMIT" {
                            result.push_str(&part_uppercase);
                        } else {
                            result.push_str(part);
                        }
                        
                        // 添加空格，除非是最后一个部分
                        if i < parts.len() - 1 {
                            result.push(' ');
                        }
                    }
                    
                    return Ok(result);
                } else {
                    // 如果不包含FROM子句，就添加FROM子句，只将SQL关键字转换为大写
                    let select_part = segment.trim_start_matches("select ");
                    return Ok(format!("SELECT {} FROM {}", select_part, last_table));
                }
            } else if segment.starts_with("where ") {
                return Ok(format!("{} {}", base_sql, segment));
            } else if segment.starts_with("group ") {
                return Ok(format!("{} {}", base_sql, segment));
            } else if segment.starts_with("order ") {
                return Ok(format!("{} {}", base_sql, segment));
            } else if segment.starts_with("limit ") {
                return Ok(format!("{} {}", base_sql, segment));
            }
        }
        
        // 特殊处理第一个select语句
        if segment.starts_with("select ") && self.temp_tables.is_empty() {
            // 直接返回段内容，确保包含FROM子句
            return Ok(segment.to_string());
        }
        
        // 如果没有临时表，直接返回段内容
        Ok(segment.to_string())
    }
    
    /// 提取Piped SQL中的对象信息
    pub fn extract_objects(&self, piped_sql: &str) -> Result<Vec<SqlObject>, String> {
        // 这里简化处理，实际实现应更复杂
        // 将Piped SQL转换为标准SQL后再提取对象
        let mut parser = self.clone();
        let standard_sql = parser.convert_to_standard_sql(piped_sql)?;
        
        // 创建一个简单的对象提取器
        let mut objects = Vec::new();
        
        // 提取表名（简化版）
        let table_pattern = Regex::new(r#"FROM\s+([a-zA-Z0-9_.]+)"#).map_err(|e| e.to_string())?;
        let mut table_names = HashSet::new();
        
        for capture in table_pattern.captures_iter(&standard_sql.to_uppercase()) {
            if let Some(table_name) = capture.get(1) {
                table_names.insert(table_name.as_str().to_string());
            }
        }
        
        // 转换为SqlObject
        for table_name in table_names {
            objects.push(SqlObject {
                database: None,
                schema: None,
                table: table_name,
                column: None,
                alias: None,
            });
        }
        
        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_convert_simple_piped_sql() {
        let mut parser = PipedSqlParser::new(DatabaseType::MySQL).unwrap();
        let piped_sql = "select * from users where age > 18 | where active = true | limit 10";
        let result = parser.convert_to_standard_sql(piped_sql).unwrap();
        
        // 添加调试输出
        println!("转换结果: {}", result);
        
        assert!(result.contains("SELECT * FROM users"));
        assert!(result.contains("WHERE age > 18"));
        assert!(result.contains("AND active = true"));
        assert!(result.contains("LIMIT 10"));
    }
    
    #[test]
    fn test_convert_with_temp_table() {
        let mut parser = PipedSqlParser::new(DatabaseType::PostgreSQL).unwrap();
        let piped_sql = "let temp_users = select * from users where age > 18 | select name, email from temp_users where active = true";
        let result = parser.convert_to_standard_sql(piped_sql).unwrap();
        
        // 添加调试输出
        println!("转换结果: {}", result);
        
        assert!(result.contains("CREATE TEMP TABLE temp_users"));
        assert!(result.contains("SELECT name, email FROM temp_users"));
    }
}