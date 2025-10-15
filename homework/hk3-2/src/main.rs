
// 只填空，不要删除任何代码行!
#[derive(Debug)]
struct TrafficLight {
    color: String, 
    // 在 Rust 结构体定义和实例化时，每个字段后面都要加逗号 ,，即使是最后一个字段也建议加逗号。
}

impl TrafficLight {
    // pub fn 表示公开的函数定义。 pub 让这个函数可以在其他模块或文件中被访问。
    pub fn show_state(&self)  { 
        //&mut self：表示可变借用，可以修改结构体成员。如果你要在方法里改变 color，就需要用 &mut self，并且调用时变量也要声明为 mut。
        println!("the current state is {}", self.color);
    }
}
fn main() {
    let light = TrafficLight{
        color: "red".to_owned(), 
        //"red" 是字符串字面量。 .to_owned() 把字符串字面量 "red" 转换为 String 类型（因为结构体字段类型是 String）。
        // &str：字符串切片，通常是字符串字面量，比如 "red"，它是只读的、存储在程序的静态内存区。
        // String：可变字符串类型，分配在堆上，可以动态增长和修改，比如 String::from("red") 或 "red".to_owned()。
    };
    // 不要拿走 light 的所有权
    light.show_state();
    // 否则下面代码会报错
    println!("{:?}", light);
}