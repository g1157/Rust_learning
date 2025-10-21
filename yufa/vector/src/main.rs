

// 第一个类型是 Vec<T>，也被称为 vector。vector 允许我们在一个单独的数据结构中储存多于一个的值，它在内存中彼此相邻地排列所有的值。vector 只能储存相同类型的值。
// Rust 在编译时必须确切知道 vector 中的类型，这样它才能确定在堆上需要为每个元素分配多少内存。
fn main() {
    let v: Vec<i32> = Vec::new();  //这里我们增加了一个类型注解<i32>
        // 为了方便 Rust 提供了 vec! 宏，这个宏会根据我们提供的值来创建一个新的 vector。
    let v = vec![1, 2, 3];
    let mut v = Vec::new();
    v.push(4);

        //有两种方法引用 vector 中储存的值：通过索引或使用 get 方法。
    let v = vec![1, 2, 3, 4, 5];
    let third: &i32 = &v[2];
        // 当引用一个不存在的元素时 Rust 会造成 panic

    let forth: Option<&i32> = v.get(3);
        // 当 get 方法被传递了一个数组外的索引时，它不会 panic 而是返回 None

    // 回忆一下不能在相同作用域中同时存在可变和不可变引用的规则
        // 当我们获取了 vector 的第一个元素的不可变引用并尝试在 vector 末尾增加一个元素的时候，如果尝试在函数的后面再次引用这个元素是行不通的

    // 遍历 vector  
    let v = vec![100, 32, 57];
    for i in &v {
        println!("{i}");
    }

    let mut v = vec![100, 32, 57];
    for i in &mut v {  // 可变引用
        *i += 50;      // 在使用 += 运算符之前必须使用解引用运算符（*）获取 i 中的值
    }

// 储存不同类型
// vector 只能储存相同类型的值。这是很不方便的；绝对会有需要储存一系列不同类型的值的用例。幸运的是，枚举的成员都被定义为相同的枚举类型，所以当需要在 vector 中储存不同类型值时，我们可以定义并使用一个枚举！

    enum SpreadsheetCell {
        Int(i32),
        Float(f64),
        Text(String),
    }

    let row = vec![
        SpreadsheetCell::Int(3),
        SpreadsheetCell::Text(String::from("blue")),
        SpreadsheetCell::Float(10.12),
    ];

    // 类似于任何其他的 struct，vector 在其离开作用域时会被释放
}
