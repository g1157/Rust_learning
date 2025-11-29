# hk3-5 - 枚举类型与 match 表达式

学习 Rust 枚举类型和模式匹配的练习。

## 功能

使用枚举重新实现交通灯：
- 定义 `TrafficLightColor` 枚举
- 实现 `color()` 方法返回颜色字符串

## 技术要点

- **枚举定义**: `enum TrafficLightColor { Red, Yellow, Green }`
- **match 表达式**: 模式匹配处理不同枚举变体
- **命名空间**: 使用 `::` 访问枚举变体
- **Debug trait**: 支持 `{:?}` 格式化输出

## 枚举 vs 结构体

- 结构体：聚合多个字段
- 枚举：表示一组可能的值中的一个

## 运行

```bash
cargo run
```
