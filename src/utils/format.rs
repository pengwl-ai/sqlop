use crate::core::error::Result;
use crate::core::types::DatabaseType;

pub struct SqlFormatter {
    database_type: DatabaseType,
    indent_size: usize,
    uppercase_keywords: bool,
}

impl SqlFormatter {
    pub fn new(database_type: DatabaseType) -> Self {
        Self {
            database_type,
            indent_size: 4,
            uppercase_keywords: true,
        }
    }

    pub fn format(&self, sql: &str) -> Result<String> {
        let mut formatted = String::new();
        let mut indent_level = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        let mut in_comment = false;
        for line in sql.lines() {
            let mut chars = line.chars().peekable();
            let mut current_line = String::new();
            let mut needs_indent = true;

            while let Some(ch) = chars.next() {
                // 处理字符串字面量
                if !in_comment && (ch == '\'' || ch == '"') {
                    if !in_string {
                        in_string = true;
                        string_char = ch;
                    } else if ch == string_char {
                        // 检查是否是转义字符
                        if let Some(prev_char) = current_line.chars().last() {
                            if prev_char != '\\' {
                                in_string = false;
                            }
                        } else {
                            in_string = false;
                        }
                    }
                }

                // 处理注释
                if !in_string && ch == '-' && chars.peek() == Some(&'-') {
                    in_comment = true;
                    current_line.push(ch);
                    if let Some(next_ch) = chars.next() {
                        current_line.push(next_ch);
                    }
                    continue;
                }

                if in_comment {
                    current_line.push(ch);
                    continue;
                }

                if in_string {
                    current_line.push(ch);
                    continue;
                }

                // 处理关键字和格式化
                match ch {
                    '(' => {
                        current_line.push(ch);
                        indent_level += 1;
                        current_line.push('\n');
                        current_line.push_str(&self.get_indent(indent_level));
                        needs_indent = false;
                    }
                    ')' => {
                        indent_level = indent_level.saturating_sub(1);
                        current_line.push('\n');
                        current_line.push_str(&self.get_indent(indent_level));
                        current_line.push(ch);
                    }
                    ',' => {
                        current_line.push(ch);
                        current_line.push('\n');
                        current_line.push_str(&self.get_indent(indent_level));
                        needs_indent = false;
                    }
                    ';' => {
                        current_line.push(ch);
                        current_line.push('\n');
                        current_line.push('\n');
                        indent_level = 0;
                    }
                    ' ' | '\t' => {
                        // 跳过多余的空白字符
                        if let Some(last_char) = current_line.chars().last() {
                            if last_char != ' ' && last_char != '\n' {
                                current_line.push(' ');
                            }
                        }
                    }
                    '\n' => {
                        // 跳过换行符，在最后统一处理
                    }
                    _ => {
                        if needs_indent && !current_line.trim().is_empty() {
                            current_line.push_str(&self.get_indent(indent_level));
                            needs_indent = false;
                        }
                        current_line.push(ch);
                    }
                }
            }

            // 处理关键字大小写
            if self.uppercase_keywords {
                current_line = self.uppercase_keywords_in_line(&current_line);
            }

            // 处理数据库特定的格式化
            current_line = self.apply_database_specific_formatting(&current_line);

            if !current_line.trim().is_empty() {
                formatted.push_str(&current_line);
                formatted.push('\n');
            }

            in_comment = false;
        }

        Ok(formatted.trim().to_string())
    }

    fn get_indent(&self, level: usize) -> String {
        " ".repeat(level * self.indent_size)
    }

    fn uppercase_keywords_in_line(&self, line: &str) -> String {
        let keywords = vec![
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP",
            "ALTER", "TABLE", "INDEX", "VIEW", "JOIN", "INNER", "OUTER", "LEFT", "RIGHT",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "UNION", "AND",
            "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL", "COUNT", "SUM",
            "AVG", "MIN", "MAX", "DISTINCT", "AS", "CASE", "WHEN", "THEN", "ELSE", "END",
            "PRIMARY", "KEY", "FOREIGN", "REFERENCES", "CONSTRAINT", "DEFAULT", "UNIQUE",
            "CHECK", "TRIGGER", "PROCEDURE", "FUNCTION", "DATABASE", "SCHEMA", "USER",
            "GRANT", "REVOKE", "COMMIT", "ROLLBACK", "BEGIN", "TRANSACTION", "SAVEPOINT",
        ];

        let mut result = line.to_string();
        for keyword in keywords {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", keyword)).unwrap();
            result = re.replace_all(&result, keyword).to_string();
        }
        result
    }

