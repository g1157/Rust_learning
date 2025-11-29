# hk3-3 - Self 类型与可变借用

学习 Rust 中 `Self` 类型和可变借用的练习。

## 功能

扩展 `TrafficLight` 结构体，演示：
- `Self` 类型的使用
- 可变借用 (`&mut self`)
- 修改结构体字段

## 技术要点

- **Self 关键字**: `self: &Self` 等价于 `&self`
- **可变借用**: `&mut TrafficLight` 允许修改结构体成员
- **to_string()**: 与 `to_owned()` 类似，转换为 `String` 类型

## 运行

```bash
cargo run
```
