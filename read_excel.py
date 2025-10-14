import pandas as pd

# 设置中文字体支持
pd.set_option('display.unicode.ambiguous_as_wide', True)
pd.set_option('display.unicode.east_asian_width', True)

# 读取Excel文件
file_path = '/Volumes/Macintosh HD/Users/zhushuai/rust/src/sqlop/tests/安恒词法解析（复杂查询） (1).xlsx'

try:
    # 获取所有sheet名称
    excel_file = pd.ExcelFile(file_path)
    sheet_names = excel_file.sheet_names
    
    print(f"Excel文件包含 {len(sheet_names)} 个工作表:")
    for i, sheet_name in enumerate(sheet_names):
        print(f"{i+1}. {sheet_name}")
    
    print("\n" + "="*50 + "\n")
    
    # 读取每个sheet的前几行数据
    for sheet_name in sheet_names:
        print(f"工作表: {sheet_name}")
        
        # 读取前10行数据
        df = pd.read_excel(file_path, sheet_name=sheet_name, nrows=10)
        
        # 打印列名
        print("列名:")
        for col in df.columns:
            print(f"- {col}")
        
        # 检查是否包含 'OprCmd' 和 'OprObject' 列（注意大小写）
        if 'OprCmd' in df.columns and 'OprObject' in df.columns:
            print("\n找到关键列 OprCmd 和 OprObject，显示前5行数据:")
            display_df = df[['OprCmd', 'OprObject']].head(5)
            print(display_df)
            
            # 保存关键信息到文本文件以便查看
            with open('excel_content_summary.txt', 'w', encoding='utf-8') as f:
                f.write(f"Excel文件包含 {len(sheet_names)} 个工作表:\n")
                for i, sheet_name_saved in enumerate(sheet_names):
                    f.write(f"{i+1}. {sheet_name_saved}\n")
                
                f.write(f"\n\n工作表: {sheet_name}\n")
                f.write("关键列数据示例:\n")
                for _, row in df[['OprCmd', 'OprObject']].head(5).iterrows():
                    f.write(f"SQL: {row['OprCmd']}\n")
                    f.write(f"解析结果: {row['OprObject']}\n")
                    f.write("-"*50 + "\n")
        else:
            # 如果没有找到这两列，显示前3行所有数据
            print("\n未找到关键列 OprCmd 和 OprObject，显示前3行所有数据:")
            print(df.head(3))
        
        print("\n" + "="*50 + "\n")
        
    print("已将Excel文件内容摘要保存到 excel_content_summary.txt")
    
except Exception as e:
    print(f"读取Excel文件时出错: {e}")