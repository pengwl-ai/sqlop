pub mod performance;
pub mod validation;
pub mod format;

pub use performance::PerformanceMonitor;
pub use validation::SqlValidator;
pub use validation::SqlSecurityChecker;
pub use format::SqlFormatter;

use crate::core::error::Result;
use crate::core::types::DatabaseType;

/// 获取数据库类型的默认配置
pub fn get_default_config(db_type: &DatabaseType) -> crate::core::types::DatabaseConfig {
    use crate::core::types::DatabaseConfig;
    
    match db_type {
        DatabaseType::MySQL => DatabaseConfig {
            quote_char: '`',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: None,
            custom_keywords: vec!["AUTO_INCREMENT".to_string()],
        },
        DatabaseType::PostgreSQL => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::SQLServer => DatabaseConfig {
            quote_char: '[',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("dbo".to_string()),
            custom_keywords: vec!["IDENTITY".to_string()],
        },
        DatabaseType::Oracle => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: true,
            support_schema: false,
            default_schema: None,
            custom_keywords: vec!["SEQUENCE".to_string()],
        },
        DatabaseType::Hive => DatabaseConfig {
            quote_char: '`',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("default".to_string()),
            custom_keywords: vec!["EXTERNAL".to_string()],
        },
        DatabaseType::GaussDB => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::Kingbase => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::Highgo => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::Greenplum => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::Vastbase => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("public".to_string()),
            custom_keywords: vec!["SERIAL".to_string()],
        },
        DatabaseType::Sybase => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: Some("dbo".to_string()),
            custom_keywords: vec!["IDENTITY".to_string()],
        },
        DatabaseType::DB2 => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: true,
            support_schema: true,
            default_schema: None,
            custom_keywords: vec!["GENERATED".to_string()],
        },
        DatabaseType::Dameng => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: true,
            default_schema: None,
            custom_keywords: vec!["IDENTITY".to_string()],
        },
        DatabaseType::SQLite => DatabaseConfig {
            quote_char: '"',
            identifier_case_sensitive: false,
            support_schema: false,
            default_schema: None,
            custom_keywords: vec!["AUTOINCREMENT".to_string()],
        },
    }
}

/// 检查 SQL 语句是否包含敏感信息
pub fn contains_sensitive_info(sql: &str, patterns: &[crate::core::types::SensitivePattern]) -> bool {
    use regex::Regex;
    
    // 首先尝试匹配列名
    for pattern in patterns {
        for col_name in &pattern.column_names {
            // 使用单词边界来匹配完整的列名
            let col_regex = format!(r"(?i)\b{}\b", regex::escape(col_name));
            if let Ok(re) = Regex::new(&col_regex) {
                if re.is_match(sql) {
                    return true;
                }
            }
        }
        
        // 然后尝试匹配正则表达式模式
        if let Ok(re) = Regex::new(&pattern.pattern) {
            if re.is_match(sql) {
                return true;
            }
        }
    }
    
    false
}

