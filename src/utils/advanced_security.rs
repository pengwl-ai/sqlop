// 高级安全检查模块
// 实现细粒度权限控制和高级SQL安全检查功能

use std::collections::{HashMap, HashSet};
use regex::Regex;
use crate::core::types::{OperationType, SqlObject, ObjectType};
// 移除不存在的导入

/// 安全规则引擎
pub struct SecurityRuleEngine {
    // 定义的安全规则
    rules: Vec<Box<dyn SecurityRule>>,
    // 权限策略管理器
    permission_manager: PermissionManager,
    // SQL注入检测模式
    injection_patterns: Vec<Regex>,
    // 敏感数据模式
    sensitive_data_patterns: HashMap<String, Regex>,
}

impl SecurityRuleEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            rules: Vec::new(),
            permission_manager: PermissionManager::new(),
            injection_patterns: Vec::new(),
            sensitive_data_patterns: HashMap::new(),
        };
        
        // 初始化内置规则
        engine.init_builtin_rules();
        // 初始化SQL注入检测模式
        engine.init_injection_patterns();
        // 初始化敏感数据模式
        engine.init_sensitive_data_patterns();
        
        engine
    }
    
    fn init_builtin_rules(&mut self) {
        // 添加内置安全规则
        self.add_rule(Box::new(SqlInjectionRule));
        self.add_rule(Box::new(SensitiveOperationRule));
        self.add_rule(Box::new(SensitiveDataAccessRule));
        self.add_rule(Box::new(PermissionCheckRule));
        self.add_rule(Box::new(ResourceLimitRule));
    }
    
    fn init_injection_patterns(&mut self) {
        // SQL注入检测正则表达式
        self.injection_patterns.push(Regex::new(r"(?i)UNION\s+SELECT").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)OR\s+1\s*=\s*1").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)AND\s+1\s*=\s*1").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)DROP\s+TABLE").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)DELETE\s+FROM").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)INSERT\s+INTO").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i);--").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)EXEC\s*\(").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)xp_cmdshell").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)sp_executesql").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)pg_catalog\.pg_file_.*").unwrap());
        self.injection_patterns.push(Regex::new(r"(?i)LOAD\s+DATA").unwrap());
    }
    
    fn init_sensitive_data_patterns(&mut self) {
        // 敏感数据检测正则表达式
        self.sensitive_data_patterns.insert("credit_card".to_string(), Regex::new(r"\b(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\d{3})\d{11})\b").unwrap());
        self.sensitive_data_patterns.insert("ssn".to_string(), Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap());
        self.sensitive_data_patterns.insert("phone".to_string(), Regex::new(r"\b\d{3}[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap());
        self.sensitive_data_patterns.insert("email".to_string(), Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());
        self.sensitive_data_patterns.insert("password".to_string(), Regex::new(r#"(?i)password\s*[=:]\s*['"][^'"]*['"]"#).unwrap());
        self.sensitive_data_patterns.insert("username".to_string(), Regex::new(r#"(?i)username\s*[=:]\s*['"][^'"]*['"]"#).unwrap());
        self.sensitive_data_patterns.insert("api_key".to_string(), Regex::new(r#"(?i)api[_-]?key\s*[=:]\s*['"][^'"]*['"]"#).unwrap());
        self.sensitive_data_patterns.insert("token".to_string(), Regex::new(r#"(?i)token\s*[=:]\s*['"][^'"]*['"]"#).unwrap());
    }
    
    pub fn add_rule(&mut self, rule: Box<dyn SecurityRule>) {
        self.rules.push(rule);
    }
    
    pub fn check_security(&self, sql: &str, user: &str, database_objects: &[SqlObject]) -> SecurityCheckResult {
        let mut result = SecurityCheckResult {
            warnings: Vec::new(),
            errors: Vec::new(),
            security_info: SecurityInfo::default(),
            is_secure: true,
        };
        
        // 检查SQL注入风险
        result.security_info.is_injection_risk = self.detect_sql_injection(sql);
        
        // 检查敏感数据访问
        let sensitive_data = self.detect_sensitive_data(sql);
        if !sensitive_data.is_empty() {
            result.security_info.accesses_sensitive_data = true;
            for (_data_type, values) in sensitive_data {
                result.security_info.sensitive_data_accessed.extend(values);
            }
        }
        
        // 执行所有安全规则检查
        for rule in &self.rules {
            let rule_result = rule.check(sql, user, database_objects, self);
            
            // 合并结果
            result.warnings.extend(rule_result.warnings);
            result.errors.extend(rule_result.errors);
            
            // 更新安全信息
            result.security_info.is_injection_risk = result.security_info.is_injection_risk || rule_result.security_info.is_injection_risk;
            result.security_info.is_sensitive_operation = result.security_info.is_sensitive_operation || rule_result.security_info.is_sensitive_operation;
            result.security_info.accesses_sensitive_data = result.security_info.accesses_sensitive_data || rule_result.security_info.accesses_sensitive_data;
            result.security_info.has_permission_violation = result.security_info.has_permission_violation || rule_result.security_info.has_permission_violation;
            result.security_info.exceeds_resource_limits = result.security_info.exceeds_resource_limits || rule_result.security_info.exceeds_resource_limits;
            result.security_info.sensitive_data_accessed.extend(rule_result.security_info.sensitive_data_accessed);
            result.security_info.protected_objects_accessed.extend(rule_result.security_info.protected_objects_accessed);
        }
        
        // 如果有错误，标记为不安全
        if !result.errors.is_empty() {
            result.is_secure = false;
        }
        
        result
    }
    
    pub fn detect_sql_injection(&self, sql: &str) -> bool {
        for pattern in &self.injection_patterns {
            if pattern.is_match(sql) {
                return true;
            }
        }
        
        // 检查注释绕过
        if sql.contains("--") || sql.contains("#") || sql.contains("/*") {
            // 更复杂的注释绕过检测
            let lower_sql = sql.to_lowercase();
            if lower_sql.contains("-- ") || lower_sql.contains("# ") {
                return true;
            }
        }
        
        false
    }
    
    pub fn detect_sensitive_data(&self, sql: &str) -> HashMap<String, Vec<String>> {
        let mut detected_data = HashMap::new();
        
        for (data_type, pattern) in &self.sensitive_data_patterns {
            let captures: Vec<String> = pattern.find_iter(sql)
                .map(|m| m.as_str().to_string())
                .collect();
            
            if !captures.is_empty() {
                detected_data.insert(data_type.clone(), captures);
            }
        }
        
        detected_data
    }
    
    pub fn get_permission_manager(&self) -> &PermissionManager {
        &self.permission_manager
    }
    
    pub fn get_permission_manager_mut(&mut self) -> &mut PermissionManager {
        &mut self.permission_manager
    }
    
    pub fn get_operation_type(&self, sql: &str) -> OperationType {
        let lower_sql = sql.to_lowercase().trim().to_string();
        
        if lower_sql.starts_with("select") {
            OperationType::SELECT
        } else if lower_sql.starts_with("insert") {
            OperationType::INSERT
        } else if lower_sql.starts_with("update") {
            OperationType::UPDATE
        } else if lower_sql.starts_with("delete") {
            OperationType::DELETE
        } else if lower_sql.starts_with("create") {
            OperationType::CREATE
        } else if lower_sql.starts_with("drop") {
            OperationType::DROP
        } else if lower_sql.starts_with("alter") {
            OperationType::ALTER
        } else if lower_sql.starts_with("grant") {
            OperationType::GRANT
        } else if lower_sql.starts_with("revoke") {
            OperationType::REVOKE
        } else if lower_sql.starts_with("admin") || lower_sql.starts_with("backup") || lower_sql.starts_with("restore") {
            OperationType::OTHER
        } else {
            OperationType::OTHER
        }
    }
}

pub trait SecurityRule {
    fn check(&self, sql: &str, user: &str, database_objects: &[SqlObject], engine: &SecurityRuleEngine) -> SecurityCheckResult;
}

struct SqlInjectionRule;

impl SecurityRule for SqlInjectionRule {
    fn check(&self, sql: &str, _user: &str, _database_objects: &[SqlObject], engine: &SecurityRuleEngine) -> SecurityCheckResult {
        let mut result = SecurityCheckResult::default();
        
        if engine.detect_sql_injection(sql) {
            result.errors.push(SecurityIssue {
                issue_type: SecurityIssueType::SqlInjection,
                severity: Severity::CRITICAL,
                message: "Potential SQL injection attack detected".to_string(),
                details: Some(format!("SQL contains patterns that suggest SQL injection: {}", sql)),
            });
            result.security_info.is_injection_risk = true;
            result.is_secure = false;
        }
        
        result
    }
}

struct SensitiveOperationRule;

impl SecurityRule for SensitiveOperationRule {
    fn check(&self, sql: &str, _user: &str, _database_objects: &[SqlObject], _engine: &SecurityRuleEngine) -> SecurityCheckResult {
        let mut result = SecurityCheckResult::default();
        
        let lower_sql = sql.to_lowercase();
        
        // 检查敏感操作
        if lower_sql.contains("drop table") || lower_sql.contains("truncate table") {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::SensitiveOperation,
                severity: Severity::HIGH,
                message: "Sensitive DDL operation detected".to_string(),
                details: Some("DROP TABLE or TRUNCATE TABLE operations can cause data loss".to_string()),
            });
            result.security_info.is_sensitive_operation = true;
        }
        
        if lower_sql.contains("grant") && lower_sql.contains("all privileges") {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::SensitiveOperation,
                severity: Severity::HIGH,
                message: "Overly permissive GRANT operation detected".to_string(),
                details: Some("GRANT ALL PRIVILEGES might grant excessive permissions".to_string()),
            });
            result.security_info.is_sensitive_operation = true;
        }
        
        if lower_sql.contains("update") && !lower_sql.contains("where") {
            result.errors.push(SecurityIssue {
                issue_type: SecurityIssueType::SensitiveOperation,
                severity: Severity::CRITICAL,
                message: "Unconditional UPDATE operation detected".to_string(),
                details: Some("UPDATE without WHERE clause will modify all rows".to_string()),
            });
            result.security_info.is_sensitive_operation = true;
            result.is_secure = false;
        }
        
        if lower_sql.contains("delete") && !lower_sql.contains("where") {
            result.errors.push(SecurityIssue {
                issue_type: SecurityIssueType::SensitiveOperation,
                severity: Severity::CRITICAL,
                message: "Unconditional DELETE operation detected".to_string(),
                details: Some("DELETE without WHERE clause will remove all rows".to_string()),
            });
            result.security_info.is_sensitive_operation = true;
            result.is_secure = false;
        }
        
        result
    }
}

struct SensitiveDataAccessRule;

impl SecurityRule for SensitiveDataAccessRule {
    fn check(&self, sql: &str, _user: &str, database_objects: &[SqlObject], engine: &SecurityRuleEngine) -> SecurityCheckResult {
        let mut result = SecurityCheckResult::default();
        
        // 检测敏感数据访问
        let sensitive_data = engine.detect_sensitive_data(sql);
        
        if !sensitive_data.is_empty() {
            for (data_type, values) in sensitive_data {
                result.warnings.push(SecurityIssue {
                    issue_type: SecurityIssueType::SensitiveDataAccess,
                    severity: Severity::MEDIUM,
                    message: format!("Sensitive data type '{}' detected in SQL", data_type),
                    details: Some(format!("Found {} occurrences of sensitive data", values.len())),
                });
                result.security_info.sensitive_data_accessed.extend(values);
                result.security_info.accesses_sensitive_data = true;
            }
        }
        
        // 检查对受保护对象的访问
        for obj in database_objects {
            // 检查是否访问敏感数据（简化实现）
            let obj_identifier = match (&obj.database, &obj.schema, &obj.table) {
                (Some(db), Some(schema), table) => format!("{}.{}.{}", db, schema, table),
                (Some(db), None, table) => format!("{}.{}", db, table),
                (None, Some(schema), table) => format!("{}.{}", schema, table),
                (None, None, table) => table.clone(),
            };
            
            // 简单检查敏感表名
            if obj_identifier.to_lowercase().contains("sensitive") || 
               obj_identifier.to_lowercase().contains("secret") ||
               obj_identifier.to_lowercase().contains("password") {
                result.errors.push(SecurityIssue {
                    issue_type: SecurityIssueType::SensitiveDataAccess,
                    severity: Severity::HIGH,
                    message: format!("Access to potentially sensitive object: {}", obj_identifier),
                    details: Some(format!("Object '{}' may contain sensitive data", obj_identifier)),
                });
                result.security_info.sensitive_data_accessed.extend([obj_identifier]);
                result.security_info.accesses_sensitive_data = true;
                result.is_secure = false;
            }
        }
        
        result
    }
}

struct PermissionCheckRule;

impl SecurityRule for PermissionCheckRule {
    fn check(&self, sql: &str, user: &str, database_objects: &[SqlObject], engine: &SecurityRuleEngine) -> SecurityCheckResult {
        let mut result = SecurityCheckResult::default();
        
        let operation_type = engine.get_operation_type(sql);
        
        // 检查对每个数据库对象的权限
        for obj in database_objects {
            // 构建对象标识符
            let obj_identifier = match (&obj.database, &obj.schema, &obj.table) {
                (Some(db), Some(schema), table) => format!("{}.{}.{}", db, schema, table),
                (Some(db), None, table) => format!("{}.{}", db, table),
                (None, Some(schema), table) => format!("{}.{}", schema, table),
                (None, None, table) => table.clone(),
            };
            
            // 确定对象类型
            let obj_type = if obj.column.is_some() {
                ObjectType::Column
            } else {
                ObjectType::Table
            };
            
            if !engine.permission_manager.check_permission(user, &obj_identifier, obj_type, operation_type.clone()) {
                result.errors.push(SecurityIssue {
                    issue_type: SecurityIssueType::PermissionViolation,
                    severity: Severity::HIGH,
                    message: format!("Permission denied for user '{}'", user),
                    details: Some(format!("User '{}' has no permission to perform {:?} operation on '{}'", user, operation_type, obj_identifier)),
                });
                result.security_info.has_permission_violation = true;
                result.is_secure = false;
            }
        }
        
        result
    }
}

struct ResourceLimitRule;

impl SecurityRule for ResourceLimitRule {
    fn check(&self, sql: &str, _user: &str, _database_objects: &[SqlObject], _engine: &SecurityRuleEngine) -> SecurityCheckResult {
        let mut result = SecurityCheckResult::default();
        
        // 检查SQL复杂度
        let sql_length = sql.len();
        let token_count = sql.split_whitespace().count();
        
        if sql_length > 10000 {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimit,
                severity: Severity::MEDIUM,
                message: "SQL statement too long".to_string(),
                details: Some(format!("SQL length: {} characters (limit: 10000)", sql_length)),
            });
            result.security_info.exceeds_resource_limits = true;
        }
        
        if token_count > 1000 {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimit,
                severity: Severity::MEDIUM,
                message: "SQL statement too complex".to_string(),
                details: Some(format!("Token count: {} (limit: 1000)", token_count)),
            });
            result.security_info.exceeds_resource_limits = true;
        }
        
        // 检查JOIN操作数量
        let join_count = sql.to_lowercase().matches("join").count();
        if join_count > 10 {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimit,
                severity: Severity::HIGH,
                message: "Too many JOIN operations".to_string(),
                details: Some(format!("JOIN count: {} (limit: 10)", join_count)),
            });
            result.security_info.exceeds_resource_limits = true;
        }
        
        // 检查子查询数量
        let subquery_count = sql.to_lowercase().matches("select").count() - 1; // 减去主查询
        if subquery_count > 5 {
            result.warnings.push(SecurityIssue {
                issue_type: SecurityIssueType::ResourceLimit,
                severity: Severity::HIGH,
                message: "Too many subqueries".to_string(),
                details: Some(format!("Subquery count: {} (limit: 5)", subquery_count)),
            });
            result.security_info.exceeds_resource_limits = true;
        }
        
        result
    }
}

