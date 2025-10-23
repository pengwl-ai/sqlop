// SQL抽象语法树访问者模式实现
// 参考JSQLParser的访问者模式设计，为sqlop提供更强大的AST处理能力

use sqlparser::ast::{Statement, Select, SetExpr, Expr, SelectItem, ObjectName};
use std::collections::HashSet;
use crate::core::types::{OperationType, SqlObject};

/// SQL AST访问者接口
pub trait SqlAstVisitor {
    /// 访问SQL语句
    fn visit_statement(&mut self, statement: &Statement);
    
    /// 访问SELECT语句
    fn visit_select(&mut self, select: &Select);
    
    /// 访问INSERT语句
    fn visit_insert(&mut self, statement: &Statement);
    
    /// 访问UPDATE语句
    fn visit_update(&mut self, statement: &Statement);
    
    /// 访问DELETE语句
    fn visit_delete(&mut self, statement: &Statement);
    
    /// 访问CREATE TABLE语句
    fn visit_create_table(&mut self, statement: &Statement);
    
    /// 访问DROP语句
    fn visit_drop(&mut self, statement: &Statement);
    
    /// 访问ALTER TABLE语句
    fn visit_alter_table(&mut self, statement: &Statement);
    
    /// 访问TRUNCATE语句
    fn visit_truncate(&mut self, statement: &Statement);
    
    /// 访问表达式
    fn visit_expr(&mut self, expr: &Expr);
    
    /// 访问表引用
    fn visit_table(&mut self, table: &ObjectName);
    
    /// 处理子查询作为表引用的情况
    fn visit_subquery_as_table(&mut self, query: &Box<sqlparser::ast::Query>, alias: Option<&sqlparser::ast::TableAlias>);
    
    /// 判断标识符是否为SQL关键字（使用结构化的方法）
    fn is_sql_keyword(&self, name: &str) -> bool {
        let keyword_lower = name.to_lowercase();
        // 基本SQL关键字集合
        const SQL_KEYWORDS: [&str; 60] = [
            "select", "from", "where", "join", "group", "by", "order", "insert", 
            "update", "delete", "create", "drop", "alter", "in", "not", "between", 
            "like", "exists", "inner", "left", "right", "full", "outer", "on", "as", 
            "having", "distinct", "all", "union", "intersect", "except", "limit", 
            "offset", "for", "while", "case", "when", "then", "else", "end",
            "and", "or", "is", "null", "true", "false", "with", "cross", "natural",
            "any", "some", "explain", "analyze", "rownum", "partition", "over", "window",
            "escape", "raise", "notice"
        ];
        
        SQL_KEYWORDS.contains(&keyword_lower.as_str())
    }
    
    /// 判断标识符是否为常见函数名
    fn is_common_function(&self, name: &str) -> bool {
        let function_lower = name.to_lowercase();
        const COMMON_FUNCTIONS: [&str; 30] = [
            "unnest", "explode", "cast", "convert", "count", "sum", "avg", "min", "max",
            "abs", "ceil", "floor", "round", "trim", "ltrim", "rtrim", "upper", "lower",
            "substring", "concat", "length", "coalesce", "ifnull", "isNull", "nvl",
            "date_format", "to_date", "from_unixtime", "unix_timestamp", "now"
        ];
        
        COMMON_FUNCTIONS.contains(&function_lower.as_str())
    }
    
    /// 判断标识符是否为常见的schema名称
    fn is_common_schema(&self, name: &str) -> bool {
        let schema_lower = name.to_lowercase();
        const COMMON_SCHEMAS: [&str; 10] = [
            "public", "dbo", "sys", "pg_catalog", "information_schema",
            "gp01", "gp02", "postgres", "mysql", "oracle"
        ];
        
        COMMON_SCHEMAS.contains(&schema_lower.as_str())
    }
    
    /// 判断标识符是否为特殊列名（如INPUT__FILE__NAME和key）
    fn is_special_column(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // 特殊列名集合
        const SPECIAL_COLUMNS: [&str; 2] = ["key", "input__file__name"];
        
        SPECIAL_COLUMNS.contains(&name_lower.as_str())
    }
    
