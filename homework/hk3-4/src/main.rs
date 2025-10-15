#[derive(Debug)]
struct TrafficLight {
    color: String,
}

impl TrafficLight {
    // 1. 实现下面的关联函数 new,
    // 2. 该函数返回一个 TrafficLight 实例，包含 color "red"
    // 3. 该函数必须使用 Self 作为类型，不能在签名或者函数体中使用 TrafficLight
    pub fn new() -> Self { //不是方法的关联函数经常被用作返回一个结构体新实例的构造函数，相当于在大类用函数创建了一个小类
        Self {
            color: "red".to_owned(), 
        }
    }

    pub fn get_state(&self) -> &str {
        &self.color         //有color字段，返回color字段的引用
    }
}

fn main() {
    let light = TrafficLight::new();
    assert_eq!(light.get_state(), "red");         //与red对比，让上面定义为red
}