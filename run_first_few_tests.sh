#!/bin/bash

# 编译程序
cargo build --release --bin xlsx_test_runner

# 运行程序并只显示前10个测试用例的结果
target/release/xlsx_test_runner tests/安恒词法解析（复杂查询）\ \(1\).xlsx 2>&1 | head -n 1000