    /// 基于AST上下文判断标识符是否为有效的列名
    fn is_valid_column_identifier(&self, name: &str) -> bool {
        // 空字符串或纯数字不是有效列名
        if name.is_empty() || name.chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // 清理名称中的引号
        let clean_name = name.trim_matches(&['`', '"', '[', ']'][..]);
        
        // 特殊列名总是有效的
        if self.is_special_column(clean_name) {
            return true;
        }
        
        // 检查是否为SQL关键字或常见函数名
        if self.is_sql_keyword(clean_name) || self.is_common_function(clean_name) {
            return false;
        }
        
        // 检查是否包含非法字符
        !clean_name.contains(['(', ')', ';', ',', '=', '<', '>', '&', '|', '!', '*', '/', '+', '-', '%', '^', '$', '{', '}', '~', '?', '@', '#'])
    }
}

/// SQL对象提取器，实现访问者模式来提取数据库对象信息
pub struct ObjectExtractor {
    pub databases: HashSet<String>,
    pub schemas: HashSet<String>,
    pub tables: HashSet<String>,
    pub columns: HashSet<String>,
    pub objects: Vec<SqlObject>,
    pub operation_type: Option<OperationType>,
}

impl ObjectExtractor {
    /// 创建新的对象提取器实例
    pub fn new() -> Self {
        ObjectExtractor {
            databases: HashSet::new(),
            schemas: HashSet::new(),
            tables: HashSet::new(),
            columns: HashSet::new(),
            objects: Vec::new(),
            operation_type: None,
        }
    }
    
    /// 提交提取的对象信息
    pub fn commit(&self) -> (HashSet<String>, HashSet<String>, HashSet<String>, HashSet<String>) {
        (self.databases.clone(), self.schemas.clone(), self.tables.clone(), self.columns.clone())
    }
    
    /// 从表名中提取数据库、模式和表名
    pub fn extract_table_name(&mut self, table: &ObjectName) {
        let parts: Vec<String> = table.0.iter().map(|ident| ident.value.clone()).collect();
        
        // 清理标识符并移除空部分
        let cleaned_parts: Vec<String> = parts
            .iter()
            .map(|part| part.trim_matches(&['`', '"', '[', ']'][..]).to_string())
            .filter(|part| !part.is_empty())
            .collect();
        
        // 处理不同长度的表名部分
        match cleaned_parts.len() {
            1 => {
                let table_name = &cleaned_parts[0];
                // 过滤掉SQL关键字和函数名
                if !self.is_sql_keyword(table_name) && !self.is_common_function(table_name) {
                    // 允许中文表名且长度合理
                    if table_name.len() <= 128 {
                        self.tables.insert(table_name.to_string());
                    }
                }
            },
            2 => {
                let first_part = &cleaned_parts[0];
                let second_part = &cleaned_parts[1];
                
                // 检查第一部分是否为常见schema名称
                if self.is_common_schema(first_part) {
                    // 常见schema情况下，第二部分始终作为表名
                    if !self.is_sql_keyword(second_part) && !self.is_common_function(second_part) {
                        self.schemas.insert(first_part.to_string());
                        self.tables.insert(second_part.to_string());
                    }
                } else {
                    // 非常见schema情况下，两部分都可能是表名或schema
                    // 但优先将第二部分作为表名
                    if !self.is_sql_keyword(second_part) && !self.is_common_function(second_part) {
                        self.tables.insert(second_part.to_string());
                        // 第一部分如果不是关键字或函数名，则作为schema
                        if !self.is_sql_keyword(first_part) && !self.is_common_function(first_part) {
                            self.schemas.insert(first_part.to_string());
                        }
                    }
                }
            },
            3 => {
                // 三部分表名：database.schema.table
                let database = &cleaned_parts[0];
                let schema = &cleaned_parts[1];
                let table = &cleaned_parts[2];
                
                // 确保都不是关键字或函数名
                if !self.is_sql_keyword(table) && !self.is_common_function(table) {
                    self.databases.insert(database.to_string());
                    self.schemas.insert(schema.to_string());
                    self.tables.insert(table.to_string());
                }
            },
            _ => {
                // 超过三部分的表名，只取最后一部分作为表名
                if let Some(table_part) = cleaned_parts.last() {
                    if !self.is_sql_keyword(table_part) && !self.is_common_function(table_part) {
                        self.tables.insert(table_part.to_string());
                    }
                }
            }
        }
    }
}

