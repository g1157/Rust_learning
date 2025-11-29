# hk3-1 - Rectangle 结构体与方法

学习 Rust 结构体和方法的基础练习。

## 功能

实现一个 `Rectangle` 结构体，包含：
- `width` 和 `height` 字段
- `area()` 方法计算矩形面积

## 技术要点

- **结构体定义**: `struct Rectangle { width: u32, height: u32 }`
- **impl 块**: 在结构体上下文中定义方法
- **self 引用**: 使用 `&self` 作为方法的第一个参数
- **断言宏**: `assert_eq!` 用于验证结果

## 运行

```bash
cargo run
```