    fn apply_database_specific_formatting(&self, line: &str) -> String {
        match self.database_type {
            DatabaseType::MySQL => self.format_mysql_specific(line),
            DatabaseType::PostgreSQL => self.format_postgresql_specific(line),
            DatabaseType::SQLServer => self.format_sqlserver_specific(line),
            DatabaseType::Oracle => self.format_oracle_specific(line),
            DatabaseType::Hive => self.format_hive_specific(line),
            _ => line.to_string(),
        }
    }

    fn format_mysql_specific(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // 处理 MySQL 反引号

        
        // 处理 MySQL 特有的函数
        let mysql_functions = vec![
            ("NOW()", "NOW()"),
            ("CURDATE()", "CURDATE()"),
            ("CURTIME()", "CURTIME()"),
            ("UNIX_TIMESTAMP()", "UNIX_TIMESTAMP()"),
            ("FROM_UNIXTIME", "FROM_UNIXTIME"),
            ("DATE_FORMAT", "DATE_FORMAT"),
            ("IF", "IF"),
        ];
        
        for (func, replacement) in mysql_functions {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", func)).unwrap();
            formatted = re.replace_all(&formatted, replacement).to_string();
        }
        
        formatted
    }

    fn format_postgresql_specific(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // 处理 PostgreSQL 双引号

        
        // 处理 PostgreSQL 特有的函数
        let pg_functions = vec![
            ("CURRENT_TIMESTAMP", "CURRENT_TIMESTAMP"),
            ("CURRENT_DATE", "CURRENT_DATE"),
            ("CURRENT_TIME", "CURRENT_TIME"),
            ("EXTRACT", "EXTRACT"),
            ("TO_CHAR", "TO_CHAR"),
            ("TO_DATE", "TO_DATE"),
            ("TO_TIMESTAMP", "TO_TIMESTAMP"),
            ("ARRAY_AGG", "ARRAY_AGG"),
            ("STRING_AGG", "STRING_AGG"),
        ];
        
        for (func, replacement) in pg_functions {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", func)).unwrap();
            formatted = re.replace_all(&formatted, replacement).to_string();
        }
        
        formatted
    }

    fn format_sqlserver_specific(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // 处理 SQL Server 方括号

        
        // 处理 SQL Server 特有的函数
        let sqlserver_functions = vec![
            ("GETDATE()", "GETDATE()"),
            ("DATEADD", "DATEADD"),
            ("DATEDIFF", "DATEDIFF"),
            ("DATENAME", "DATENAME"),
            ("DATEPART", "DATEPART"),
            ("CONVERT", "CONVERT"),
            ("CAST", "CAST"),
            ("ISNULL", "ISNULL"),
            ("COALESCE", "COALESCE"),
        ];
        
        for (func, replacement) in sqlserver_functions {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", func)).unwrap();
            formatted = re.replace_all(&formatted, replacement).to_string();
        }
        
        formatted
    }

    fn format_oracle_specific(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // 处理 Oracle 双引号

        
        // 处理 Oracle 特有的函数
        let oracle_functions = vec![
            ("SYSDATE", "SYSDATE"),
            ("TO_DATE", "TO_DATE"),
            ("TO_CHAR", "TO_CHAR"),
            ("TO_NUMBER", "TO_NUMBER"),
            ("NVL", "NVL"),
            ("DECODE", "DECODE"),
            ("ROWNUM", "ROWNUM"),
            ("ROWID", "ROWID"),
        ];
        
        for (func, replacement) in oracle_functions {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", func)).unwrap();
            formatted = re.replace_all(&formatted, replacement).to_string();
        }
        
        formatted
    }

    fn format_hive_specific(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // 处理 Hive 反引号

        
        // 处理 Hive 特有的函数
        let hive_functions = vec![
            ("CURRENT_TIMESTAMP", "CURRENT_TIMESTAMP"),
            ("UNIX_TIMESTAMP", "UNIX_TIMESTAMP"),
            ("FROM_UNIXTIME", "FROM_UNIXTIME"),
            ("DATE_FORMAT", "DATE_FORMAT"),
            ("TO_DATE", "TO_DATE"),
            ("YEAR", "YEAR"),
            ("MONTH", "MONTH"),
            ("DAY", "DAY"),
            ("HOUR", "HOUR"),
            ("MINUTE", "MINUTE"),
            ("SECOND", "SECOND"),
        ];
        
        for (func, replacement) in hive_functions {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", func)).unwrap();
            formatted = re.replace_all(&formatted, replacement).to_string();
        }
        
        formatted
    }

