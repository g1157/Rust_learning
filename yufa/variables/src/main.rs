// 3.1变量和可变性

// 变量

// fn main() {
//     let x = 5;   //Rust 编译器保证，如果声明一个值不会变，它就真的不会变，所以你不必自己跟踪它。这意味着你的代码更易于推导。
//     println!("The value of x is: {x}");
//     x = 6;        // error[E0384]: cannot assign twice to immutable variable `x`
//     println!("The value of x is: {x}");
// }

// fn main() {
//     let mut x = 5;   // mutable，可改变的 
//     println!("The value of x is: {x}");
//     x = 6;
//     println!("The value of x is: {x}");
// }

// 常量

// const THREE_HOURS_IN_SECONDS: u32 = 60 * 60 * 3;

// 常量不光默认不可变，它总是不可变。声明常量使用 const 关键字而不是 let，并且 必须 注明值的类型。
// 编译器能够在编译时计算一组有限的操作，这使我们可以选择以更容易理解和验证的方式写出此值，而不是将此常量设置为值 10,800。


// 遮蔽

fn main() {
    let x = 5;   // 声明一个不可变量
    
    let x = x + 1; // 却可以通过let遮蔽，本质上创建了一个新变量，我们可以改变值的类型，并且复用这个名字

    let spaces = "   ";
    let spaces = spaces.len();
    println!("The value of spaces is: {spaces}");

    //然而，如果尝试使用 mut，将会得到一个编译时错误
    // let mut spaces = "   ";
    // spaces = spaces.len(); // spaces.len() 返回的是一个 usize 类型的值，表示 spaces 字符串的长度（即 3），而 spaces 变量依然是 &str 类型。由于 usize 和 &str 类型不兼容

    { //在使用花括号创建的内部作用域内
        let x = x * 2; 
        println!("The value of x in the inner scope is: {x}");
    }

    println!("The value of x is: {x}");
    // 当该作用域结束时，内部遮蔽的作用域也结束了，x 又返回到 6
}

