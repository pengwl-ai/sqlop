# SQL 解析引擎项目结构说明

本文件详细介绍了SQL解析引擎项目的目录结构和各个文件的主要功能，帮助开发者快速了解项目组织和代码分布。

## 项目根目录结构

```
sqlop/
├── Cargo.lock          # 依赖版本锁定文件
├── Cargo.toml          # 项目配置和依赖声明
├── README.md           # 项目主文档
├── config/             # 配置文件目录
├── docs/               # 文档目录
├── run_first_few_tests.sh # 快速运行测试脚本
├── src/                # 源代码目录
├── target/             # 编译输出目录
├── test_results.txt    # 测试结果记录文件
└── tests/              # 测试文件目录
```

## 主要目录说明

### 1. src/ - 源代码目录

源代码目录是项目的核心，包含所有Rust源代码。根据功能模块进行了清晰的划分：

```
src/
├── adapters/           # 数据库适配器实现
├── bin/                # 可执行二进制文件
├── core/               # 核心功能实现
├── examples/           # 使用示例代码
├── lib.rs              # 库入口文件
├── main.rs             # 主程序入口
└── utils/              # 通用工具函数
```

#### adapters/ 目录

该目录包含了针对不同数据库类型的适配器实现，用于处理各数据库的特定语法和特性：

```
adapters/
├── common.rs           # 适配器通用功能和接口
├── dameng.rs           # 达梦数据库适配器
├── db2.rs              # DB2数据库适配器
├── dialects/           # 数据库方言定义
├── gaussdb.rs          # GaussDB数据库适配器
├── highgo.rs           # HighGo数据库适配器
├── hive.rs             # Hive数据库适配器
├── kingbase.rs         # Kingbase数据库适配器
├── mod.rs              # 适配器模块定义
├── mysql.rs            # MySQL数据库适配器
├── oracle.rs           # Oracle数据库适配器
├── postgresql.rs       # PostgreSQL数据库适配器
├── sqlite.rs           # SQLite数据库适配器
├── sqlserver.rs        # SQL Server数据库适配器
└── sybase.rs           # Sybase数据库适配器
```

#### bin/ 目录

该目录包含了项目的可执行二进制文件，主要用于测试和运行SQL解析引擎：

```
bin/
├── run_tests.rs        # 测试运行器，支持多种测试命令
└── excel_sql_tester.rs # Excel测试用例运行器，支持从Excel文件读取SQL测试用例并完整显示结果
```

**excel_sql_tester.rs** 是一个功能强大的测试工具，可以：
- 从Excel文件读取SQL测试用例
- 支持多种数据库类型的SQL语句测试
- 完整显示SQL语句内容，不进行截断
- 输出详细的解析结果，包括库、表、schema、列信息
- 提供性能统计信息，如解析率、EPS等

#### core/ 目录

该目录包含了SQL解析引擎的核心功能实现：

```
core/
├── ast_visitor.rs               # AST（抽象语法树）访问器实现
├── database_specific.rs         # 数据库特定功能处理
├── enhanced_parser.rs           # 增强的SQL解析器（早期版本）
├── enhanced_parser_improved_optimized.rs # 优化的增强解析器，提供关键字过滤功能
├── error.rs                     # 错误定义和处理
├── mod.rs                       # 核心模块定义
├── parser.rs                    # SQL解析器实现
├── piped_sql.rs                 # Piped SQL处理
├── sql_transpiler.rs            # SQL转译器
├── types.rs                     # 核心数据类型定义
└── utils/                       # 核心工具函数
```

**enhanced_parser_improved_optimized.rs** 是引擎的核心组件之一，主要功能：
- 实现全面的SQL关键字过滤算法
- 准确识别表名、列名、库名和模式名
- 支持多语言标识符（包括中文）处理
- 提供高性能的标识符验证功能
- 通过多重过滤策略提高解析准确性

该组件是解决SQL关键字错误识别问题的关键，性能测试显示可达到每秒处理39万条SQL语句的能力。

#### examples/ 目录

该目录包含了SQL解析引擎的使用示例代码：

```
examples/
├── basic_usage.rs      # 基本使用示例
└── multi_database_test.rs # 多数据库支持测试示例
```

#### utils/ 目录

该目录包含了通用的工具函数：

```
utils/
├── format.rs           # SQL格式化工具
├── mod.rs              # 工具模块定义
├── performance.rs      # 性能监控工具
└── validation.rs       # SQL验证工具
```

### 2. docs/ - 文档目录

该目录包含项目的详细文档：

```
docs/
├── 14_database_expansion_plan.md # 14种数据库扩充计划
├── design_summary.md     # 项目设计概要
├── improvement_plan.md   # 项目改进计划
└── project_structure.md  # 项目结构说明（本文件）
```

### 3. tests/ - 测试文件目录

该目录包含项目的测试文件：

```
tests/
├── DSP解析与策略能力列表 (1).xlsx # DSP解析测试用例
├── benchmarks/          # 性能基准测试
│   └── parser_benchmark.rs # SQL解析性能测试
└── 安恒词法解析（复杂查询） (1).xlsx # 复杂SQL查询测试用例
```

### 4. config/ - 配置文件目录

该目录包含项目的配置文件：

```
config/
└── default.toml        # 默认配置文件
```

## 重要文件说明

### Cargo.toml

项目的配置文件，定义了项目的元数据、依赖关系、特性等。通过该文件，Rust的包管理器Cargo可以管理项目的构建和依赖。

### README.md

项目的主文档，包含项目的概述、功能特性、快速开始指南等信息，是新用户了解项目的入口。

### run_first_few_tests.sh

一个便捷的测试脚本，用于快速编译和运行部分测试用例，命令如下：

```bash
# 编译程序
cargo build --release --bin xlsx_test_runner

# 运行程序并只显示前10个测试用例的结果
target/release/xlsx_test_runner tests/安恒词法解析（复杂查询）\ \(1\).xlsx 2>&1 | head -n 1000
```

### src/lib.rs

项目的库入口文件，定义了库的公共API和导出的模块。

### src/main.rs

项目的主程序入口，负责启动和运行SQL解析引擎。

### src/bin/xlsx_test_runner.rs

Excel测试用例运行器，支持从Excel文件读取SQL测试用例并执行测试。该文件是项目的主要测试工具之一。

## 代码组织原则

1. **模块化设计**：项目采用模块化设计，将不同功能的代码组织到不同的模块中，提高代码的可读性和可维护性。

2. **适配器模式**：通过适配器模式支持多种数据库类型，每种数据库有自己的适配器实现，统一接口但有特定的处理逻辑。

3. **清晰的责任分离**：核心逻辑、数据库特定功能、工具函数等都有明确的责任划分，避免代码耦合。

4. **测试驱动开发**：项目包含丰富的测试用例，支持从Excel文件读取测试用例，便于测试和验证功能的正确性。

5. **性能优化**：包含性能监控和优化功能，确保SQL解析引擎在各种场景下都能高效运行。

## 使用指南

### 编译项目

```bash
cargo build --release
```

### 运行测试

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration_tests

# 运行性能测试
cargo test --test performance/eps_test

# 使用Excel测试运行器
cargo run --bin excel_sql_tester tests/安恒词法解析（复杂查询）\ \(1\).xlsx
```

### 运行示例

```bash
# 运行基本使用示例
cargo run --example basic_usage

# 运行多数据库支持测试示例
cargo run --example multi_database_test
```

通过本文件，您应该已经对SQL解析引擎项目的结构有了全面的了解。如需进一步了解具体模块的实现细节，请参考相应的源代码和其他文档。