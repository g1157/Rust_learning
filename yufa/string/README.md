# String - 字符串

Rust 字符串类型学习 - String 与 &str。

## 学习内容

### 1. 字符串类型

| 类型 | 说明 |
|------|------|
| `&str` | 字符串切片，不可变引用 |
| `String` | 可增长、可变、拥有所有权 |

### 2. 创建 String

```rust
let s = String::new();                    // 空字符串
let s = "initial".to_string();            // 从字面值
let s = String::from("initial");          // from 函数
```

### 3. 更新字符串

```rust
let mut s = String::from("foo");
s.push_str("bar");    // 追加字符串切片
s.push('!');          // 追加单个字符
```

### 4. 拼接字符串

```rust
// 使用 + 运算符（s1 被移动）
let s3 = s1 + &s2;

// 使用 format! 宏（不移动任何值）
let s = format!("{s1}-{s2}-{s3}");
```

### 5. 索引访问

**String 不支持索引**，因为：
- UTF-8 编码，字符可能占多个字节
- 无法确定用户想要字节、Unicode 标量还是字形簇

```rust
// 使用切片（需确保在字符边界）
let hello = "Здравствуйте";
let s = &hello[0..4];  // "Зд"

// 遍历字符
for c in "Зд".chars() {
    println!("{c}");
}

// 遍历字节
for b in "Зд".bytes() {
    println!("{b}");
}
```

### 6. 关键点

- 字符串是 UTF-8 编码
- `String` 是 `Vec<u8>` 的封装
- 切片索引必须在有效的 UTF-8 字符边界

## 运行

```bash
cd yufa/string
cargo run
```
