// 数据库方言实现模块
pub mod sqlite_dialect;
pub mod hive_dialect;
pub mod db2_dialect;
pub mod dameng_dialect;
pub mod oracle_dialect;
pub mod gaussdb_dialect;
pub mod kingbase_dialect;
pub mod highgo_dialect;
pub mod greenplum_dialect;
pub mod vastbase_dialect;
pub mod enhanced_mysql_dialect;

// 导出所有方言结构体
pub use sqlite_dialect::SQLiteDialect;
pub use hive_dialect::HiveDialect;
pub use db2_dialect::DB2Dialect;
pub use dameng_dialect::DamengDialect;
pub use oracle_dialect::OracleDialect;
pub use gaussdb_dialect::GaussDBDialect;
pub use kingbase_dialect::KingbaseDialect;
pub use highgo_dialect::HighgoDialect;
pub use greenplum_dialect::GreenplumDialect;
pub use vastbase_dialect::VastbaseDialect;
pub use enhanced_mysql_dialect::EnhancedMySqlDialect;