    pub fn set_indent_size(&mut self, size: usize) {
        self.indent_size = size;
    }

    pub fn set_uppercase_keywords(&mut self, uppercase: bool) {
        self.uppercase_keywords = uppercase;
    }

    pub fn minify(&self, sql: &str) -> Result<String> {
        let mut minified = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut in_comment = false;
        let mut last_char_was_space = false;

        for ch in sql.chars() {
            // 处理字符串字面量
            if !in_comment && (ch == '\'' || ch == '"') {
                if !in_string {
                    in_string = true;
                    string_char = ch;
                } else if ch == string_char {
                    if let Some(prev_char) = minified.chars().last() {
                        if prev_char != '\\' {
                            in_string = false;
                        }
                    } else {
                        in_string = false;
                    }
                }
            }

            // 处理注释
            if !in_string && ch == '-' {
                if let Some(next_ch) = minified.chars().last() {
                    if next_ch == '-' {
                        in_comment = true;
                        minified.pop(); // 移除前一个 '-' 
                        continue;
                    }
                }
            }

            if in_comment {
                if ch == '\n' {
                    in_comment = false;
                }
                continue;
            }

            if in_string {
                minified.push(ch);
                last_char_was_space = false;
                continue;
            }

            // 压缩空白字符
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    if !last_char_was_space && !minified.is_empty() {
                        let last_char = minified.chars().last().unwrap();
                        if !last_char.is_ascii_punctuation() || (last_char == ',' && ch == ' ') {
                            minified.push(' ');
                            last_char_was_space = true;
                        }
                    }
                }
                ',' => {
                    minified.push(ch);
                    minified.push(' ');
                    last_char_was_space = true;
                }
                '=' => {
                    if !minified.is_empty() && !last_char_was_space {
                        minified.push(' ');
                    }
                    minified.push(ch);
                    minified.push(' ');
                    last_char_was_space = true;
                }
                _ => {
                    if last_char_was_space {
                        let _last_char = minified.chars().last().unwrap_or(' ');
                        if ch.is_ascii_punctuation() && ch != '_' {
                            minified.pop(); // 移除标点前的空格
                        }
                    }
                    minified.push(ch);
                    last_char_was_space = false;
                }
            }
        }

        Ok(minified.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let formatter = SqlFormatter::new(DatabaseType::MySQL);
        let sql = "select name,age from users where id=1";
        let formatted = formatter.format(sql).unwrap();
        
        assert!(formatted.contains("SELECT"));
        assert!(formatted.contains("FROM"));
        assert!(formatted.contains("WHERE"));
    }

    #[test]
    fn test_indentation() {
        let formatter = SqlFormatter::new(DatabaseType::MySQL);
        let sql = "select * from users where id in (select id from temp)";
        let formatted = formatter.format(sql).unwrap();
        
        assert!(formatted.contains('\n'));
        assert!(formatted.contains("    "));
    }

    #[test]
    fn test_mysql_formatting() {
        let formatter = SqlFormatter::new(DatabaseType::MySQL);
        let sql = "select `name`, now() from users";
        let formatted = formatter.format(sql).unwrap();
        
        assert!(formatted.contains("`name`"));
        assert!(formatted.contains("NOW()"));
    }

    #[test]
    fn test_postgresql_formatting() {
        let formatter = SqlFormatter::new(DatabaseType::PostgreSQL);
        let sql = "select \"name\", current_timestamp from users";
        let formatted = formatter.format(sql).unwrap();
        
        assert!(formatted.contains("\"name\""));
        assert!(formatted.contains("CURRENT_TIMESTAMP"));
    }

    #[test]
    fn test_minify() {
        let formatter = SqlFormatter::new(DatabaseType::MySQL);
        let sql = "SELECT   name,  age FROM   users WHERE id = 1";
        let minified = formatter.minify(sql).unwrap();
        
        assert!(!minified.contains("  "));
        assert!(!minified.contains('\n'));
        assert!(minified.contains("SELECT name, age FROM users WHERE id = 1"));
    }
}