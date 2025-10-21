
 // 大部分错误并没有严重到需要程序完全停止执行。有时函数失败的原因很容易理解并加以处理
use std::fs::File;
use std::io::ErrorKind;

enum Result<T, E> {       // T 和 E 是泛型类型参数
    Ok(T),                // T 代表成功时返回的 Ok 变体中的数据的类型，而 E 代表失败时返回的 Err 变体中的错误的类型。
    Err(E),
}


fn main() {
    let greeting_file_result = File::open("hello.txt");
    // 当 File::open 成功时，greeting_file_result 变量将会是一个包含文件句柄的 Ok 实例。当失败时，greeting_file_result 变量将会是一个包含了更多关于发生了何种错误的信息的 Err 实例。
    let greeting_file = match greeting_file_result {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating the file: {e:?}"),
            },
            _ => {
                panic!("Problem opening the file: {error:?}");
            }
        },
    };

    // match 能够胜任它的工作，不过它可能有点冗长并且不总是能很好的表明其意图
    let greeting_file = File::open("hello.txt").unwrap();
        // 如果 Result 值是变体 Ok，unwrap 会返回 Ok 中的值。如果 Result 是变体 Err，unwrap 会为我们调用 panic!
    let greeting_file = File::open("hello.txt")
        .expect("hello.txt should be included in this project");
        // expect 方法也允许我们自定义 panic! 的错误信息。使用 expect 而不是 unwrap 并提供一个好的错误信息可以表明你的意图并更易于追踪 panic 的根源。


}    
