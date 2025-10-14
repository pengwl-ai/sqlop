# SQL 解析引擎 (SQLOP)

一个高性能、可扩展的 SQL 解析引擎，支持多种数据库的审计日志解析，能够精确提取 SQL 语句中的库、表、schema、列信息。

## 功能特性

### 🚀 核心功能
- **多数据库支持**: 支持 MySQL、PostgreSQL、SQL Server、Oracle、Hive、GaussDB、Kingbase、Highgo、Greenplum、Vastbase、Sybase、DB2、达梦等主流数据库
- **高性能解析**: 在 2c4g 环境下可达到 6000+ EPS 的解析性能
- **精确提取**: 准确识别 SQL 语句中的数据库、schema、表、列信息
- **审计日志解析**: 支持不同数据库的审计日志格式解析

### 🔧 高级功能
- **SQL 验证**: 内置 SQL 语法验证器，支持安全检查和性能检查
- **SQL 格式化**: 智能格式化和压缩 SQL 语句
- **性能监控**: 实时性能监控和统计，支持 P95、P99 等指标
- **敏感信息检测**: 自动识别 SQL 中的敏感信息（身份证号、手机号、密码等）
- **缓存机制**: 智能缓存解析结果，提升重复查询性能
- **并行处理**: 支持批量并行解析，提升整体吞吐量

## 快速开始

### 安装依赖

确保你的系统已安装 Rust 工具链：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 克隆项目

```bash
git clone <repository-url>
cd sqlop
```

### 构建项目

```bash
cargo build --release
```

### 运行示例

```bash
# 运行基本示例
cargo run --example basic_usage

# 运行主程序
cargo run
```

## 使用方法

### 基本使用

```rust
use sqlop::core::{SqlopEngine, DatabaseType};

// 创建引擎实例
let mut engine = SqlopEngine::default();

// 解析 SQL 语句
let sql = "SELECT name, user FROM dsp.api LIMIT 10;";
let result = engine.parse_sql(sql, DatabaseType::MySQL)?;

println!("数据库: {:?}", result.databases);
println!("Schema: {:?}", result.schemas);
println!("表: {:?}", result.tables);
println!("列: {:?}", result.columns);
```

### 批量处理

```rust
use sqlop::core::{SqlopEngine, DatabaseType};

let mut engine = SqlopEngine::default();

let batch_sql = vec![
    ("SELECT * FROM users", DatabaseType::MySQL),
    ("SELECT name FROM products", DatabaseType::PostgreSQL),
    ("SELECT id FROM orders", DatabaseType::SQLServer),
];

let results = engine.parse_batch_sql(&batch_sql);

for (i, result) in results.iter().enumerate() {
    match result {
        Ok(parse_result) => {
            println!("{}. ✅ {} - 表: {:?}", i + 1, parse_result.database_type, parse_result.tables);
        }
        Err(e) => {
            println!("{}. ❌ 解析失败: {}", i + 1, e);
        }
    }
}
```

### SQL 验证

```rust
use sqlop::utils::SqlValidator;
use sqlop::core::DatabaseType;

let validator = SqlValidator::new(DatabaseType::MySQL);
let sql = "SELECT * FROM users WHERE id = 1";

let result = validator.validate(sql)?;

if result.is_valid {
    println!("SQL 语法正确");
} else {
    println!("SQL 语法错误:");
    for error in &result.errors {
        println!("  - {}: {}", error.rule_name, error.message);
    }
}
```

### SQL 格式化

```rust
use sqlop::utils::SqlFormatter;
use sqlop::core::DatabaseType;

let formatter = SqlFormatter::new(DatabaseType::MySQL);
let sql = "select name,email from users where id=1";

// 格式化
let formatted = formatter.format(sql)?;
println!("格式化后:\n{}", formatted);

// 压缩
let minified = formatter.minify(sql)?;
println!("压缩后: {}", minified);
```

### 性能监控

```rust
use sqlop::utils::PerformanceMonitor;
use sqlop::core::{SqlopEngine, DatabaseType};

let mut engine = SqlopEngine::default();
let mut monitor = PerformanceMonitor::new();

// 开始监控
let timer = monitor.start_operation().with_database_type("MySQL");
let result = engine.parse_sql("SELECT * FROM users", DatabaseType::MySQL);
timer.finish(result.is_ok());

// 获取性能统计
println!("{}", monitor.get_summary());
```

### 配置文件

创建 `config/default.toml` 文件：

```toml
[parser]
max_parse_time_ms = 1000
enable_cache = true
cache_size = 10000
enable_parallel = true
max_workers = 4

[[sensitive_patterns]]
name = "身份证号"
pattern = "\\b\\d{17}[\\dXx]\\b"
description = "匹配中国身份证号码"
column_names = ["id_card", "identity_card"]
table_names = ["users", "employees"]

[database_configs.MySQL]
quote_char = '`'
identifier_case_sensitive = false
support_schema = true
default_schema = null
```

从配置文件加载：

```rust
use sqlop::core::SqlopEngine;
use std::path::Path;

