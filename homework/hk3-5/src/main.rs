#[derive(Debug)]
// 你选中的 #[derive(Debug)] 是一个属性宏，作用是让后面的类型（结构体或枚举）自动实现 Debug trait。
// 这样你就可以用 println!("{:?}", c); 这种调试格式，直接打印枚举或结构体的内容，方便调试和查看数据。


// 结构体给予你将字段和数据聚合在一起的方法，像 Rectangle 结构体有 width 和 height 两个字段。而枚举给予你一个途径去声明某个值是一个集合中的一员。

enum TrafficLightColor {
    Red,
    Yellow,
    Green,
}

// 为 TrafficLightColor 实现所需的方法
impl TrafficLightColor {
    pub fn color(&self) -> &str {
        match self {       //用match表达式来匹配枚举的不同变体，适合红绿灯场景
            TrafficLightColor::Red => "red",
            TrafficLightColor::Yellow => "yellow",
            TrafficLightColor::Green => "green",
        }
    }
}

fn main() {
    let c = TrafficLightColor::Yellow; 
    //注意枚举的变体位于其标识符的命名空间中，并使用两个冒号分开。这么设计的益处是直接表现属于哪个大类型
    assert_eq!(c.color(), "yellow");

    println!("{:?}",c);
}