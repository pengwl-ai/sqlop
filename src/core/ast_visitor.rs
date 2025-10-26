// SQL抽象语法树访问者模式实现
// 参考JSQLParser的访问者模式设计，为sqlop提供更强大的AST处理能力

use regex::Regex;
use sqlparser::ast::{Statement, Select, SetExpr, Expr, SelectItem, ObjectName, Query, TableFactor, TableWithJoins};
use std::collections::{HashSet, HashMap};
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
    
    /// 访问CREATE VIEW语句
    fn visit_create_view(&mut self, statement: &Statement);
    
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
    
    /// 处理表源（包括普通表和子查询表）
    fn visit_table_source(&mut self, relation: &TableFactor);
    
    /// 处理查询语句
    fn visit_query(&mut self, query: &Query);
    
    /// 访问集合表达式（SELECT、UNION、INTERSECT、EXCEPT等）
    fn visit_set_expr(&mut self, set_expr: &SetExpr);
    
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
    // 判断是否为潜在的表名（即使是关键字也可能作为表名）
    fn is_potential_table_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // 这些词常见用作表名，即使也是SQL关键字
        const POTENTIAL_TABLE_NAMES: [&str; 12] = [
            "user", "users", "order", "orders", "group", "account", 
            "role", "roles", "type", "types", "status", "log"
        ];
        POTENTIAL_TABLE_NAMES.contains(&name_lower.as_str())
    }
    
    // 判断是否为潜在的列名（即使是关键字或函数名也可能作为列名）
    fn is_potential_column_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        // 这些词常见用作列名，即使也是SQL关键字或函数名
        const POTENTIAL_COLUMN_NAMES: [&str; 15] = [
            "name", "type", "order", "group", "count", "sum", 
            "min", "max", "avg", "date", "time", "key", 
            "value", "text", "status"
        ];
        POTENTIAL_COLUMN_NAMES.contains(&name_lower.as_str())
    }
    
    fn is_valid_column_identifier(&self, name: &str) -> bool {
        // 检查是否为空或只包含空格
        if name.trim().is_empty() {
            return false;
        }
        
        // 允许中文字符和其他非ASCII字符作为标识符
        // 允许SQL关键字和大多数特殊字符（带引号的标识符几乎可以包含任何字符）
        // 基本上只要不是空字符串就认为是有效的标识符
        true
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
                // 放宽表名过滤，仅过滤明显的SQL关键字，保留可能用作表名的常见词
                if !self.is_sql_keyword(table_name) || self.is_potential_table_name(table_name) {
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
                
                // 对数据库和模式名称更宽松处理，只验证表名部分
                // 允许任何非空字符串作为数据库和模式名称
                if database.trim().len() > 0 && schema.trim().len() > 0 {
                    self.databases.insert(database.to_string());
                    self.schemas.insert(schema.to_string());
                    
                    // 表名部分仍然需要基本验证，但也更宽松
                    if table.trim().len() > 0 && (!self.is_sql_keyword(table) || self.is_potential_table_name(table)) {
                        self.tables.insert(table.to_string());
                    }
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

impl ObjectExtractor {
    // 仅提取简单表达式中的列名，避免过度提取
    pub fn visit_simple_expr_columns(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(ident) => {
                let ident_name = &ident.value;
                // 只提取可能的列名
                if self.is_valid_column_identifier(ident_name) {
                    self.columns.insert(ident_name.to_string());
                }
            },
            Expr::CompoundIdentifier(idents) => {
                if let Some(last_ident) = idents.last() {
                    let column_name = &last_ident.value;
                    // 只提取可能的列名
                    if self.is_valid_column_identifier(column_name) {
                        self.columns.insert(column_name.to_string());
                    }
                }
            },
            Expr::BinaryOp { left, right, .. } => {
                // 递归处理二元操作符的两边
                self.visit_simple_expr_columns(left);
                self.visit_simple_expr_columns(right);
            },
            Expr::Function(func) => {
                // 处理函数参数中的简单列引用
                for arg in &func.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(expr_arg) = arg {
                        if let sqlparser::ast::FunctionArgExpr::Expr(arg_expr) = expr_arg {
                            self.visit_simple_expr_columns(arg_expr);
                        }
                    }
                }
            },
            Expr::Between { expr, low, high, .. } => {
                self.visit_simple_expr_columns(expr);
                self.visit_simple_expr_columns(low);
                self.visit_simple_expr_columns(high);
            },
            Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
                self.visit_simple_expr_columns(expr);
            },
            Expr::InList { expr, list, .. } => {
                self.visit_simple_expr_columns(expr);
                // 只处理列表中的简单表达式
                for item in list {
                    if let Expr::Identifier(_) | Expr::CompoundIdentifier(_) = item {
                        self.visit_simple_expr_columns(item);
                    }
                }
            },
            Expr::Like { expr, pattern, .. } => {
                self.visit_simple_expr_columns(expr);
                // 只处理模式中的简单表达式
                if let Expr::Identifier(_) | Expr::CompoundIdentifier(_) = **pattern {
                    self.visit_simple_expr_columns(pattern);
                }
            },
            Expr::Case { operand, conditions, results, else_result, .. } => {
                if let Some(op) = operand {
                    self.visit_simple_expr_columns(op);
                }
                for (cond, result) in conditions.iter().zip(results.iter()) {
                    self.visit_simple_expr_columns(cond);
                    self.visit_simple_expr_columns(result);
                }
                if let Some(else_res) = else_result {
                    self.visit_simple_expr_columns(else_res);
                }
            },
            Expr::Cast { expr, .. } | Expr::TryCast { expr, .. } => {
                self.visit_simple_expr_columns(expr);
            },
            _ => {
                // 对子查询和复杂表达式，不提取列名
            }
        }
    }
    
    // 仅提取表达式中的子查询表名，不提取列名
    pub fn visit_subquery_tables(&mut self, expr: &Expr) {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                self.visit_subquery_tables(left);
                self.visit_subquery_tables(right);
            },
            Expr::Exists { subquery, .. } => {
                // 处理EXISTS子查询中的表，不提取列名
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                // 仅合并表名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
            },
            Expr::InSubquery { subquery, .. } => {
                // 处理IN子查询中的表，不提取列名
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                // 仅合并表名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
            },
            Expr::Subquery(subquery) => {
                // 处理普通子查询中的表，不提取列名
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                // 仅合并表名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
            },
            Expr::Function(func) => {
                for arg in &func.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(expr_arg) = arg {
                        if let sqlparser::ast::FunctionArgExpr::Expr(arg_expr) = expr_arg {
                            self.visit_subquery_tables(arg_expr);
                        }
                    }
                }
            },
            Expr::Case { operand, conditions, results, else_result, .. } => {
                if let Some(op) = operand {
                    self.visit_subquery_tables(op);
                }
                for (cond, result) in conditions.iter().zip(results.iter()) {
                    self.visit_subquery_tables(cond);
                    self.visit_subquery_tables(result);
                }
                if let Some(else_res) = else_result {
                    self.visit_subquery_tables(else_res);
                }
            },
            _ => {},
        }
    }
}

