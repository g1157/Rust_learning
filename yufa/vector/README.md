# Vector - 动态数组

Rust Vec<T> 集合类型学习 - 可增长的同类型元素列表。

## 学习内容

### 1. 创建 Vector

```rust
let v: Vec<i32> = Vec::new();  // 空 vector，需类型注解
let v = vec![1, 2, 3];         // 使用宏创建
```

### 2. 更新 Vector

```rust
let mut v = Vec::new();
v.push(4);
v.push(5);
```

### 3. 读取元素

```rust
let v = vec![1, 2, 3, 4, 5];

// 索引访问（越界会 panic）
let third: &i32 = &v[2];

// get 方法（返回 Option，越界返回 None）
let third: Option<&i32> = v.get(2);
```

### 4. 遍历

```rust
// 不可变遍历
for i in &v {
    println!("{i}");
}

// 可变遍历
for i in &mut v {
    *i += 50;  // 解引用后修改
}
```

### 5. 存储不同类型

使用枚举包装不同类型：

```rust
enum Cell {
    Int(i32),
    Float(f64),
    Text(String),
}

let row = vec![
    Cell::Int(3),
    Cell::Text(String::from("blue")),
    Cell::Float(10.12),
];
```

### 6. 借用规则

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];  // 不可变借用
// v.push(4);       // 错误！不能同时存在可变和不可变借用
println!("{first}");
```

### 7. 关键点

- Vector 只能存储相同类型
- 离开作用域时自动释放
- 编译时必须知道元素类型

## 运行

```bash
cd yufa/vector
cargo run
```
