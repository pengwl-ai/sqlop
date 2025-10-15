// SQL抽象语法树访问者模式实现
// 参考JSQLParser的访问者模式设计，为sqlop提供更强大的AST处理能力

use sqlparser::ast::{Statement, Select, SetExpr, TableFactor, JoinOperator, Expr, SelectItem, ObjectName, JoinConstraint, GroupByExpr};
use std::collections::{HashSet, VecDeque};
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
    pub fn new(databases: &mut HashSet<String>, 
               schemas: &mut HashSet<String>, 
               tables: &mut HashSet<String>, 
               columns: &mut HashSet<String>, 
               objects: &mut Vec<SqlObject>,
               operation_type: &mut OperationType) -> Self {
        // 窃取传入的集合的所有权，在visit完成后通过take方法返回结果
        Self {
            databases: std::mem::take(databases),
            schemas: std::mem::take(schemas),
            tables: std::mem::take(tables),
            columns: std::mem::take(columns),
            objects: std::mem::take(objects),
            operation_type: Some(std::mem::take(operation_type)),
        }
    }
    
    /// 提交提取结果到外部集合
    pub fn commit(mut self, 
                 databases: &mut HashSet<String>, 
                 schemas: &mut HashSet<String>, 
                 tables: &mut HashSet<String>, 
                 columns: &mut HashSet<String>, 
                 objects: &mut Vec<SqlObject>,
                 operation_type: &mut OperationType) {
        // 将结果归还到外部集合
        *databases = std::mem::take(&mut self.databases);
        *schemas = std::mem::take(&mut self.schemas);
        *tables = std::mem::take(&mut self.tables);
        *columns = std::mem::take(&mut self.columns);
        *objects = std::mem::take(&mut self.objects);
        *operation_type = self.operation_type.take().unwrap_or(OperationType::OTHER);
    }
    
    /// 从ObjectName提取表名信息
    fn extract_table_name(&mut self, name: &ObjectName) {
        let mut database = None;
        let mut schema = None;
        let mut table_name = String::new();

        // 解析表名并去除反引号
        if name.0.len() == 1 {
            table_name = name.0[0].value.clone().trim_matches('`').to_string();
        } else if name.0.len() == 2 {
            schema = Some(name.0[0].value.clone().trim_matches('`').to_string());
            table_name = name.0[1].value.clone().trim_matches('`').to_string();
        } else if name.0.len() >= 3 {
            database = Some(name.0[0].value.clone().trim_matches('`').to_string());
            schema = Some(name.0[1].value.clone().trim_matches('`').to_string());
            table_name = name.0[2].value.clone().trim_matches('`').to_string();
        }

        if let Some(db) = &database {
            self.databases.insert(db.clone());
        }
        if let Some(sch) = &schema {
            self.schemas.insert(sch.clone());
        }
        self.tables.insert(table_name.clone());

        self.objects.push(SqlObject {
            database: database.clone(),
            schema: schema.clone(),
            table: table_name,
            column: None,
            alias: None,
        });
    }
    
    /// 从SelectItem提取列信息
    fn extract_from_select_item(&mut self, select_item: &SelectItem) {
        match select_item {
            SelectItem::UnnamedExpr(expr) => {
                self.visit_expr(expr);
            }
            SelectItem::ExprWithAlias { expr, alias } => {
                self.visit_expr(expr);
                self.columns.insert(alias.value.clone());
            }
            SelectItem::QualifiedWildcard(obj_name, _) => {
                // 处理 table.* 形式
                self.visit_table(obj_name);
            }
            SelectItem::Wildcard(_) => {
                // 对于 * 不做特殊处理
            }
        }
    }
    
    /// 处理FROM子句中的表引用
    fn extract_from_table_factor(&mut self, table_factor: &TableFactor) {
        match table_factor {
            TableFactor::Table { name, .. } => {
                self.extract_table_name(name);
            },
            _ => {}
        }
    }
}