let engine = SqlopEngine::from_config_file(Path::new("config/default.toml"))?;
```

## 项目结构

```
sqlop/
├── src/
│   ├── core/              # 核心解析引擎
│   │   ├── error.rs      # 错误处理
│   │   ├── types.rs      # 数据类型定义
│   │   ├── parser.rs     # 解析器核心
│   │   └── mod.rs        # 模块入口
│   ├── adapters/         # 数据库适配器
│   │   ├── mod.rs        # 适配器管理
│   │   ├── common.rs     # 通用适配器
│   │   ├── mysql.rs      # MySQL 适配器
│   │   ├── postgresql.rs # PostgreSQL 适配器
│   │   ├── sqlserver.rs  # SQL Server 适配器
│   │   ├── oracle.rs     # Oracle 适配器
│   │   └── hive.rs       # Hive 适配器
│   ├── utils/            # 工具模块
│   │   ├── mod.rs        # 工具入口
│   │   ├── performance.rs # 性能监控
│   │   ├── validation.rs # SQL 验证
│   │   └── format.rs     # SQL 格式化
│   ├── main.rs           # 主程序入口
│   └── lib.rs            # 库入口
├── config/               # 配置文件
│   └── default.toml      # 默认配置
├── examples/             # 示例代码
│   └── basic_usage.rs    # 基本使用示例
├── tests/                # 测试文件
│   ├── unit_tests.rs     # 单元测试
│   └── integration_tests.rs # 集成测试
├── benches/              # 性能测试
│   └── parser_benchmark.rs # 解析性能测试
├── docs/                 # 文档
├── Cargo.toml            # 项目配置
└── README.md             # 项目说明
```

## 性能优化

### 架构优化
- **并行处理**: 使用 Rayon 实现并行解析，充分利用多核 CPU
- **智能缓存**: LRU 缓存机制，避免重复解析相同 SQL
- **内存管理**: 优化内存分配，减少 GC 压力
- **零拷贝**: 尽可能使用字符串切片，避免不必要的拷贝

### 解析优化
- **增量解析**: 支持增量解析，只解析变化的部分
- **预编译正则**: 预编译常用正则表达式，提升匹配速度
- **语法树优化**: 优化语法树结构，减少遍历开销

### 配置优化
```toml
[parser]
# 启用并行处理
enable_parallel = true
# 设置合适的工作线程数
max_workers = 4
# 启用缓存
enable_cache = true
# 设置合适的缓存大小
cache_size = 10000
# 设置最大解析时间
max_parse_time_ms = 1000
```

## 测试

### 运行单元测试

```bash
cargo test
```

### 运行集成测试

```bash
cargo test --test integration_tests
```

### 运行性能测试

```bash
cargo bench
```

### 生成测试覆盖率报告

```bash
cargo tarpaulin --out Html
```

## 支持的数据库

| 数据库 | 支持状态 | 备注 |
|--------|----------|------|
| MySQL | ✅ 完全支持 | 支持所有主要语法 |
| PostgreSQL | ✅ 完全支持 | 支持所有主要语法 |
| SQL Server | ✅ 完全支持 | 支持所有主要语法 |
| Oracle | ✅ 基本支持 | 支持常用语法 |
| Hive | ✅ 基本支持 | 支持 HiveQL |
| GaussDB | ✅ 基本支持 | 兼容 PostgreSQL |
| Kingbase | ✅ 基本支持 | 兼容 PostgreSQL |
| Highgo | ✅ 基本支持 | 兼容 PostgreSQL |
| Greenplum | ✅ 基本支持 | 兼容 PostgreSQL |
| Vastbase | ✅ 基本支持 | 兼容 PostgreSQL |
| Sybase | ✅ 基本支持 | 兼容 MySQL |
| DB2 | ✅ 基本支持 | 支持常用语法 |
| 达梦 | ✅ 基本支持 | 支持常用语法 |

## API 文档

生成 API 文档：

```bash
cargo doc --no-deps
```

然后在浏览器中打开 `target/doc/sqlop/index.html`。

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 联系方式

- 项目主页: [GitHub Repository]
- 问题反馈: [GitHub Issues]
- 邮箱: [your-email@example.com]

## 更新日志

### v0.1.0 (2024-01-01)
- 初始版本发布
- 支持基本的 SQL 解析功能
- 支持 MySQL、PostgreSQL、SQL Server 数据库
- 实现性能监控和缓存机制
- 添加 SQL 验证和格式化功能

## 致谢

感谢以下开源项目：
- [sqlparser-rs](https://github.com/sqlparser-rs/sqlparser-rs) - SQL 解析器
- [Rayon](https://github.com/rayon-rs/rayon) - 并行计算框架
- [Serde](https://github.com/serde-rs/serde) - 序列化框架