impl SqlAstVisitor for ObjectExtractor {
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Query(query) => {
                self.operation_type = Some(OperationType::SELECT);
                // 处理查询语句
                self.visit_query(query);
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
            Statement::CreateView { .. } => {
                self.operation_type = Some(OperationType::CREATE);
                self.visit_create_view(statement);
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
            // 添加对其他语句类型的通用处理
            _ => {
                // 对于不支持的语句类型，尝试转换为字符串并查找表名模式
                // 这是一个后备策略
                let stmt_str = format!("{:?}", statement);
                // 查找常见的表名模式（如FROM table_name, JOIN table_name等）
                if let Ok(regex) = Regex::new(r"(?i)(?:FROM|JOIN|UPDATE|DELETE FROM|INTO)\s+([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)*)") {
                    for cap in regex.captures_iter(&stmt_str) {
                        if let Some(table_match) = cap.get(1) {
                            let table_name = table_match.as_str();
                            // 尝试提取表名（简化处理）
                            if let Some(last_part) = table_name.split('.').last() {
                                if !last_part.is_empty() && (!self.is_sql_keyword(last_part) || self.is_potential_table_name(last_part)) {
                                    self.tables.insert(last_part.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 新增查询处理方法
    fn visit_query(&mut self, query: &Query) {
        // 首先处理WITH子句中的CTE
        if let Some(with_clause) = &query.with {
            for cte in &with_clause.cte_tables {
                // 处理CTE内部的查询，提取其中的表名和列名
                let mut cte_extractor = ObjectExtractor::new();
                cte_extractor.visit_statement(&Statement::Query(cte.query.clone()));
                
                // 合并CTE中提取的表名、列名、数据库和模式到主查询
                for table in cte_extractor.tables {
                    self.tables.insert(table);
                }
                for column in cte_extractor.columns {
                    self.columns.insert(column);
                }
                for db in cte_extractor.databases {
                    self.databases.insert(db);
                }
                for schema in cte_extractor.schemas {
                    self.schemas.insert(schema);
                }
            }
        }
        
        // 处理查询语句的主体部分，支持各种集合操作（包括SELECT、UNION等）
        self.visit_set_expr(&*query.body);
    }
    
    /// 访问集合表达式（目前只处理SELECT语句）
    fn visit_set_expr(&mut self, set_expr: &SetExpr) {
        match set_expr {
            SetExpr::Select(select) => {
                self.visit_select(select);
            },
            SetExpr::SetOperation { left, right, .. } => {
                // 处理UNION/INTERSECT/EXCEPT等集合操作
                self.visit_set_expr(left);
                self.visit_set_expr(right);
            },
            _ => {
                // 其他类型的集合表达式暂时不处理
            },
        }
    }
    
    // 不再需要独立的extract_column_from_qualified方法，逻辑已内联到visit_expr中
    
    fn visit_select(&mut self, select: &Select) {
        // 首先处理FROM子句中的表名
        for from_item in &select.from {
            self.visit_table_source(&from_item.relation);
            
            // 处理JOIN子句中的表
            for join in &from_item.joins {
                self.visit_table_source(&join.relation);
                
                // 处理JOIN条件中的表达式
                match &join.join_operator {
                    sqlparser::ast::JoinOperator::Inner(constraint) => {
                        if let sqlparser::ast::JoinConstraint::On(expr) = constraint {
                            self.visit_subquery_tables(expr);
                        }
                    },
                    sqlparser::ast::JoinOperator::LeftOuter(constraint) => {
                        if let sqlparser::ast::JoinConstraint::On(expr) = constraint {
                            self.visit_subquery_tables(expr);
                        }
                    },
                    sqlparser::ast::JoinOperator::RightOuter(constraint) => {
                        if let sqlparser::ast::JoinConstraint::On(expr) = constraint {
                            self.visit_subquery_tables(expr);
                        }
                    },
                    sqlparser::ast::JoinOperator::FullOuter(constraint) => {
                        if let sqlparser::ast::JoinConstraint::On(expr) = constraint {
                            self.visit_subquery_tables(expr);
                        }
                    },
                    _ => {},
                }
            }
        }
        
        // 然后处理SELECT子句中的列名
        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(expr) => {
                    self.visit_expr(expr);
                },
                SelectItem::ExprWithAlias { expr, alias: _ } => {
                    // 只处理表达式，不提取别名以避免过度提取
                    self.visit_expr(expr);
                },
                SelectItem::QualifiedWildcard(qualifier, _) => {
                    // 处理限定通配符，例如 table.*
                    self.visit_table(qualifier);
                    // 添加通配符
                    self.columns.insert("*".to_string());
                },
                SelectItem::Wildcard(_) => {
                    // 处理通配符*
                    self.columns.insert("*".to_string());
                }
            }
        }
        
        // 只处理子查询中的表名，不提取WHERE和HAVING中的列名
        if let Some(where_clause) = &select.selection {
            self.visit_subquery_tables(where_clause);
        }
        
        if let Some(having_clause) = &select.having {
            self.visit_subquery_tables(having_clause);
        }
    }
    
    // 处理表源（包括普通表和子查询表）
    fn visit_table_source(&mut self, relation: &TableFactor) {
        match relation {
            TableFactor::Table { name, .. } => {
                // 只提取原始表名，不提取别名
                self.extract_table_name(&name);
            },
            TableFactor::Derived { subquery, .. } => {
                // 处理子查询作为表（派生表）
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                // 仅合并表名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
            },
            _ => {}
        }
    }
    
    // 处理子查询作为表
     fn visit_subquery_as_table(&mut self, _query: &Box<sqlparser::ast::Query>, _alias: Option<&sqlparser::ast::TableAlias>) {
         // 暂不实现，因为我们已经在visit_table_source中处理了子查询表
         // 这个方法是为了满足接口要求，但实际处理逻辑已迁移到其他地方
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
        if let Statement::CreateTable { name, columns, query, .. } = statement {
            // 提取表名
            self.extract_table_name(name);
            
            // 提取表列定义
            for column_def in columns {
                self.columns.insert(column_def.name.value.to_string());
            }
            
            // 处理CREATE TABLE AS SELECT语句中的子查询
            if let Some(subquery) = query {
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                
                // 合并子查询中的表名和列名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
                for column in temp_extractor.columns {
                    self.columns.insert(column);
                }
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
        // 暂时注释掉，需要查看正确的Truncate结构体字段
        // if let Statement::Truncate { table_name, .. } = statement {
        //     self.extract_table_name(table_name);
        // }
    }
    
    fn visit_create_view(&mut self, statement: &Statement) {
        if let Statement::CreateView { query, .. } = statement {
            // 处理视图定义中的查询，不再将视图名添加为表名
            let mut temp_extractor = ObjectExtractor::new();
            temp_extractor.visit_statement(&Statement::Query(query.clone()));
            
            // 合并查询中的表名、列名、数据库和模式
            for table in temp_extractor.tables {
                self.tables.insert(table);
            }
            for column in temp_extractor.columns {
                self.columns.insert(column);
            }
            for db in temp_extractor.databases {
                self.databases.insert(db);
            }
            for schema in temp_extractor.schemas {
                self.schemas.insert(schema);
            }
        }
    }
    
    fn visit_table(&mut self, table: &ObjectName) {
        // 直接调用提取表名的方法
        self.extract_table_name(table);
    }
    
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(ident) => {
                let ident_name = &ident.value;
                // 特殊处理列名
                if ident_name == "*" {
                    self.columns.insert("*".to_string());
                    return;
                }
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
                // 从复合标识符中提取列名
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
                // 只处理函数参数中的列引用，不处理函数名本身
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
                // 递归处理二元操作符的两边
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
                // 只处理列表中的简单表达式
                for item in list {
                    if let Expr::Identifier(_) | Expr::CompoundIdentifier(_) = item {
                        self.visit_expr(item);
                    }
                }
            },
            Expr::Like { expr, negated: _, pattern, escape_char: _ } => {
                self.visit_expr(expr);
                // 增强处理：检查模式中是否引用了列，支持复合标识符
                if let Expr::Identifier(_) | Expr::CompoundIdentifier(_) = **pattern {
                    self.visit_expr(pattern);
                }
            },
            Expr::Case { operand, conditions, results, else_result, .. } => {
                if let Some(operand) = operand {
                    self.visit_expr(operand);
                }
                for (cond, result) in conditions.iter().zip(results.iter()) {
                    self.visit_expr(cond);
                    self.visit_expr(result);
                }
                if let Some(else_res) = else_result {
                    self.visit_expr(else_res);
                }
            },
            Expr::Cast { expr, .. } | Expr::TryCast { expr, .. } => {
                self.visit_expr(expr);
            },
            // 简化子查询处理，子查询中的表名在visit_subquery_tables中处理
            // 处理子查询相关表达式
            Expr::Subquery(subq) => {
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subq.clone()));
                
                // 合并子查询中的表名和列名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
                for column in temp_extractor.columns {
                    self.columns.insert(column);
                }
            },
            Expr::InSubquery { subquery, .. } => {
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                
                // 合并子查询中的表名和列名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
                for column in temp_extractor.columns {
                    self.columns.insert(column);
                }
            },
            Expr::Exists { subquery, .. } => {
                let mut temp_extractor = ObjectExtractor::new();
                temp_extractor.visit_statement(&Statement::Query(subquery.clone()));
                
                // 合并子查询中的表名和列名
                for table in temp_extractor.tables {
                    self.tables.insert(table);
                }
                for column in temp_extractor.columns {
                    self.columns.insert(column);
                }
            },
            Expr::ArrayIndex { obj, indexes } => {
                self.visit_expr(obj);
                for idx in indexes {
                    self.visit_expr(idx);
                }
            },
            _ => {}
        }
    }
}