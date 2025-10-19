use serde::Deserialize;
use std::collections::HashMap; // HashMap<K, V>，一个泛型集合类型，K 是键的类型，V 是值的类型
use plotly::{Plot, Scatter};  
use plotly::common::Mode;

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]   //给 AirModel 派生 Clone 和 Copy
enum AirModel {
    Isothermal,
    Adiabatic,
}

// 炮弹基本数据，与json文件结构对应
#[derive(Debug, Deserialize)]
struct Cannon {        
    x0: f64,
    y0: f64,
    theta: f64,
    v0: f64,
    resistance_type: AirModel,
    style: String,         
}

fn read_config(config_path: &str) -> HashMap<String, Cannon> {
    let config = std::fs::read_to_string(config_path)   //std:fs filesystem用于处理文件系统相关的操作
        .expect("读取配置文件失败");
    serde_json::from_str(&config)                    //会把配置文件里的 JSON 内容转换成 Rust 里的 HashMap，自动从 JSON 反序列化
        .expect("错误的解析了配置文件") 
}

#[derive(Debug)]
struct CannonState {
    x: f64,
    y: f64,
    t: f64,
    vx: f64,
    vy: f64,
    resistance_type: AirModel,
}

const G: f64 = 9.8;
const DT: f64 = 0.5;
const alpha: f64 = 0.5; // 经验值，可调整
const T: f64 = 300.0; // 当前温度，单位K

impl CannonState {
    fn new(cannon: &Cannon) -> Self {
        let theta_rad = cannon.theta.to_radians();      //改为弧度制
        let vx = cannon.v0 * theta_rad.cos();
        let vy = cannon.v0 * theta_rad.sin();
        Self {                                          // 返回一个新的 CannonState 实例，多了vx、vy字段
            x: cannon.x0,      //初始化用:而不是=
            y: cannon.y0,
            t: 0.0,
            vx,
            vy,
            resistance_type: cannon.resistance_type,
        }
    }
    fn next(&self) -> Result<Self, Self> {                //Result<T, E> 是 Rust 标准库的一个枚举类型，表示“要么成功（Ok），要么失败（Err）”。
        let v = (self.vx.powi(2) + self.vy.powi(2)).sqrt();   //powi 是 Rust 标准库 f64 类型的方法，表示“以整数次幂”

        // 计算密度修正因子
        let density_factor = match self.resistance_type {
            AirModel::Adiabatic => {
                // 绝热模型
                let T0: f64 = 300.0;
                let a: f64 = 6.5e-3;
                (1.0 - a * self.y / T0).powf(2.5)
            }
            AirModel::Isothermal => {
                // 等温模型
                let h = 1e4;
                (-self.y / h).exp()
            }
        };
        let B2M = 4e-5 * (T / 300.0).powf(alpha); //参考温度300K
        let next_x = self.x + self.vx * DT;
        // let next_vx = self.vx - B2M * v * self.vx * DT;
        let next_vx = self.vx - B2M * v * self.vx * DT * density_factor;
        let next_y = self.y + self.vy * DT;
        // let next_vy = self.vy - G * DT - B2M * v * self.vy * DT;
        let next_vy = self.vy - G * DT - B2M * v * self.vy * DT * density_factor;
        let next_t = self.t + DT;

        let mut result = Self {  //更新
            x: next_x,
            y: next_y,
            t: next_t,
            vx: next_vx,
            vy: next_vy,
            resistance_type: self.resistance_type, // 补上这一行
        };

        if next_y < 0.0 {       //落地修正
            let r = - self.y / next_y;
            result.x = (self.x + r * next_x) / (r + 1.0);
            result.y = 0.0; 
            return Err(result); //将result包装在Err中返回给第二个Self
        }
        Ok(result) //将result包装在Ok中返回给第一个Self
    }
}

#[derive(Debug)]
struct CannonTrace {
    path: Vec<CannonState>,
}

impl CannonTrace {
    fn new(initial_state: CannonState) -> Self {
        Self {
            path: vec![initial_state],  // vec! 是 Rust 标准库的宏，用来快速创建一个动态数组（Vec 类型）
        }
    }
    fn calculate_trace(mut self) -> Self {
        loop {
            match self.path.last().unwrap().next() {    //last返回动态数组（Vec）最新状态的引用，unwrap解引用
                Ok(next_state) => {                     //match 会自动把 Ok(result) 里的 result 绑定到 next_state，把 Err(result) 绑定到 final_state
                    self.path.push(next_state);         //用 push 方法把这个新状态加入到 self.path 的最后
                }
                Err(final_state) => {
                    self.path.push(final_state);
                    break;
                }
            }
        }
        self
    }
}



fn plot_traces(cannon_traces: &HashMap<String, CannonTrace>) {
    let mut plot = Plot::new();
    
    // 为每个轨迹创建散点图
    for (name, trace) in cannon_traces {
        let x_coords: Vec<f64> = trace.path.iter().map(|state| state.x).collect();
        let y_coords: Vec<f64> = trace.path.iter().map(|state| state.y).collect();
        
        // 创建散点图
        let scatter = Scatter::new(x_coords, y_coords)
            .name(name.clone())
            .mode(Mode::Lines);
        
        // 添加到图表
        plot.add_trace(scatter);
    }
    
    // 简单的图表，不设置额外的标题（根据plotly版本的API调整）
    
    // 保存为HTML文件
    plot.write_html("cannon_traces.html");
    println!("轨迹图已保存到 cannon_traces.html");
    
    // 尝试在浏览器中打开（可选）
    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg("cannon_traces.html").spawn().ok(); }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/C", "start", "cannon_traces.html"]).spawn().ok(); }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg("cannon_traces.html").spawn().ok(); }
}

fn main() {
    let config = read_config("cannon_config.json");
    println!("{:?}", config);

    let mut cannon_traces: HashMap<String, CannonTrace> = HashMap::new();

    for (name, cannon) in config.iter() {
        println!("炮弹名字：{}", name);
        let state = CannonState::new(cannon);
        println!("初始状态: {:?}", state);

        let trace = CannonTrace::new(state)
            .calculate_trace();

        cannon_traces.insert(name.clone(), trace);  //insert用于向 map 里添加一个键值对， 是当前炮弹的名字（字符串），作为键。trace 是当前炮弹的完整轨迹（CannonTrace 类型），作为值

        println!("完整路径长度: {}", cannon_traces[name].path.len());
        println!("================================");

    }
    
    // 调用绘图函数
    plot_traces(&cannon_traces);

}
