// Rust 是 静态类型（statically typed）语言，也就是说在编译时就必须知道所有变量的类型。


fn main() {

    // 如果变量没被使用会报 ^^^^^^ help: if this is intentional, prefix it with an underscore: `_second`
    let guess: u32 = "42".parse().expect("Not a number!");
    // 如果不像上面的代码这样添加类型注解 : u32，Rust 会错误
    
    let x = 2.0; // f64
    let y: f32 = 3.0; // f32
    
    // addition
    let sum = 5 + 10;
    // subtraction
    let difference = 95.5 - 4.3;
    // multiplication
    let product = 4 * 30;
    // division
    let quotient = 56.7 / 32.2;
    let truncated = -5 / 3; // 结果为 -1
                        // 整数除法会向零舍入到最接近的整数
    // remainder
    let remainder = 43 % 5;

    let t = true;
    let f: bool = false; // with explicit type annotation


    let c = 'z'; // 我们用单引号声明 char 字面值，而与之相反的是，使用双引号声明字符串字面值
    let z: char = 'ℤ'; // with explicit type annotation
    let heart_eyed_cat = '😻';
    // Rust 的 char 类型的大小为四个字节 (four bytes)，并代表了一个 Unicode 标量值（Unicode Scalar Value），这意味着它可以比 ASCII 表示更多内容

    let tup: (i32, f64, u8) = (500, 6.4, 1);
    
    let tup = (500, 6.4, 1);
    let (x, y, z) = tup;  // 模式匹配（pattern matching）来解构（destructure）元组值
    println!("The value of y is: {y}");    

    //也可以使用点号（.）后跟值的索引来直接访问所需的元组元素
    let x: (i32, f64, u8) = (500, 6.4, 1);
    let five_hundred = x.0;
    let six_point_four = x.1;
    let one = x.2;


    let months = ["January", "February", "March", "April", "May", "June", "July",
              "August", "September", "October", "November", "December"];
    let a: [i32; 5] = [1, 2, 3, 4, 5];

    let a = [3; 5];    // 这种写法与 let a = [3, 3, 3, 3, 3]; 效果相同，但更简洁。
    
    let a = [1, 2, 3, 4, 5];
    let first = a[0];
    let second = a[1];
    // 如果索引超出了数组长度，Rust 会 panic，这是 Rust 术语，它用于程序因为错误而退出的情况。
}

// 标量 
// 标量（scalar）类型代表一个单独的值。Rust 有四种基本的标量类型：整型、浮点型、布尔类型和字符类型

    // 整型  架构相关	isize	usize 如 u32是占据 32 比特位的无符号整数
    // 有符号 和 无符号 代表数字能否为负值 整型默认是 i32


    // 浮点型
    // Rust 也有两个原生的 浮点数（floating-point numbers）类型，它们是带小数点的数字。Rust 的浮点数类型是 f32 和 f64，分别占 32 位和 64 位。默认类型是 f64

    // 布尔类型

    // 字符类型

// 复合类型

// 元组类型：元组是一个将多个不同类型的值组合进一个复合类型的主要方式，包含在圆括号中的逗号分隔的值列表来创建一个元组

// 数组类型：每个元素的类型必须相同。Rust 中的数组与一些其他语言中的数组不同，Rust 中的数组长度是固定的，当你确定元素个数不会改变时，数组会更有用