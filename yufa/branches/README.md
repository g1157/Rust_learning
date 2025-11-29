# Branches - 分支与循环

Rust 控制流语法学习 - if/else 分支和循环结构。

## 学习内容

### 1. if/else 条件分支

```rust
let number = 3;
if number < 5 {
    println!("condition was true");
} else {
    println!("condition was false");
}
```

**要点**:
- 条件必须是 `bool` 类型（不像 Ruby/JavaScript 会自动转换）
- `if` 是表达式，可以返回值
- 各分支返回值类型必须相同

### 2. else if 多重条件

```rust
if number % 4 == 0 {
    println!("divisible by 4");
} else if number % 3 == 0 {
    println!("divisible by 3");
}
```

### 3. 三种循环

| 循环类型 | 用途 |
|----------|------|
| `loop` | 无限循环，需手动 `break` |
| `while` | 条件循环 |
| `for` | 遍历集合，最常用 |

### 4. loop 返回值

```rust
let result = loop {
    counter += 1;
    if counter == 10 {
        break counter * 2;  // 返回值
    }
};
```

### 5. 循环标签

```rust
'outer: loop {
    loop {
        break 'outer;  // 退出外层循环
    }
}
```

### 6. for 循环与 Range

```rust
for element in array { }          // 遍历数组
for number in (1..4).rev() { }    // 范围反转
for i in 1..=100 { }              // 包含终点
```

## 运行

```bash
cd yufa/branches
cargo run
```
