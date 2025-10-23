use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 数据库对象类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectType {
    Database,
    Schema,
    Table,
    View,
    Column,
    Index,
    Function,
    Procedure,
    User,
    Role,
    Trigger,
    Sequence,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLServer,
    Oracle,
    Hive,
    GaussDB,
    Kingbase,
    Highgo,
    Greenplum,
    Vastbase,
    Sybase,
    DB2,
    Dameng,
    SQLite,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::SQLServer => write!(f, "SQLServer"),
            DatabaseType::Oracle => write!(f, "Oracle"),
            DatabaseType::Hive => write!(f, "Hive"),
            DatabaseType::GaussDB => write!(f, "GaussDB"),
            DatabaseType::Kingbase => write!(f, "Kingbase"),
            DatabaseType::Highgo => write!(f, "Highgo"),
            DatabaseType::Greenplum => write!(f, "Greenplum"),
            DatabaseType::Vastbase => write!(f, "Vastbase"),
            DatabaseType::Sybase => write!(f, "Sybase"),
            DatabaseType::DB2 => write!(f, "DB2"),
            DatabaseType::Dameng => write!(f, "Dameng"),
            DatabaseType::SQLite => write!(f, "SQLite"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SqlObject {
    pub database: Option<String>,
    pub schema: Option<String>,
    pub table: String,
    pub column: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseResult {
    pub database_type: DatabaseType,
    pub original_sql: String,
    pub databases: HashSet<String>,
    pub schemas: HashSet<String>,
    pub tables: HashSet<String>,
    pub columns: HashSet<String>,
    pub objects: Vec<SqlObject>,
    pub operation_type: OperationType,
    pub parse_time_ms: u64,
}

impl ParseResult {
    /// 检查tables集合是否包含指定的表名（接受&str类型）
    pub fn tables_contains(&self, table_name: &str) -> bool {
        self.tables.contains(table_name)
    }
    
    /// 检查columns集合是否包含指定的列名（接受&str类型）
    pub fn columns_contains(&self, column_name: &str) -> bool {
        self.columns.contains(column_name)
    }
    
    /// 检查schemas集合是否包含指定的模式名（接受&str类型）
    pub fn schemas_contains(&self, schema_name: &str) -> bool {
        self.schemas.contains(schema_name)
    }
    
    /// 检查databases集合是否包含指定的数据库名（接受&str类型）
    pub fn databases_contains(&self, database_name: &str) -> bool {
        self.databases.contains(database_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
    CREATE,
    DROP,
    ALTER,
    TRUNCATE,
    GRANT,
    REVOKE,
    EXECUTE,
    OTHER,
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::OTHER
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: String,
    pub database_type: DatabaseType,
    pub user: Option<String>,
    pub client_ip: Option<String>,
    pub database_name: Option<String>,
    pub sql_text: String,
    pub execution_time_ms: Option<u64>,
    pub rows_affected: Option<u64>,
    pub status: String,
}

impl Default for AuditLog {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            timestamp: "".to_string(),
            database_type: DatabaseType::MySQL,
            user: None,
            client_ip: None,
            database_name: None,
            sql_text: "".to_string(),
            execution_time_ms: None,
            rows_affected: None,
            status: "success".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    pub max_parse_time_ms: u64,
    pub enable_cache: bool,
    pub cache_size: usize,
    pub enable_parallel: bool,
    pub max_workers: usize,
    pub sensitive_patterns: Vec<SensitivePattern>,
    pub database_configs: std::collections::HashMap<DatabaseType, DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivePattern {
    pub name: String,
    pub pattern: String,
    pub description: String,
    pub column_names: Vec<String>,
    pub table_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub quote_char: char,
    pub identifier_case_sensitive: bool,
    pub support_schema: bool,
    pub default_schema: Option<String>,
    pub custom_keywords: Vec<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_parse_time_ms: 1000,
            enable_cache: true,
            cache_size: 10000,
            enable_parallel: true,
            max_workers: 4,
            sensitive_patterns: vec![],
            database_configs: std::collections::HashMap::new(),
        }
    }
}

/// SQL语句位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlPosition {
    pub line: u64,
    pub column: u64,
    pub start_offset: u64,
    pub end_offset: u64,
}

impl Default for SqlPosition {
    fn default() -> Self {
        Self {
            line: 0,
            column: 0,
            start_offset: 0,
            end_offset: 0,
        }
    }
}

/// SQL对象血缘关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlLineage {
    pub source_objects: Vec<SqlObject>,
    pub target_objects: Vec<SqlObject>,
    pub operation_type: OperationType,
    pub condition: Option<String>,
}

/// 表达式类型扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionInfo {
    pub expression_type: String,
    pub complexity: u64,
    pub function_calls: Vec<String>,
    pub subqueries: Vec<String>,
    pub position: SqlPosition,
}

impl Default for ExpressionInfo {
    fn default() -> Self {
        Self {
            expression_type: "".to_string(),
            complexity: 0,
            function_calls: vec![],
            subqueries: vec![],
            position: SqlPosition::default(),
        }
    }
}

/// 增强的解析结果，包含更多AST相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedParseResult {
    pub base_result: ParseResult,
    pub lineages: Vec<SqlLineage>,
    pub expressions: Vec<ExpressionInfo>,
    pub warnings: Vec<String>,
    pub ast_node_count: u64,
    pub raw_sql: String,
    pub processed_sql: Option<String>,
    pub simplified_sql: String,
    pub is_enhanced_parsing: bool,
    pub error_message: Option<String>,
}

/// 解析器性能统计信息
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub parse_count: u64,
    pub total_parse_time_ms: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: usize,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            parse_count: 0,
            total_parse_time_ms: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_size: 0,
        }
    }
}