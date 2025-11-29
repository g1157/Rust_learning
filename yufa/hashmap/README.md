# HashMap - 哈希表

Rust HashMap 集合类型学习 - 键值对存储。

## 学习内容

### 1. 创建和插入

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);
```

**注意**: HashMap 不在 prelude 中，需要手动引入。

### 2. 更新值

```rust
// 覆盖已有值
scores.insert(String::from("Blue"), 25);

// 仅在键不存在时插入
scores.entry(String::from("Blue")).or_insert(50);
```

### 3. 访问值

```rust
let team = String::from("Blue");
let score = scores.get(&team)
    .copied()       // Option<&V> -> Option<V>
    .unwrap_or(0);  // 默认值
```

### 4. 遍历

```rust
for (key, value) in &scores {
    println!("{key}: {value}");
}
```

### 5. 根据旧值更新

```rust
let text = "hello world wonderful world";
let mut map = HashMap::new();

for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0);
    *count += 1;  // 解引用并修改
}
```

### 6. 所有权

| 类型 | 行为 |
|------|------|
| `Copy` 类型（如 `i32`） | 值被复制 |
| `String` 等 | 所有权转移到 HashMap |
| 引用 | 引用的值必须有效 |

## 运行

```bash
cd yufa/hashmap
cargo run
```
