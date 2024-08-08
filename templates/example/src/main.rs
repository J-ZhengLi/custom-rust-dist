//! 一个 cargo 示例项目。
//! 您也可以随时运行 “cargo init <name>” 来创建新项目.

/// 用于计算 [斐波那契数列](https://en.wikipedia.org/wiki/Fibonacci_sequence) 的第 n 个索引。
/// 
/// ## 用法
/// 
/// ### 使用 CodeLLDB
/// 
/// 参考：CodeLLDB [用户界面](https://code.visualstudio.com/docs/editor/debugging#_user-interface)
/// 
/// - 单击左上方 `Debug executable` 按钮, 或单击主函数顶部的 `运行` 按钮. 
/// 
/// ### 或使用常规方式
/// 1. 在 VS code 中打开一个新的集成终端。 [教程](https://code.visualstudio.com/docs/terminal/basics)
/// 2. 输入以下命令 `cargo run`
fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let input = 10;
    println!("fibonacchi({}) = {}", input, fibonacci(input));
}
