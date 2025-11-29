# hk3-2 - TrafficLight 结构体与方法

学习 Rust 结构体方法和借用的练习。

## 功能

实现一个 `TrafficLight` 结构体，演示：
- 结构体字段的定义
- 公开方法 (`pub fn`)
- 不可变借用 (`&self`)

## 技术要点

- **Debug trait**: `#[derive(Debug)]` 自动实现调试输出
- **String 类型**: 使用 `to_owned()` 将字符串字面量转换为 `String`
- **借用规则**: 使用 `&self` 不转移所有权

## 运行

```bash
cargo run
```
