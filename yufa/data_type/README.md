# Data Type - 数据类型

Rust 基本数据类型学习 - 标量类型与复合类型。

## 学习内容

### 核心概念

Rust 是**静态类型语言**，编译时必须知道所有变量的类型。

### 1. 标量类型（Scalar Types）

四种基本标量类型：

| 类型 | 说明 | 示例 |
|------|------|------|
| 整型 | 有符号/无符号整数 | `i32`, `u64`, `isize` |
| 浮点型 | 小数 | `f32`, `f64`（默认） |
| 布尔型 | 真/假 | `bool` |
| 字符型 | Unicode 标量值 | `char`（4字节） |

### 2. 整型

```rust
let x: u32 = 42;      // 无符号 32 位
let y: i64 = -100;    // 有符号 64 位
let z: isize = 0;     // 架构相关（32/64位）
```

### 3. 浮点型

```rust
let x = 2.0;          // f64（默认）
let y: f32 = 3.0;     // f32
```

### 4. 字符类型

```rust
let c = 'z';          // 单引号
let heart = '❤';      // Unicode 字符
```

### 5. 元组（Tuple）

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;  // 解构
let first = tup.0;    // 索引访问
```

### 6. 数组（Array）

```rust
let arr = [1, 2, 3, 4, 5];
let arr: [i32; 5] = [1, 2, 3, 4, 5];  // 显式类型
let arr = [3; 5];                      // [3, 3, 3, 3, 3]
let first = arr[0];                    // 索引访问
```

**注意**: 数组长度固定，索引越界会 panic。

## 运行

```bash
cd yufa/data_type
cargo run
```
