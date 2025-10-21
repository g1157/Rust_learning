// String
// Rust 的核心语言中只有一种字符串类型，字符串 slice str，它通常以被借用的形式出现，&str
// 记住字符串是 UTF-8 编码的，所以可以包含任何经过正确编码的数据

fn main() {
    // 很多 Vec<T> 上可用的操作在 String 中同样可用，事实上 String 被实现为一个带有一些额外保证、限制和功能的字节 vector 的封装
    let mut s = String::new();

    // 创建
        // 使用 to_string 方法从字符串字面值创建 String
    let data = "initial contents";
    let s = data.to_string();
    let s = "initial contents".to_string();

        // 使用 String::from 函数
    let s = String::from("initial contents");

    // 更新
        // push_str 方法的作用是将一个 &str（字符串切片）追加到 String 的末尾
    let mut s = String::from("foo");
    s.push_str("bar");

        // push 方法将单个字符追加到 String
    let mut s = String::from("lo");
    s.push('l');

    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1 + &s2; // 注意 s1 被移动了，不能继续使用。s1 在相加后不再有效的原因，和使用 s2 的引用的原因，与使用 + 运算符时调用的函数签名有关。

    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");
    let s = format!("{s1}-{s2}-{s3}"); // format! 宏类似于 println!，但它并不打印到屏幕上，而是返回一个包含了格式化文本的 String


    // 索引
        // String 不支持索引访问，String 是一个 Vec<u8> 的封装
        // 因为 Rust 无法知道你想要的到底是第几个字节、第几个 Unicode 标量值，还是第几个“人类可读的字符”（grapheme cluster）
    
        //可以使用 [] 和一个 range 来创建含特定字节的字符串 slice    
    let hello = "Здравствуйте";
    let s = &hello[0..4];

        // 操作字符串每一部分的最好的方法是明确表示需要字符还是字节
    for c in "Зд".chars() {      //chars 方法会将其分开并返回两个 char 类型的值
        println!("{c}");  
    }

    for b in "Зд".bytes() {     //bytes 方法返回每一个原始字节
        println!("{b}");
    }

}