/// 提取 SQL 语句中的字符串字面量
pub fn extract_string_literals(sql: &str) -> Vec<String> {
    use regex::Regex;
    
    let re = Regex::new(r###"'([^']*)'|"([^"]*)"###).unwrap();
    let mut literals = Vec::new();
    
    for cap in re.captures_iter(sql) {
        if let Some(literal) = cap.get(1).or_else(|| cap.get(2)) {
            let literal_str = literal.as_str();
            // 不添加空字符串
            if !literal_str.is_empty() {
                literals.push(literal_str.to_string());
            }
        }
    }
    
    literals
}

/// 标准化 SQL 语句中的标识符
pub fn normalize_identifier(identifier: &str, db_type: &DatabaseType) -> String {
    let config = get_default_config(db_type);
    
    // 移除引号
    let unquoted = if identifier.starts_with(&config.quote_char.to_string()) 
        && identifier.ends_with(&config.quote_char.to_string()) {
        &identifier[1..identifier.len()-1]
    } else {
        identifier
    };
    
    // 根据大小写敏感性处理
    if config.identifier_case_sensitive {
        unquoted.to_string()
    } else {
        unquoted.to_uppercase()
    }
}

/// 验证 SQL 语句的基本语法
pub fn validate_sql_syntax(sql: &str) -> Result<()> {
    let trimmed = sql.trim();
    
    if trimmed.is_empty() {
        return Err(crate::core::error::ParseError::SqlParseError(
            "SQL 语句为空".to_string()
        ));
    }
    
    // 检查基本的 SQL 关键字
    let has_keyword = trimmed.split_whitespace()
        .next()
        .map(|first_word| {
            matches!(first_word.to_uppercase().as_str(), 
                "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "CREATE" | 
                "DROP" | "ALTER" | "TRUNCATE" | "GRANT" | "REVOKE" | "EXECUTE")
        })
        .unwrap_or(false);
    
    if !has_keyword {
        return Err(crate::core::error::ParseError::SqlParseError(
            "SQL 语句缺少有效的关键字".to_string()
        ));
    }
    
    Ok(())
}

/// 计算 SQL 语句的复杂度
pub fn calculate_sql_complexity(sql: &str) -> u32 {
    let mut complexity = 0;
    
    // 基础复杂度
    complexity += 1;
    
    // 子查询增加复杂度
    let subquery_count = sql.matches("(").count() + sql.matches("SELECT").count() - 1;
    complexity += subquery_count as u32 * 2;
    
    // JOIN 增加复杂度
    let join_count = sql.to_uppercase().matches("JOIN").count();
    complexity += join_count as u32 * 3;
    
    // 聚合函数增加复杂度
    let agg_functions = vec!["COUNT", "SUM", "AVG", "MIN", "MAX", "GROUP_CONCAT"];
    for func in agg_functions {
        complexity += sql.to_uppercase().matches(func).count() as u32;
    }
    
    // WHERE 条件增加复杂度
    let where_count = sql.to_uppercase().matches("WHERE").count();
    complexity += where_count as u32 * 2;
    
    // ORDER BY 增加复杂度
    let order_count = sql.to_uppercase().matches("ORDER BY").count();
    complexity += order_count as u32;
    
    // GROUP BY 增加复杂度
    let group_count = sql.to_uppercase().matches("GROUP BY").count();
    complexity += group_count as u32 * 2;
    
    complexity
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_default_config() {
        let mysql_config = get_default_config(&DatabaseType::MySQL);
        assert_eq!(mysql_config.quote_char, '`');
        assert_eq!(mysql_config.identifier_case_sensitive, false);
        
        let pg_config = get_default_config(&DatabaseType::PostgreSQL);
        assert_eq!(pg_config.quote_char, '"');
        assert_eq!(pg_config.default_schema, Some("public".to_string()));
    }
    
    #[test]
    fn test_extract_string_literals() {
        let sql = "SELECT * FROM users WHERE name = 'John' AND email = \"john@example.com\"";
        let literals = extract_string_literals(sql);
        assert_eq!(literals, vec!["John".to_string(), "john@example.com".to_string()]);
    }
    
    #[test]
    fn test_normalize_identifier() {
        let mysql_id = normalize_identifier("`user_name`", &DatabaseType::MySQL);
        assert_eq!(mysql_id, "USER_NAME");
        
        let pg_id = normalize_identifier("\"user_name\"", &DatabaseType::PostgreSQL);
        assert_eq!(pg_id, "USER_NAME");
    }
    
    #[test]
    fn test_calculate_sql_complexity() {
        let simple_sql = "SELECT * FROM users";
        assert_eq!(calculate_sql_complexity(simple_sql), 1);
        
        let complex_sql = "SELECT u.name, COUNT(o.id) FROM users u JOIN orders o ON u.id = o.user_id WHERE u.status = 'active' GROUP BY u.name ORDER BY COUNT(o.id) DESC";
        assert!(calculate_sql_complexity(complex_sql) > 10);
    }
}