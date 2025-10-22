#!/bin/bash

# SQL解析测试脚本 - 限制2核4G资源

echo "=== 开始运行SQL解析测试（限制2核4G资源）==="

# 进入项目根目录
cd "$(dirname "$0")"

# 设置资源限制：2个CPU核心，4GB内存
export RUSTFLAGS="-C debuginfo=0 -C codegen-units=2"
export CARGO_BUILD_JOBS=2

# 在macOS上使用taskpolicy限制CPU亲和性（如果可用）
if command -v taskpolicy &> /dev/null; then
    echo "使用taskpolicy限制CPU资源..."
    taskpolicy -c background -p $$
fi

# 运行测试程序，使用引号包围文件路径以处理空格和特殊字符
echo "正在编译并运行测试..."
# cargo run --jobs 2 --bin excel_sql_tester \
#   "tests/安恒词法解析（复杂查询） (1).xlsx" \
#   "tests/DSP解析与策略能力列表 (1).xlsx"

cargo run --jobs 2 --bin excel_sql_tester \
  "tests/sql_parse_test_set(1).xlsx"

# 检查执行结果
if [ $? -eq 0 ]; then
  echo "\n=== 测试成功完成！==="
  echo "结果已写入 result.txt 文件"
  
  # 单独提取并显示EPS结果
   echo "\n=== EPS 性能测试结果 ==="
   # 使用更简单的命令提取EPS值
   EPS_LINE=$(grep 'EPS:' result.txt)
   EPS_VALUE=$(echo $EPS_LINE | awk '{print $3}')
   echo "EPS 值: $EPS_VALUE"
   echo "性能要求: 目标6000 EPS"
   
   # 检查是否满足性能要求
   if [ ! -z "$EPS_VALUE" ] && (( $(echo "$EPS_VALUE >= 6000" | bc -l) )); then
     echo "✅ 性能满足要求！"
   else
     echo "❌ 性能未满足要求！"
     # 直接输出文件中的EPS行
     echo "实际性能结果: $EPS_LINE"
   fi
  
  echo "\n查看结果文件的部分内容："
  head -n 10 result.txt
  echo "..."
  tail -n 15 result.txt
else
  echo "\n=== 测试执行失败！==="
  exit 1
fi