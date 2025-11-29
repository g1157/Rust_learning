# Result - 错误处理

Rust Result 类型学习 - 可恢复错误处理。

## 学习内容

### 1. Result 枚举

```rust
enum Result<T, E> {
    Ok(T),   // 成功，包含返回值
    Err(E),  // 失败，包含错误信息
}
```

### 2. 使用 match 处理

```rust
use std::fs::File;
use std::io::ErrorKind;

let file = match File::open("hello.txt") {
    Ok(file) => file,
    Err(error) => match error.kind() {
        ErrorKind::NotFound => match File::create("hello.txt") {
            Ok(fc) => fc,
            Err(e) => panic!("创建失败: {e:?}"),
        },
        _ => panic!("打开失败: {error:?}"),
    },
};
```

### 3. unwrap 和 expect

```rust
// unwrap: 成功返回值，失败 panic
let file = File::open("hello.txt").unwrap();

// expect: 同 unwrap，但可自定义错误信息
let file = File::open("hello.txt")
    .expect("hello.txt should exist");
```

### 4. 何时使用

| 方法 | 使用场景 |
|------|----------|
| `match` | 需要精细处理不同错误 |
| `unwrap` | 确定不会失败，或测试代码 |
| `expect` | 比 `unwrap` 更易调试 |
| `?` 运算符 | 传播错误到调用者 |

## 运行

```bash
cd yufa/result
cargo run
```

**注意**: 运行后会创建 `hello.txt` 文件。
