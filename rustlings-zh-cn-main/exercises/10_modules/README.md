# 模块(Modules)

在本节中，我们将介绍Rust的模块系统(module system)。

## 对应知识

- [The Module System](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)


## AI
Rust 的模块化做什么、有何好处、与对 struct 的 impl 有何区别

### 一、模块化是干嘛的

用 mod 定义命名空间与可见性边界，把代码按功能分组。
模块里可以放：结构体、枚举、函数、常量、trait、子模块等。
通过路径与 use 导入来组织访问：crate::a::b、super::、self:: 等。
通过可见性控制对外暴露：pub、pub(crate)、pub(super)、pub(in path)。
### 二、好处

封装与信息隐藏：只暴露需要的 API，内部细节保持私有。
命名隔离：避免全局名字冲突，路径更清晰。
代码组织：更易维护、测试、生成文档（rustdoc 按模块分组）。
API 设计：用 re-export（pub use）构建对外的扁平、友好接口面。
依赖管理/编译单元：和 crate 一起形成清晰的项目边界。
### 三、与对 struct 的 impl 的区别

关注点不同
模块（mod）：组织与可见性。是“放东西的命名空间”。
impl 块：给类型增加行为（方法、关联函数、关联常量）或实现 trait。
是否创建类型
模块不创建新类型；impl 针对已有类型（struct/enum/trait 对象）定义方法。
模块不创建新类型；impl 针对已有类型（struct/enum/trait 对象）定义方法。
访问方式
模块内的函数通过路径调用：sausage_factory::make_sausage()。
impl 内的方法可用点语法：value.method() 或 Type::assoc_fn()。
可见性规则
模块是私有边界：未标 pub 的项在模块外不可见。
impl 中的每个方法也有自己的可见性（pub/默认私有），同时还受所在模块的可见性影响（方法想对外可见，类型与字段也需合适的可见性）。
组织位置
一个类型可以在多个模块里有多个 impl（只要在同一 crate 内且不冲突）。
模块可以包含多个类型、函数、子模块等。