pub struct PermissionManager {
    // 用户权限映射: (用户, 对象名, 对象类型) -> 操作集合
    user_permissions: HashMap<(String, String, ObjectType), HashSet<OperationType>>,
    
    // 角色权限映射: 角色 -> (对象名, 对象类型) -> 操作集合
    role_permissions: HashMap<String, HashMap<(String, ObjectType), HashSet<OperationType>>>,
    
    // 用户角色映射: 用户 -> 角色集合
    user_roles: HashMap<String, HashSet<String>>,
    
    // 默认权限映射: 对象类型 -> 操作集合
    default_permissions: HashMap<ObjectType, HashSet<OperationType>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            user_permissions: HashMap::new(),
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
            default_permissions: HashMap::new(),
        }
    }
    
    pub fn grant_permission(&mut self, user: &str, object_name: &str, object_type: ObjectType, operation: OperationType) {
        let key = (user.to_string(), object_name.to_string(), object_type);
        self.user_permissions.entry(key)
            .or_insert_with(HashSet::new)
            .insert(operation);
    }
    
    pub fn revoke_permission(&mut self, user: &str, object_name: &str, object_type: ObjectType, operation: OperationType) {
        let key = (user.to_string(), object_name.to_string(), object_type);
        if let Some(operations) = self.user_permissions.get_mut(&key) {
            operations.remove(&operation);
            if operations.is_empty() {
                self.user_permissions.remove(&key);
            }
        }
    }
    
    pub fn check_permission(&self, user: &str, object_name: &str, object_type: ObjectType, operation: OperationType) -> bool {
        // 1. 检查用户直接权限
        let direct_key = (user.to_string(), object_name.to_string(), object_type.clone());
        if let Some(operations) = self.user_permissions.get(&direct_key) {
            if operations.contains(&operation) {
                return true;
            }
        }
        
        // 2. 检查用户角色权限
        if let Some(roles) = self.user_roles.get(user) {
            for role in roles {
                if let Some(role_perms) = self.role_permissions.get(role) {
                    let role_key = (object_name.to_string(), object_type.clone());
                    if let Some(operations) = role_perms.get(&role_key) {
                        if operations.contains(&operation) {
                            return true;
                        }
                    }
                }
            }
        }
        
        // 3. 检查默认权限
        if let Some(operations) = self.default_permissions.get(&object_type) {
            if operations.contains(&operation) {
                return true;
            }
        }
        
        false
    }
    
    pub fn create_role(&mut self, role_name: &str) {
        self.role_permissions.entry(role_name.to_string())
            .or_insert_with(HashMap::new);
    }
    
    pub fn grant_role_permission(&mut self, role_name: &str, object_name: &str, object_type: ObjectType, operation: OperationType) {
        let role_entry = self.role_permissions.entry(role_name.to_string())
            .or_insert_with(HashMap::new);
        
        let key = (object_name.to_string(), object_type);
        role_entry.entry(key)
            .or_insert_with(HashSet::new)
            .insert(operation);
    }
    
    pub fn assign_role(&mut self, user: &str, role_name: &str) {
        self.user_roles.entry(user.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_name.to_string());
    }
    
    pub fn set_default_permissions(&mut self, object_type: ObjectType, operations: HashSet<OperationType>) {
        self.default_permissions.insert(object_type, operations);
    }
}