impl SqlAstVisitor for ObjectExtractor {
    fn visit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Query(query) => {
                if let SetExpr::Select(select) = &*query.body {
                    self.operation_type = Some(OperationType::SELECT);
                    self.visit_select(select);
                }
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
            _ => {
                self.operation_type = Some(OperationType::OTHER);
            }
        }
    }
    
    fn visit_select(&mut self, select: &Select) {
        // 处理SELECT列
        for select_item in &select.projection {
            self.extract_from_select_item(select_item);
        }
        
        // 处理FROM子句中的表引用和JOIN语句
        for table_with_joins in &select.from {
            // 处理主表
            self.extract_from_table_factor(&table_with_joins.relation);
            
            // 处理JOIN子句中的表
                for join in &table_with_joins.joins {
                    self.extract_from_table_factor(&join.relation);
                    
                    // 处理ON条件中的列
                    match &join.join_operator {
                        JoinOperator::Inner(condition) |
                        JoinOperator::LeftOuter(condition) |
                        JoinOperator::RightOuter(condition) |
                        JoinOperator::FullOuter(condition) => {
                            match condition {
                                JoinConstraint::On(expr) => {
                                    self.visit_expr(expr);
                                },
                                JoinConstraint::Using(cols) => {
                                    // 处理USING子句中的列
                                    for col in cols {
                                        self.columns.insert(col.value.clone());
                                    }
                                },
                                _ => {}
                            }
                        },
                        JoinOperator::CrossJoin => {
                            // Cross join没有条件
                        },
                        _ => {}
                    }
            }
        }
        
        // 处理WHERE子句中的列引用
        if let Some(selection) = &select.selection {
            self.visit_expr(selection);
        }
        
        // 处理GROUP BY子句中的列
        if let GroupByExpr::Expressions(exprs) = &select.group_by {
            for expr in exprs {
                self.visit_expr(expr);
            }
        }
        
        // 处理HAVING子句中的列
        if let Some(having) = &select.having {
            self.visit_expr(having);
        }
    }
    
    fn visit_insert(&mut self, statement: &Statement) {
        if let Statement::Insert { table_name, columns, .. } = statement {
            self.extract_table_name(table_name);
            
            // 提取INSERT语句中的列名
            for col in columns {
                self.columns.insert(col.value.clone());
            }
            
            // 处理SELECT子查询或VALUES子句中的表达式
            // sqlparser 0.43.0版本中InsertSource已经被重构，这里简化处理
        }
    }
    
    fn visit_update(&mut self, statement: &Statement) {
        if let Statement::Update { table, assignments, selection, .. } = statement {
            // 处理UPDATE语句中的表
            self.extract_from_table_factor(&table.relation);
            
            // 处理SET子句中的列和表达式
            for assignment in assignments {
                self.visit_expr(&assignment.value);
                if let Some(last_ident) = assignment.id.last() {
                    self.columns.insert(last_ident.value.clone().trim_matches('`').to_string());
                }
            }
            
            // 处理WHERE子句中的列
            if let Some(expr) = selection {
                self.visit_expr(expr);
            }
        }
    }
    
    fn visit_delete(&mut self, statement: &Statement) {
        if let Statement::Delete { from, selection, .. } = statement {
            // 处理DELETE语句中的表
            for table_ref in from {
                match &table_ref.relation {
                    TableFactor::Table { ref name, .. } => {
                        self.extract_table_name(name);
                    },
                    _ => {}
                }
            }
            
            // 处理WHERE子句中的列
            if let Some(selection) = selection {
                self.visit_expr(selection);
            }
        }
    }
    
    fn visit_create_table(&mut self, statement: &Statement) {
        if let Statement::CreateTable { name, columns, .. } = statement {
            self.extract_table_name(name);
            
            // 提取表列定义
            for column_def in columns {
                self.columns.insert(column_def.name.value.clone());
            }
        }
    }
    
    fn visit_drop(&mut self, statement: &Statement) {
        if let Statement::Drop { names, .. } = statement {
            // 处理DROP语句中的对象名
            for name in names {
                self.visit_table(name);
            }
        }
    }
    
    fn visit_alter_table(&mut self, statement: &Statement) {
        if let Statement::AlterTable { name, .. } = statement {
            self.extract_table_name(name);
            
            // 根据ALTER操作类型处理不同的内容
            // 这里可以根据需要添加更多的处理逻辑
        }
    }
    
    fn visit_truncate(&mut self, statement: &Statement) {
        if let Statement::Truncate { table_name, .. } = statement {
            self.extract_table_name(table_name);
        }
    }
    
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier(ident) => {
                self.columns.insert(ident.value.clone().trim_matches('`').to_string());
            }
            Expr::CompoundIdentifier(idents) => {
                if let Some(last_ident) = idents.last() {
                    self.columns.insert(last_ident.value.clone().trim_matches('`').to_string());
                }
            }
            Expr::Function(function) => {
                for arg in &function.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(expr) = arg {
                        if let sqlparser::ast::FunctionArgExpr::Expr(expr) = expr {
                            self.visit_expr(expr);
                        }
                    }
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::Nested(expr) => {
                self.visit_expr(expr);
            }
            Expr::Between { ref expr, ref low, ref high, .. } => {
                self.visit_expr(expr);
                self.visit_expr(low);
                self.visit_expr(high);
            }
            Expr::IsNull(expr) | Expr::IsNotNull(expr) => {
                self.visit_expr(expr);
            }
            Expr::InList { ref expr, ref list, .. } => {
                self.visit_expr(expr);
                for item in list {
                    self.visit_expr(item);
                }
            }
            Expr::InSubquery { ref expr, .. } => {
                self.visit_expr(expr);
            }
            Expr::Like { ref expr, ref pattern, .. } => {
                self.visit_expr(expr);
                self.visit_expr(pattern);
            }
            Expr::Subquery(query) => {
                // 处理子查询
                if let SetExpr::Select(select) = &*query.body {
                    self.visit_select(select);
                }
            }
            Expr::Array(_) => {
                // 不处理数组中的列
            }
            Expr::Tuple(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            Expr::Exists { .. } => {
                // Exists子查询中的列会在后续处理
            }
            // 其他类型的表达式处理
            _ => {}
        }
    }
    
    fn visit_table(&mut self, table: &ObjectName) {
        self.extract_table_name(table);
    }
}

/// SQL AST遍历器，提供更灵活的AST遍历功能
pub struct AstTraverser {
    visitors: VecDeque<Box<dyn SqlAstVisitor>>,
}

impl AstTraverser {
    pub fn new() -> Self {
        Self {
            visitors: VecDeque::new(),
        }
    }
    
    pub fn add_visitor(&mut self, visitor: Box<dyn SqlAstVisitor>) {
        self.visitors.push_back(visitor);
    }
    
    pub fn traverse(&mut self, statement: &Statement) {
        for visitor in &mut self.visitors {
            visitor.visit_statement(statement);
        }
    }
}