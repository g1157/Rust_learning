# Functions - 函数

Rust 函数语法学习 - 参数、返回值与表达式。

## 学习内容

### 1. 函数定义

```rust
fn another_function() {
    println!("Another function.");
}
```

**命名规范**: 使用 `snake_case`（小写+下划线）

### 2. 带参数的函数

```rust
fn print_value(x: i32) {
    println!("The value is: {x}");
}

fn print_labeled(value: i32, label: char) {
    println!("{value}{label}");
}
```

**注意**: 必须声明参数类型

### 3. 返回值

```rust
fn five() -> i32 {
    5  // 无分号，作为返回值
}

fn plus_one(x: i32) -> i32 {
    x + 1  // 表达式，不加分号
}
```

### 4. 语句 vs 表达式

Rust 是**基于表达式的语言**：

| 类型 | 特点 | 示例 |
|------|------|------|
| 语句 | 不返回值 | `let x = 5;` |
| 表达式 | 返回值 | `5`, `x + 1`, `{}` |

```rust
let y = {
    let x = 3;
    x + 1  // 表达式，无分号
};  // y = 4
```

### 5. 关键点

- 函数调用是表达式
- 宏调用是表达式
- 代码块 `{}` 是表达式
- 表达式加分号变成语句

## 运行

```bash
cd yufa/fnc
cargo run
```