impl SqlAstVisitor for ObjectExtractor {
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Query(_query) => {
                 self.operation_type = Some(OperationType::SELECT);
                 // 简化处理，仅关注表名提取
                 // ...
              },
            Statement::Insert { .. } => {
                self.operation_type = Some(OperationType::INSERT);
                self.visit_insert(statement);
            },
            Statement::Update { .. } => {
                self.operation_type = Some(OperationType::UPDATE);
                self.visit_update(statement);
            },
            Statement::Delete { .. } => {
                self.operation_type = Some(OperationType::DELETE);
                self.visit_delete(statement);
            },
            Statement::CreateTable { .. } => {
                self.operation_type = Some(OperationType::CREATE);
                self.visit_create_table(statement);
            },
            Statement::Drop { .. } => {
                self.operation_type = Some(OperationType::DROP);
                self.visit_drop(statement);
            },
            Statement::AlterTable { .. } => {
                self.operation_type = Some(OperationType::ALTER);
                self.visit_alter_table(statement);
            },
            Statement::Truncate { .. } => {
                self.operation_type = Some(OperationType::TRUNCATE);
                self.visit_truncate(statement);
            },
            _ => {}
        }
    }
    
    fn visit_select(&mut self, select: &Select) {
        // 处理SELECT子句中的列
        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(expr) => {
                    self.visit_expr(expr);
                },
                SelectItem::ExprWithAlias { expr, alias: _ } => {
                    self.visit_expr(expr);
                },
                SelectItem::QualifiedWildcard(_qualifier, _) => {
                    // 暂时跳过处理
                },
                SelectItem::Wildcard(_) => {}
            }
        }
        
        // 处理FROM子句中的表引用 - 暂时简化处理
        // ...
        
        // 处理WHERE子句 - 暂时跳过复杂处理
        
        // 处理GROUP BY子句 - 暂时跳过复杂处理
        
        // 处理HAVING子句 - 暂时跳过复杂处理
    }
    
    fn visit_insert(&mut self, statement: &Statement) {
        if let Statement::Insert { table_name, columns, source: _, .. } = statement {
            // 提取表名
            self.extract_table_name(table_name);
            
            // 提取INSERT列名（包括特殊列名）
            for column in columns {
                let column_name = &column.value;
                // 直接添加列名到columns集合，包括INPUT__FILE__NAME和key等特殊列名
                self.columns.insert(column_name.to_string());
            }
        }
    }
    
    fn visit_update(&mut self, _statement: &Statement) {
        // 暂时跳过Update处理
    }
    
    fn visit_delete(&mut self, _statement: &Statement) {
        // 暂时跳过Delete处理
    }
    
    fn visit_create_table(&mut self, statement: &Statement) {
        if let Statement::CreateTable { name, columns, .. } = statement {
            // 提取表名
            self.extract_table_name(name);
            
            // 提取表列定义
            for column_def in columns {
                self.columns.insert(column_def.name.value.to_string());
            }
        }
    }
    
    fn visit_drop(&mut self, _statement: &Statement) {
        // 暂时跳过Drop处理
    }
    
    fn visit_alter_table(&mut self, _statement: &Statement) {
        // 暂时跳过AlterTable处理
    }
    
    fn visit_truncate(&mut self, statement: &Statement) {
        if let Statement::Truncate { table_name, .. } = statement {
            self.extract_table_name(table_name);
        }
    }
    
    fn visit_table(&mut self, table: &ObjectName) {
        // 直接调用提取表名的方法
        self.extract_table_name(table);
    }
    
    fn visit_subquery_as_table(&mut self, query: &Box<sqlparser::ast::Query>, alias: Option<&sqlparser::ast::TableAlias>) {
        // 处理子查询内部的语句
        // 直接处理子查询的主体
        match &*query.body {
            SetExpr::Select(select) => {
                self.visit_select(select);
            },
            _ => {}
        }
        
        // 处理子查询的别名（作为表名处理）
        if let Some(alias) = alias {
            let alias_name = &alias.name.value;
            // 检查别名是否为有效的表名（不是关键字或函数名）
            if !self.is_sql_keyword(alias_name) && !self.is_common_function(alias_name) {
                self.tables.insert(alias_name.to_string() + "$s");
                self.schemas.insert(alias_name.to_string() + "$s");
            }
        }
    }
    
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(ident) => {
                let ident_name = &ident.value;
                
                // 结构化判断是否为特殊列名
                if self.is_special_column(ident_name) {
                    self.columns.insert(ident_name.to_string());
                } else {
                    // 基本验证：不是关键字或函数名，并且符合列名规则
                    if !self.is_sql_keyword(ident_name) && !self.is_common_function(ident_name) && self.is_valid_column_identifier(ident_name) {
                        self.columns.insert(ident_name.to_string());
                    }
                }
            },
            Expr::CompoundIdentifier(idents) => {
                if let Some(last_ident) = idents.last() {
                    let column_name = &last_ident.value;
                    
                    // 特殊列名处理
                    if self.is_special_column(column_name) {
                        self.columns.insert(column_name.to_string());
                    } else {
                        // 基本验证：不是关键字或函数名，并且符合列名规则
                        if !self.is_sql_keyword(column_name) && !self.is_common_function(column_name) && self.is_valid_column_identifier(column_name) {
                            self.columns.insert(column_name.to_string());
                        }
                    }
                }
                
                // 前面的部分可能是表名或schema名
                if idents.len() >= 2 {
                    let table_ident = &idents[idents.len() - 2];
                    let table_name = &table_ident.value;
                    
                    // 过滤掉关键字和函数名
                    if !self.is_sql_keyword(table_name) && !self.is_common_function(table_name) {
                        self.tables.insert(table_name.to_string());
                    }
                }
            },
            Expr::Function(func) => {
                // 处理函数参数
                for arg in &func.args {
                    match arg {
                        sqlparser::ast::FunctionArg::Unnamed(expr) => {
                            match expr {
                                sqlparser::ast::FunctionArgExpr::Expr(expr) => {
                                    self.visit_expr(expr);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
            },
            Expr::BinaryOp { left, op: _, right } => {
                // 访问左右操作数
                self.visit_expr(left);
                self.visit_expr(right);
            },
            Expr::Between { expr, negated: _, low, high } => {
                self.visit_expr(expr);
                self.visit_expr(low);
                self.visit_expr(high);
            },
            Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
                self.visit_expr(expr);
            },
            Expr::InList { expr, list, negated: _ } => {
                self.visit_expr(expr);
                for item in list {
                    self.visit_expr(item);
                }
            },
            Expr::InSubquery { expr, subquery, negated: _ } => {
                self.visit_expr(expr);
                self.visit_subquery_as_table(subquery, None);
            },
            Expr::Like { expr, negated: _, pattern, escape_char: _ } => {
                self.visit_expr(expr);
                // 增强处理：检查模式中是否引用了列
                if let Expr::Identifier(_) = **pattern {
                    self.visit_expr(pattern);
                }
            },
            Expr::Subquery(subquery) => {
                self.visit_subquery_as_table(subquery, None);
            },
            Expr::Exists { subquery, negated: _ } => {
                self.visit_subquery_as_table(subquery, None);
            },
            Expr::Case { operand, conditions, results, else_result } => {
                if let Some(operand) = operand {
                    self.visit_expr(operand);
                }
                for (cond, result) in conditions.iter().zip(results.iter()) {
                    self.visit_expr(cond);
                    self.visit_expr(result);
                }
                if let Some(else_result) = else_result {
                    self.visit_expr(else_result);
                }
            },
            Expr::ArrayIndex { obj, indexes } => {
                self.visit_expr(obj);
                for idx in indexes {
                    self.visit_expr(idx);
                }
            },
            // 处理其他表达式类型
            Expr::Array(_) | Expr::Tuple(_) => {},
            Expr::Cast { expr, .. } | Expr::TryCast { expr, .. } | Expr::Extract { expr, .. } |
            Expr::Substring { expr, .. } | Expr::Trim { expr, .. } => {
                // 访问内部表达式
                self.visit_expr(expr);
            },
            _ => {}
        }
    }
}