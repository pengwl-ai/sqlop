use crate::adapters::common::CommonAdapter;
use crate::core::error::Result;
use crate::core::types::{AuditLog, DatabaseType, ParseResult};

pub struct SQLServerAdapter {
    common: CommonAdapter,
}

impl SQLServerAdapter {
    pub fn new() -> Self {
        Self {
            common: CommonAdapter::new(DatabaseType::SQLServer),
        }
    }
}

impl super::DatabaseAdapter for SQLServerAdapter {
    fn can_handle(&self, db_type: &DatabaseType) -> bool {
        matches!(db_type, DatabaseType::SQLServer)
    }

    fn parse_audit_log(&self, audit_log: &AuditLog) -> Result<ParseResult> {
        self.common.parse_audit_log(audit_log)
    }

    fn get_database_type(&self) -> DatabaseType {
        DatabaseType::SQLServer
    }

    fn normalize_sql(&self, sql: &str) -> String {
        self.common.normalize_sql(sql)
    }

    fn extract_metadata(&self, sql: &str) -> Result<ParseResult> {
        self.common.extract_metadata(sql)
    }
}