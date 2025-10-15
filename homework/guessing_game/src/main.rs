// ===================================================================
// ========================= 改进后的代码 =========================
// ===================================================================

use rand::Rng;
use std::cmp::Ordering;
use std::io;

// --- 常量定义 ---
// 将“魔法数字”定义为常量，方便统一管理和修改
const GAME_ROUNDS: u32 = 2; // 总共进行的游戏轮数
const SECRET_NUMBER_MIN: u32 = 1; // 秘密数字的最小值
const SECRET_NUMBER_MAX: u32 = 100; // 秘密数字的最大值

/// 运行一轮猜数字游戏，并返回本轮的猜测次数
fn play_round(round_number: u32) -> u32 {
    println!("\n\n--- 第 {}/{} 轮游戏开始 ---", round_number, GAME_ROUNDS);

    // 生成一个在 [MIN..=MAX] 范围内的随机数
    let secret_number = rand::thread_rng().gen_range(SECRET_NUMBER_MIN..=SECRET_NUMBER_MAX);
    let mut guess_count = 0;

    loop {
        println!("\t请输入你猜的数字 ({} - {}):", SECRET_NUMBER_MIN, SECRET_NUMBER_MAX);

        let mut guess_str = String::new();
        io::stdin()
            .read_line(&mut guess_str)
            .expect("读取行失败");

        // 解析用户输入
        let guess: u32 = match guess_str.trim().parse() {
            Ok(num) => {
                guess_count += 1;
                num
            }
            Err(_) => {
                // 当用户输入无效时，给予提示
                println!("\t无效输入！请输入一个数字。");
                continue;
            }
        };

        // 比较猜测的数字和秘密数字
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("\t太小了!"),
            Ordering::Greater => println!("\t太大了!"),
            Ordering::Equal => {
                println!("\n\t恭喜你，猜对了! 🎉");
                println!("\t本轮你一共猜了 {} 次。", guess_count);
                return guess_count; // 返回猜测次数并结束本轮
            }
        }
    }
}

fn main() {
    println!("\n\n欢迎来到猜数字游戏!");

    let mut best_score = u32::MAX; // 使用u32的最大值来初始化最佳成绩

    // 循环进行指定轮数的游戏
    for i in 1..=GAME_ROUNDS {
        let current_score = play_round(i);
        if current_score < best_score {
            best_score = current_score;
        }
    }

    // 游戏结束后，只打印一次最终总结
    println!("\n\n🎉🎉🎉 游戏结束! 🎉🎉🎉");
    if best_score == u32::MAX {
        println!("你没有完成任何一轮游戏。");
    } else {
        println!("在所有 {} 轮游戏中，你的最佳成绩是: {} 次!", GAME_ROUNDS, best_score);
    }
    println!("欢迎下次再来!\n");
}


// ===================================================================
// ========================= 你的原始代码 =========================
// ===================================================================

/*
use std::cmp::Ordering;
use std::io;           //将 io 输入/输出库引入当前作用域。io 库来自于标准库，也被称为 std：
use rand::Rng;         //将 Rng trait 引入当前作用域。Rng trait 定义了随机数生成器所需的方法


fn main() {
    let game_times = 2;
    println!("\n\n <Guess the number for {game_times} times!>");
    
    let mut count_less = 100; 
    
    for _element in 1..=game_times {
        
        let mut count = 0;
        
        println!("\n\n This is the {_element}th time you guess the number! \n");

        let secret_number = rand::thread_rng().gen_range(1..=100);
        // 这里使用的这类范围表达式使用了 start..=end 这样的形式，它对上下边界均为闭区间，所以需要指定 1..=100 来请求一个 1 和 100 之间的数。
        
        loop {
            println!("\t Please input your guess:");

            let mut guess = String::new(); 
            // let mut bananas = 5;  可变
            // String::new 的结果，这个函数会返回一个 String 的新实例，String 是一个标准库提供的字符串类型，它是 UTF-8 编码的可增长文本块。
                // :: 语法表明 new 是 String 类型的一个 关联函数

            io::stdin()
            // 获取标准输入（键盘输入） 如果程序的开头没有使用 use std::io; 引入 io 库，我们仍可以通过把函数调用写成 std::io::stdin 来使用该函数
                .read_line(&mut guess)
                // 这个函数会把用户输入的内容放到我们传入的字符串中// & 表示引用，这样我们就不会取得 guess 的所有权// mut 关键字表示这个引用是可变的
                .expect("Failed to read line");

            // let guess: u32 = guess.trim().parse().expect("Please type a number!");
            // // Rust 允许用一个新值来 遮蔽 （Shadowing） guess 之前的值。这个功能允许我们复用 guess 变量的名字，而不是被迫创建两个不同变量
            // // 这里使用了 trim 方法来去掉输入字符串的前后空白，然后调用 parse 方法将其转换为 u32 类型
            let guess: u32 = match guess.trim().parse() {
                Ok(num) => {
                    count += 1;
                    num
                },
                Err(_) => continue,
                // 如果解析失败，忽略错误（_ 表示不关心具体错误）
            };

            // println!("You guessed: {guess}");
            // println!("x = {x} and y + 2 = {}", y + 2);
            // 当打印表达式的执行结果时，格式化字符串（format string）中大括号中留空，格式化字符串后跟逗号分隔的需要打印的表达式列表，其顺序与每一个空大括号占位符的顺序一致
            // 1

            match guess.cmp(&secret_number) {
                Ordering::Less => println!("Too small!"),
                Ordering::Equal => {
                    println!("You win! ");
                    break;  // 退出循环
                }
                Ordering::Greater => println!("Too big!"),
                // 一个 match 表达式由 分支（arms） 构成。一个分支包含一个 模式（pattern）和表达式开头的值与分支模式相匹配时应该执行的代码。Rust 获取提供给 match 的值并挨个检查每个分支的模式。
            }
        }

        if count <= count_less {
            count_less = count;
        }
    }

    for _element in 1..=3 {
        println!("恭喜通关{game_times}猜数字游戏，其中最短只用 {count_less}次！ 大大的赞！！！");
    }

}
*/