pub struct SecurityCheckResult {
    pub warnings: Vec<SecurityIssue>,
    pub errors: Vec<SecurityIssue>,
    pub security_info: SecurityInfo,
    pub is_secure: bool,
}

impl Default for SecurityCheckResult {
    fn default() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
            security_info: SecurityInfo::default(),
            is_secure: true,
        }
    }
}

pub struct SecurityIssue {
    pub issue_type: SecurityIssueType,
    pub severity: Severity,
    pub message: String,
    pub details: Option<String>,
}

pub enum SecurityIssueType {
    SqlInjection,
    SensitiveOperation,
    SensitiveDataAccess,
    ProtectedObjectAccess,
    PermissionViolation,
    ResourceLimit,
    Other,
}

pub enum Severity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

#[derive(Default)]
pub struct SecurityInfo {
    pub is_injection_risk: bool,
    pub is_sensitive_operation: bool,
    pub accesses_sensitive_data: bool,
    pub has_permission_violation: bool,
    pub exceeds_resource_limits: bool,
    pub sensitive_data_accessed: HashSet<String>,
    pub protected_objects_accessed: Vec<SqlObject>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ObjectType, OperationType};
    
    #[test]
    fn test_sql_injection_detection() {
        let engine = SecurityRuleEngine::new();
        
        // 测试SQL注入检测
        assert!(engine.detect_sql_injection("SELECT * FROM users WHERE username = 'admin' --") || 
                engine.detect_sql_injection("SELECT * FROM users WHERE username = 'admin' #"));
        assert!(engine.detect_sql_injection("SELECT * FROM users WHERE 1=1 OR 1=1"));
        assert!(engine.detect_sql_injection("SELECT * FROM users UNION SELECT password FROM admin_users"));
        
        // 测试非注入SQL
        assert!(!engine.detect_sql_injection("SELECT * FROM users WHERE username = 'john'"));
        assert!(!engine.detect_sql_injection("SELECT id, name FROM products WHERE price > 100"));
    }
    
    #[test]
    fn test_permission_checking() {
        let mut manager = PermissionManager::new();
        
        // 授予权限
        manager.grant_permission("user1", "products", ObjectType::TABLE, OperationType::READ);
        
        // 检查权限
        assert!(manager.check_permission("user1", "products", ObjectType::TABLE, OperationType::READ));
        assert!(!manager.check_permission("user1", "products", ObjectType::TABLE, OperationType::WRITE));
        assert!(!manager.check_permission("user2", "products", ObjectType::TABLE, OperationType::READ));
    }
    
    #[test]
    fn test_role_based_permissions() {
        let mut manager = PermissionManager::new();
        
        // 创建角色并分配权限
        manager.create_role("reader");
        manager.grant_role_permission("reader", "users", ObjectType::TABLE, OperationType::READ);
        
        // 将角色分配给用户
        manager.assign_role("user1", "reader");
        
        // 检查权限
        assert!(manager.check_permission("user1", "users", ObjectType::TABLE, OperationType::READ));
    }
}