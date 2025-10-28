fn trim_me(input: &str) -> &str {
    // TODO: 去除字符串两端的空白字符。
    input.trim()
}

fn compose_me(input: &str) -> String {
    // TODO: 在字符串后面添加 " world!" ，有很多方法可以做到这一点。

    // format!("{} world!", input) 可行
    // format!("{input} world!")


    // input.to_string().push_str(" world!") //错误，push_str 返回的是 ()，不是 String
    // let mut s = input.to_string();
    // s.push_str(" world!");
    // s 

   input.to_owned() + " world!" 
   // 用来把“借用的值”转换成对应的“拥有所有权的值”（通常是克隆底层数据并分配内存）
   // &str 的 to_string 与 to_owned 等价，都会生成 String，to_string 依赖 Display/ToString，to_owned 更通用（适用于非字符串类型的“拥有型”对应物）

}

fn replace_me(input: &str) -> String {
    // TODO: 替换字符串中的 "cars" 为 "balloons" 。
    input.replace("cars", "balloons")
}

fn main() {
    // (可选)你可以选择性地在此处进行试验。
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_a_string() {
        assert_eq!(trim_me("Hello!     "), "Hello!");
        assert_eq!(trim_me("  What's up!"), "What's up!");
        assert_eq!(trim_me("   Hola!  "), "Hola!");
    }

    #[test]
    fn compose_a_string() {
        assert_eq!(compose_me("Hello"), "Hello world!");
        assert_eq!(compose_me("Goodbye"), "Goodbye world!");
    }

    #[test]
    fn replace_a_string() {
        assert_eq!(
            replace_me("I think cars are cool"),
            "I think balloons are cool",
        );
        assert_eq!(
            replace_me("I love to look at cars"),
            "I love to look at balloons",
        );
    }
}
