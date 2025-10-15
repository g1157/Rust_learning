//3.3 函数
// Rust 代码中的函数和变量名使用 snake case 规范风格。在 snake case 中，所有字母都是小写并使用下划线分隔单词

fn main() {
    println!("Hello, world!");

    another_function_1(); //Rust 不关心函数定义所在的位置，只要函数被调用时出现在调用之处可见的作用域内就行。
    another_function_2(5);
    print_labeled_measurement(5, 'h');
    let y = 6; //6 是一个表达式，它计算出的值是 

    //函数调用是一个表达式。宏调用是一个表达式。用大括号创建的一个新的块作用域也是一个表达式
    let y = {
        let x = 3;
        x + 1   //如果在表达式的结尾加上分号，它就变成了语句，而语句不会返回值
    };
    println!("The value of y is: {y}");
    
    let x = five();
    println!("The value of x is: {x}"); 

    let x = plus_one(5);
    println!("The value of x is: {x}");
}

fn another_function_1() {
    println!("Another function.");
}

fn another_function_2(x: i32) {
    println!("The value of x is: {x}");
}

fn print_labeled_measurement(value: i32, unit_label: char) {
    println!("The measurement is: {value}{unit_label}");
}

fn five() -> i32 { //我们并不对返回值命名，但要在箭头（->）后声明它的类型。
    5
}

fn plus_one(x: i32) -> i32 {
    x + 1          // 如果加了分号会导致没有返回值，和设定的返回一个i32类型冲突
}


// 参数
// 定义为拥有 参数（parameters）的函数，参数是特殊变量，是函数签名的一部分。当函数拥有参数（形参）时，可以为这些参数提供具体的值（实参）

// 因为 Rust 是一门基于表达式（expression-based）的语言，这是一个需要理解的重要区别。这与其他语言不同，例如 C 和 Ruby，它们的赋值语句会返回所赋的值。
// Rust中，语句（Statements）是执行一些操作但不返回值的指令。表达式（Expressions）计算并产生一个值。
// 语句不返回值。因此，不能把 let 语句赋值给另一个变量

