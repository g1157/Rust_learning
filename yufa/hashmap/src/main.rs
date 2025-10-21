// 哈希 map（hash map） 相较不常用，所以并没有被 prelude 自动引用
    //HashMap<K, V> 类型储存了一个键类型 K 对应一个值类型 V 的映射,类似字典

use std::collections::HashMap;


fn main() {
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);    // 对于像 i32 这样的实现了 Copy trait 的类型，其值可以拷贝进哈希 map
    scores.insert(String::from("Blue"), 25);    // 如果插入一个已存在的键，则会更新对应的值
    scores.insert(String::from("Yellow"), 50);
    scores.entry(String::from("Blue")).or_insert(50); 
        // entry 方法用于获取一个键的 Entry 枚举，如果该键不存在则插入并返回一个可变引用
        // or_insert 方法在键对应的值存在时就返回这个值的可变引用，如果不存在则将参数作为新值插入并返回新值的可变引用

    // 根据值来更新值
    
    let text = "hello world wonderful world";

    let mut map = HashMap::new();

    for word in text.split_whitespace() {     //split_whitespace 方法返回一个由空格分隔 text 值子 slice 的迭代器
        let count = map.entry(word).or_insert(0);
        *count += 1;
    }

    println!("{map:?}");

    
    let team_name = String::from("Blue");
    let score = scores.get(&team_name).copied().unwrap_or(0); 
        // copied 方法将 Option<&V> 转换为 Option<V>，因为 get 方法返回的是一个值的引用,通常用于 将一个迭代器中的每个元素从引用转换为其值的拷贝
        // unwrap_or 在 scores 中没有该键所对应的项时将其设置为零

    for (key, value) in &scores {
        println!("{key}: {value}");
    }

    // 对于像 String 这样拥有所有权的值，其值将被移动而哈希 map 会成为这些值的所有者
        use std::collections::HashMap;

    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");

    let mut map = HashMap::new();
    map.insert(field_name, field_value);
    // 值本身插入之后， field_name 和 field_value 不再有效
    // 但如果是引用将不改变值的作用域


}
