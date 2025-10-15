struct Rectangle {
    width: u32,
    height: u32,
}
// 为了使函数定义于 Rectangle 的上下文中，我们开始了一个 impl 块（impl 是 implementation 的缩写），这个 impl 块中的所有内容都将与 Rectangle 类型相关联。接着将 area 函数移动到 impl 大括号中，并将签名中的第一个（在这里也是唯一一个）参数和函数体中其他地方的对应参数改成 self。
// 然后在 main 中将我们先前调用 area 方法并传递 rect1 作为参数的地方，改成使用 方法语法（method syntax）在 Rectangle 实例上调用 area 方法

impl Rectangle {
    // 完成 area 方法，返回矩形 Rectangle 的面积
    fn area(&self) -> u32 {                     // 在 area 的签名中，使用 &self 来替代 rectangle: &Rectangle，&self 实际上是 self: &Self 的缩写
        self.width * self.height
    }
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };

    assert_eq!(rect1.area(), 1500);
    // assert_eq!(rect1.area(), 1500);，这个是断言宏，不是打印语句。它的作用是检查 rect1.area() 的结果是否等于 1500，如果不相等，程序会报错并显示错误信息，但如果相等，什么都不会显示
}