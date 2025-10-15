
struct TrafficLight {
    color: String,
}

impl TrafficLight {
    // 使用 Self 填空
    pub fn show_state(self: &Self)  {
        println!("the current state is {}", self.color);
    }

    // 填空，不要使用 Self 或其变体
    pub fn change_state(self: &mut TrafficLight) {
        self.color = "green".to_string()  //和to_owned()一样，.to_string() 也能把字符串字面量转换为 String 类型
    }
}
fn main() {
     let mut light = TrafficLight{
        color: "red".to_owned(),
    };
    light.show_state();
    light.change_state();
}