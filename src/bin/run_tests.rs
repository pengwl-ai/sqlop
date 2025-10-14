use std::env;
use std::process::Command;

fn main() {
    // 获取命令行参数
    let args: Vec<String> = env::args().collect();
    
    // 如果没有参数，显示帮助信息
    if args.len() <= 1 {
        show_help();
        return;
    }
    
    // 根据参数运行不同的测试
    match args[1].as_str() {
        "all" => run_all_tests(),
        "unit" => run_unit_tests(),
        "integration" => run_integration_tests(),
        "performance" => run_performance_tests(),
        "multi-database" => run_multi_database_tests(),
        "benchmarks" => run_benchmarks(),
        "examples" => run_examples(),
        "eps" => run_eps_test(),
        "update-structure" => run_update_structure_test(),
        "help" | "-h" | "--help" => show_help(),
        _ => {
            println!("未知命令: {}", args[1]);
            show_help();
        }
    }
}

fn show_help() {
    println!("SQL解析引擎测试运行器");
    println!("==================");
    println!("用法: cargo run --release -- <命令>");
    println!("");
    println!("可用命令:");
    println!("  all               - 运行所有测试");
    println!("  unit              - 运行单元测试");
    println!("  integration       - 运行集成测试");
    println!("  performance       - 运行所有性能测试");
    println!("  multi-database    - 运行多数据库支持测试");
    println!("  benchmarks        - 运行基准测试");
    println!("  examples          - 运行示例程序");
    println!("  eps               - 运行EPS性能测试");
    println!("  update-structure  - 运行UPDATE语句结构测试");
    println!("  help, -h, --help  - 显示此帮助信息");
}

fn run_command(command: &str, args: &[&str]) {
    println!("\n运行: {} {}", command, args.join(" "));
    let status = Command::new(command)
        .args(args)
        .status()
        .expect("命令执行失败");
    
    if !status.success() {
        println!("命令执行失败: {}", status);
    }
}

fn run_all_tests() {
    println!("运行所有测试...");
    run_command("cargo", &["test", "--release"]);
}

fn run_unit_tests() {
    println!("运行单元测试...");
    run_command("cargo", &["test", "--release", "--test", "unit_tests"]);
}

fn run_integration_tests() {
    println!("运行集成测试...");
    run_command("cargo", &["test", "--release", "--test", "integration_tests"]);
}

fn run_performance_tests() {
    println!("运行性能测试...");
    run_eps_test();
}

fn run_multi_database_tests() {
    println!("运行多数据库支持测试...");
    run_command("cargo", &[
        "run", "--release",
        "--features", "mysql postgresql sqlserver oracle hive gauss kingbase highgo greenplum vastbase sybase db2 dameng",
        "--bin", "multi_database_test"
    ]);
}

fn run_benchmarks() {
    println!("运行基准测试...");
    run_command("cargo", &["bench"]);
}

fn run_examples() {
    println!("运行示例程序...");
    run_command("cargo", &["run", "--release", "--bin", "basic_usage_example"]);
    run_command("cargo", &[
        "run", "--release",
        "--features", "mysql postgresql sqlserver oracle hive gauss kingbase highgo greenplum vastbase sybase db2 dameng",
        "--bin", "multi_database_test_example"
    ]);
}

fn run_eps_test() {
    println!("运行EPS性能测试...");
    // 直接运行注册的eps_test二进制目标
    run_command("cargo", &["run", "--release", "--bin", "eps_test"]);
}

fn run_update_structure_test() {
    println!("运行UPDATE语句结构测试...");
    // 直接运行注册的update_structure_test二进制目标
    run_command("cargo", &["run", "--release", "--bin", "update_structure_test"]);
}