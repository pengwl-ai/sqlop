use crate::core::error::{ParseError, Result};
use crate::core::types::{AuditLog, DatabaseType, OperationType, ParseResult, SqlObject};
use sqlparser::ast::{Statement, TableFactor, ObjectName};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect, MsSqlDialect};
use sqlparser::parser::Parser;
use std::collections::HashSet;

// 导入自定义方言
use super::dialects::{SQLiteDialect, HiveDialect, DB2Dialect, DamengDialect, OracleDialect, GaussDBDialect, KingbaseDialect, HighgoDialect, GreenplumDialect, VastbaseDialect};

pub struct CommonAdapter {
    db_type: DatabaseType,
}

impl CommonAdapter {
    pub fn new(db_type: DatabaseType) -> Self {
        Self { db_type }
    }

    pub fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        let dialect = self.get_dialect(&audit_log.database_type)?;
        self.parse_sql_with_dialect(&audit_log.sql_text, &audit_log.database_type, dialect)
    }

    pub fn normalize_sql(&self, sql: &str) -> String {
        let mut normalized = sql.to_string();
        
        // 移除注释
        normalized = self.remove_comments(&normalized);
        
        // 标准化空白字符
        normalized = self.normalize_whitespace(&normalized);
        
        // 标准化大小写
        normalized = self.normalize_case(&normalized);
        
        // 标准化引号
        normalized = self.normalize_quotes(&normalized);
        
        normalized
    }

    pub fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        let normalized_sql = self.normalize_sql(sql);
        let dialect = self.get_dialect(&self.db_type)?;
        self.parse_sql_with_dialect(&normalized_sql, &self.db_type, dialect)
    }

    fn get_dialect(&self, db_type: &DatabaseType) -> Result<Box<dyn Dialect>> {
        match db_type {
            DatabaseType::MySQL => Ok(Box::new(MySqlDialect {})),
            DatabaseType::PostgreSQL => Ok(Box::new(PostgreSqlDialect {})),
            DatabaseType::SQLServer => Ok(Box::new(MsSqlDialect {})),
            DatabaseType::Oracle => Ok(Box::new(OracleDialect {})),
            DatabaseType::Hive => Ok(Box::new(HiveDialect {})),
            DatabaseType::GaussDB => Ok(Box::new(GaussDBDialect {})),
            DatabaseType::Kingbase => Ok(Box::new(KingbaseDialect {})),
            DatabaseType::Highgo => Ok(Box::new(HighgoDialect {})),
            DatabaseType::Greenplum => Ok(Box::new(GreenplumDialect {})),
            DatabaseType::Vastbase => Ok(Box::new(VastbaseDialect {})),
            DatabaseType::Sybase => Ok(Box::new(MySqlDialect {})),
            DatabaseType::DB2 => Ok(Box::new(DB2Dialect {})),
            DatabaseType::Dameng => Ok(Box::new(DamengDialect {})),
            DatabaseType::SQLite => Ok(Box::new(SQLiteDialect {})),
        }
    }

    fn parse_sql_with_dialect(
        &self,
        sql: &str,
        db_type: &DatabaseType,
        dialect: Box<dyn Dialect>,
    ) -> Result<ParseResult> {
        let mut parser = Parser::new(&*dialect).try_with_sql(sql)?;
        let statements = parser.parse_statements()?;

        if statements.is_empty() {
            return Err(ParseError::SqlParseError("无法解析 SQL 语句".to_string()));
        }

        let mut databases = HashSet::new();
        let mut schemas = HashSet::new();
        let mut tables = HashSet::new();
        let mut columns = HashSet::new();
        let mut objects = Vec::new();
        let mut operation_type = OperationType::OTHER;

        for statement in &statements {
            self.extract_objects_from_statement(
                statement,
                &mut databases,
                &mut schemas,
                &mut tables,
                &mut columns,
                &mut objects,
                &mut operation_type,
            );
        }

        Ok(ParseResult {
            database_type: db_type.clone(),
            original_sql: sql.to_string(),
            databases,
            schemas,
            tables,
            columns,
            objects,
            operation_type,
            parse_time_ms: 0,
        })
    }

    fn extract_objects_from_statement(
        &self,
        statement: &Statement,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
        operation_type: &mut OperationType,
    ) {
        match statement {
            Statement::Query(query) => {
                *operation_type = OperationType::SELECT;
                self.extract_from_query(query, databases, schemas, tables, columns, objects);
            }
            Statement::Insert { table_name, .. } => {
                *operation_type = OperationType::INSERT;
                // 提取表名信息
                self.extract_table_name(table_name, databases, schemas, tables, objects);
            }
            Statement::Update { table, .. } => {
                *operation_type = OperationType::UPDATE;
                // 提取表名信息
                // 修复：正确处理TableWithJoins类型
                match table.relation {
                    TableFactor::Table { ref name, .. } => {
                        self.extract_table_name(name, databases, schemas, tables, objects);
                    },
                    _ => {}
                }
            }
            Statement::Delete { from, .. } => {
                *operation_type = OperationType::DELETE;
                // 提取表名信息
                for table_ref in from {
                    match &table_ref.relation {
                        TableFactor::Table { ref name, .. } => {
                            self.extract_table_name(name, databases, schemas, tables, objects);
                        },
                        _ => {}
                    }
                }
            }
            Statement::CreateTable { name, .. } => {
                *operation_type = OperationType::CREATE;
                // 提取表名信息
                self.extract_table_name(name, databases, schemas, tables, objects);
            }
            Statement::Drop { names, .. } => {
                *operation_type = OperationType::DROP;
                // 提取表名信息
                for name in names {
                    self.extract_table_name(name, databases, schemas, tables, objects);
                }
            }
            Statement::AlterTable { name, .. } => {
                *operation_type = OperationType::ALTER;
                // 提取表名信息
                self.extract_table_name(name, databases, schemas, tables, objects);
            }
            Statement::Truncate { table_name, .. } => {
                *operation_type = OperationType::TRUNCATE;
                // 提取表名信息
                self.extract_table_name(table_name, databases, schemas, tables, objects);
            }
            _ => {}
        }
    }

    /// 从ObjectName提取表名信息
    fn extract_table_name(
        &self,
        name: &ObjectName,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        let mut database = None;
        let mut schema = None;
        let mut table_name = String::new();

        // 解析表名
        if name.0.len() == 1 {
            table_name = name.0[0].value.clone();
        } else if name.0.len() == 2 {
            schema = Some(name.0[0].value.clone());
            table_name = name.0[1].value.clone();
        } else if name.0.len() >= 3 {
            database = Some(name.0[0].value.clone());
            schema = Some(name.0[1].value.clone());
            table_name = name.0[2].value.clone();
        }

        if let Some(db) = &database {
            databases.insert(db.clone());
        }
        if let Some(sch) = &schema {
            schemas.insert(sch.clone());
        }
        tables.insert(table_name.clone());

        objects.push(SqlObject {
            database: database.clone(),
            schema: schema.clone(),
            table: table_name,
            column: None,
            alias: None,
        });
    }

    fn extract_from_query(
        &self,
        query: &sqlparser::ast::Query,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        columns: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        // 处理 SELECT 列
        if let sqlparser::ast::SetExpr::Select(select) = &*query.body {
            for select_item in &select.projection {
                self.extract_from_select_item(select_item, columns);
            }
            
            // 处理 FROM 子句中的表
            for table_with_join in &select.from {
                self.extract_from_table_factor(
                    &table_with_join.relation,
                    databases,
                    schemas,
                    tables,
                    objects,
                );
            }
        }
    }

    fn extract_from_table_factor(
        &self,
        table_factor: &TableFactor,
        databases: &mut HashSet<String>,
        schemas: &mut HashSet<String>,
        tables: &mut HashSet<String>,
        objects: &mut Vec<SqlObject>,
    ) {
        match table_factor {
            TableFactor::Table { name, alias, .. } => {
                let mut database = None;
                let mut schema = None;
                let mut table_name = String::new();

                if name.0.len() == 1 {
                    table_name = name.0[0].value.clone();
                } else if name.0.len() == 2 {
                    schema = Some(name.0[0].value.clone());
                    table_name = name.0[1].value.clone();
                } else if name.0.len() == 3 {
                    database = Some(name.0[0].value.clone());
                    schema = Some(name.0[1].value.clone());
                    table_name = name.0[2].value.clone();
                }

                if let Some(db) = &database {
                    databases.insert(db.clone());
                }
                if let Some(sch) = &schema {
                    schemas.insert(sch.clone());
                }
                tables.insert(table_name.clone());

                objects.push(SqlObject {
                    database: database.clone(),
                    schema: schema.clone(),
                    table: table_name,
                    column: None,
                    alias: alias.as_ref().map(|a| a.name.value.clone()),
                });
            }
            TableFactor::Derived { .. } => {}
            TableFactor::NestedJoin { .. } => {}
            _ => {}
        }
    }

    fn extract_from_select_item(&self, select_item: &sqlparser::ast::SelectItem, columns: &mut HashSet<String>) {
        match select_item {
            sqlparser::ast::SelectItem::UnnamedExpr(expr) => {
                self.extract_columns_from_expr(expr, columns);
            }
            sqlparser::ast::SelectItem::ExprWithAlias { expr, alias } => {
                self.extract_columns_from_expr(expr, columns);
                columns.insert(alias.value.clone());
            }
            sqlparser::ast::SelectItem::QualifiedWildcard(..) => {}
            sqlparser::ast::SelectItem::Wildcard(_) => {}
        }
    }

    fn extract_columns_from_expr(&self, expr: &sqlparser::ast::Expr, columns: &mut HashSet<String>) {
        match expr {
            sqlparser::ast::Expr::Identifier(ident) => {
                columns.insert(ident.value.clone());
            }
            sqlparser::ast::Expr::CompoundIdentifier(idents) => {
                if let Some(last_ident) = idents.last() {
                    columns.insert(last_ident.value.clone());
                }
            }
            sqlparser::ast::Expr::Function(function) => {
                for arg in &function.args {
                    if let sqlparser::ast::FunctionArg::Unnamed(expr) = arg {
                        if let sqlparser::ast::FunctionArgExpr::Expr(expr) = expr {
                            self.extract_columns_from_expr(expr, columns);
                        }
                    }
                }
            }
            sqlparser::ast::Expr::BinaryOp { left, right, .. } => {
                self.extract_columns_from_expr(left, columns);
                self.extract_columns_from_expr(right, columns);
            }
            _ => {}
        }
    }

    fn remove_comments(&self, sql: &str) -> String {
        let re_single_line = regex::Regex::new(r"--.*$").unwrap();
        let re_multi_line = regex::Regex::new(r"/\*.*?\*/").unwrap();
        
        let no_single_line = re_single_line.replace_all(sql, "");
        let no_multi_line = re_multi_line.replace_all(&no_single_line, "");
        
        no_multi_line.to_string()
    }

    fn normalize_whitespace(&self, sql: &str) -> String {
        let re_whitespace = regex::Regex::new(r"\s+").unwrap();
        re_whitespace.replace_all(sql.trim(), " ").to_string()
    }

    fn normalize_case(&self, sql: &str) -> String {
        // 将 SQL 关键字转换为大写
        let keywords = vec![
            "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "DROP",
            "ALTER", "TABLE", "INDEX", "VIEW", "JOIN", "INNER", "OUTER", "LEFT", "RIGHT",
            "ON", "GROUP", "BY", "ORDER", "HAVING", "LIMIT", "OFFSET", "UNION", "AND",
            "OR", "NOT", "IN", "EXISTS", "BETWEEN", "LIKE", "IS", "NULL", "COUNT", "SUM",
            "AVG", "MIN", "MAX", "DISTINCT", "AS", "CASE", "WHEN", "THEN", "ELSE", "END",
        ];
        
        let mut normalized = sql.to_string();
        for keyword in keywords {
            let re = regex::Regex::new(&format!(r"(?i)\b{}\b", keyword)).unwrap();
            normalized = re.replace_all(&normalized, keyword).to_string();
        }
        
        normalized
    }

    fn normalize_quotes(&self, sql: &str) -> String {
        // 标准化引号，将不同类型的引号统一为双引号
        let re_single_quotes = regex::Regex::new(r"'([^']*)'").unwrap();
        let re_backticks = regex::Regex::new(r"`([^`]*)`").unwrap();
        
        let step1 = re_single_quotes.replace_all(sql, "\"$1\"");
        let step2 = re_backticks.replace_all(&step1, "\"$1\"");
        
        step2.to_string()
    }
}

// 自定义方言已移至单独的dialects模块中实现