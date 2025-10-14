pub mod mysql;
pub mod postgresql;
pub mod sqlserver;
pub mod oracle;
pub mod hive;
pub mod common;
pub mod gaussdb;
pub mod kingbase;
pub mod highgo;
pub mod sybase;
pub mod db2;
pub mod dameng;
pub mod sqlite;
// 数据库方言实现模块
pub mod dialects;

use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};

pub trait DatabaseAdapter {
    fn can_handle(&self, db_type: &DatabaseType) -> bool;
    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult>;
    fn get_database_type(&self) -> DatabaseType;
    fn normalize_sql(&self, sql: &str) -> String;
    fn extract_metadata(&self, sql: &str) -> Result<ParseResult>;
}

pub struct AdapterManager {
    adapters: Vec<Box<dyn DatabaseAdapter>>,
}

impl AdapterManager {
    pub fn new() -> Self {
        let mut adapters: Vec<Box<dyn DatabaseAdapter>> = Vec::new();
        
        // 注册所有适配器
        adapters.push(Box::new(mysql::MySQLAdapter::new()));
        adapters.push(Box::new(postgresql::PostgreSQLAdapter::new()));
        adapters.push(Box::new(sqlserver::SQLServerAdapter::new()));
        adapters.push(Box::new(oracle::OracleAdapter::new()));
        adapters.push(Box::new(hive::HiveAdapter::new()));
        adapters.push(Box::new(gaussdb::GaussDBAdapter::new()));
        adapters.push(Box::new(kingbase::KingbaseAdapter::new()));
        adapters.push(Box::new(highgo::HighgoAdapter::new()));
        adapters.push(Box::new(sybase::SybaseAdapter::new()));
        adapters.push(Box::new(db2::DB2Adapter::new()));
        adapters.push(Box::new(dameng::DamengAdapter::new()));
        adapters.push(Box::new(sqlite::SQLiteAdapter::new()));
        
        Self { adapters }
    }

    pub fn get_adapter(&self, db_type: &DatabaseType) -> Option<&dyn DatabaseAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.can_handle(db_type))
            .map(|adapter| adapter.as_ref())
    }

    pub fn register_adapter(&mut self, adapter: Box<dyn DatabaseAdapter>) {
        self.adapters.push(adapter);
    }

    pub fn list_supported_databases(&self) -> Vec<DatabaseType> {
        self.adapters
            .iter()
            .map(|adapter| adapter.get_database_type())
            .collect()
    }
}

impl Default for AdapterManager {
    fn default() -> Self {
        Self::new()
    }
}