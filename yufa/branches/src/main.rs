fn main() {
    let number = 3;

    if number < 5 {         //代码中的条件必须是 bool 值，不像 Ruby 或 JavaScript 这样的语言，Rust 并不会尝试自动地将非布尔值转换为布尔值，如5会报错
        println!("condition was true");
    } else {               //如果不提供 else 表达式并且条件为 false 时，程序会直接忽略 if 代码块并继续执行下面的代码。
        println!("condition was false");
    }

    let number = 6;
    if number % 4 == 0 {
        println!("number is divisible by 4");
    } else if number % 3 == 0 {   //else if 表达式与 if 和 else 组合来实现多重条件，Rust 只会执行第一个条件为 true 的代码块
        println!("number is divisible by 3");
    } else if number % 2 == 0 {
        println!("number is divisible by 2");
    } else {
        println!("number is not divisible by 4, 3, or 2");
    }

    let condition = true;            
    let number = if condition { 5 } else { 6 };   //if 是一个表达式，整个 if 表达式的值取决于哪个代码块被执行
                                                  //这意味着，if 的每个分支的可能的返回值都必须是相同类型
                                                  //Rust 需要在编译时就确切的知道 number 变量的类型，这样它就可以在编译时验证在每处使用的 number 变量的类型是有效的。
    println!("The value of number is: {number}");


    loop {
        println!("again!");
        break;   //使用 break 关键字来退出循环
    }

    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2;      //停止循环的 break 表达式后添加你希望返回的值；这个值就会作为循环的返回值返回
        }
    };
    println!("The result is {result}");

    xunhuanbiaoqian();
    while_loop();
    for_loop();

}

fn xunhuanbiaoqian() {
    let mut count = 0;
    'counting_up: loop {          //外层循环有 一个标签 counting_up
        println!("count = {count}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");
            if remaining == 9 {
                break;      //第一个没有指定标签的 break 将只退出内层循环
            }
            if count == 2 {
                break 'counting_up; //break 'counting_up; 语句将退出外层循环
            }
            remaining -= 1;
        }

        count += 1;
    }
    println!("End count = {count}");
}
//Rust 有三种循环：loop、while 和 for

fn while_loop() {
    let mut number = 3;

    while number != 0 {
        println!("{number}!");

        number -= 1;
    }

    println!("LIFTOFF!!!");
}

fn for_loop() {
    let a = [10, 20, 30, 40, 50];
    let mut index = 0;

    while index < 5 {
        println!("the value is: {}", a[index]);

        index += 1;
    }

    for element in a {     //for 循环的安全性和简洁性使得它成为 Rust 中使用最多的循环结构
        println!("the value is: {element}");
    }

    for number in (1..4).rev() {     //(1..4) 表示一个范围，从 1 到 3（不包括 4）。.rev() 把这个范围反转，所以顺序变成 3、2、1。 (1..=100) 表示一个范围，从 1 到 100（包括 100）。
        println!("{number}!");
    }
    println!("LIFTOFF!!!");
}