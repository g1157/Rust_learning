// use plotly::common::Mode;
// use plotly::layout::Axis;
// // use plotly::{
// //     Bar, Layout, Plot, Scatter,
// //     color::{NamedColor, Rgb, Rgba},
// // };
// use plotly::{Layout, Plot, Scatter};


// use crate::decay_data::DecayItem;
// use crate::decay_data::{DecayData, Iteration};

// pub fn draw(decay_data: &DecayData) {
//     // let x = DecayItem::extract_x(decay_data.get_sequence());
//     let sequence = decay_data.get_sequence();
//     let na = DecayItem::extract_na(sequence);
//     let nb = DecayItem::extract_nb(sequence);
//     let t = DecayItem::extract_t(sequence);
    
//     let mut plot = Plot::new();
//     // let trace = Scatter::new(t, x)
//     //     .mode(Mode::LinesMarkers)
//     //     .name(decay_data.get_name());
//     let trace_na = Scatter::new(t.clone(), na).name("A nucleus").mode(Mode::LinesMarkers);
//     let trace_nb = Scatter::new(t.clone(), nb).name("B nucleus").mode(Mode::LinesMarkers);

//     let layout1 = Layout::new().x_axis(
//         Axis::new()
//             .range(vec![0.0, decay_data.get_last_item().get_t()])
//             .show_line(true)
//             .show_grid(true)
//             .title("t")
//             .zero_line(true),
//     );
//     // plot.add_trace(trace);
//     plot.add_trace(trace_na);
//     plot.add_trace(trace_nb);
//     plot.set_layout(layout1);
//     // plot.show();
//     std::fs::write(
//         format!("plot_{}.html", decay_data.get_name()), 
//         plot.to_html(),
//     ).unwrap();
//     println!("Plot saved to plot_{}.html", decay_data.get_name());
// }

// //use plotly::{Layout, Plot, Scatter, common::Mode};

// /// 把两个 DecayData 画在同一张图上
// // pub fn plot_two_decay(data1: &DecayData, data2: &DecayData) {
// //     // 1. 抽取 (t, x) 序列
// //     let (t1, x1) = DecayItem::extract_xt(data1.get_sequence());
// //     let (t2, x2) = DecayItem::extract_xt(data2.get_sequence());

// //     // 2. 构建两条 Scatter
// //     let trace1 = Scatter::new(t1, x1).name(data1.get_name()).mode(Mode::LinesMarkers);

// //     let trace2 = Scatter::new(t2, x2).name(data2.get_name()).mode(Mode::LinesMarkers);

// //     // 3. 组装并显示
// //     let mut plot = Plot::new();
// //     plot.add_trace(trace1);
// //     plot.add_trace(trace2);

// //     plot.set_layout(
// //         Layout::new()
// //             .title("Decay Comparison")
// //             .x_axis(plotly::layout::Axis::new().title("time"))
// //             .y_axis(plotly::layout::Axis::new().title("atoms")),
// //     );

// //     plot.show();
// // }

// pub fn plot_two_decay(data1: &DecayData, data2: &DecayData) {
//     let seq1 = data1.get_sequence();
//     let t1 = DecayItem::extract_t(seq1);
//     let na1 = DecayItem::extract_na(seq1);
//     let nb1 = DecayItem::extract_nb(seq1);

//     let seq2 = data2.get_sequence();
//     let t2 = DecayItem::extract_t(seq2);
//     let na2 = DecayItem::extract_na(seq2);
//     let nb2 = DecayItem::extract_nb(seq2);

//     let trace_na1 = Scatter::new(t1.clone(), na1).name("A nucleus - data1").mode(Mode::LinesMarkers);
//     let trace_nb1 = Scatter::new(t1.clone(), nb1).name("B nucleus - data1").mode(Mode::LinesMarkers);
//     let trace_na2 = Scatter::new(t2.clone(), na2).name("A nucleus - data2").mode(Mode::LinesMarkers);
//     let trace_nb2 = Scatter::new(t2.clone(), nb2).name("B nucleus - data2").mode(Mode::LinesMarkers);

//     let mut plot = Plot::new();
//     plot.add_trace(trace_na1);
//     plot.add_trace(trace_nb1);
//     plot.add_trace(trace_na2);
//     plot.add_trace(trace_nb2);

//     plot.set_layout(
//         plotly::Layout::new()
//             .title("Double Decay Comparison")
//             .x_axis(Axis::new().title("Time"))
//             .y_axis(Axis::new().title("Number of Nuclei")),
//     );

//     // plot.show();
//     std::fs::write(
//         format!("plot_{}_vs_{}.html", data1.get_name(), data2.get_name()), 
//         plot.to_html(),
//     ).unwrap();
//     println!("Plot saved to plot_{}_vs_{}.html", data1.get_name(), data2.get_name());
// }

use plotly::{Plot, Scatter, Layout};
use plotly::common::Mode;
use plotly::layout::Axis;

use crate::decay_data::{DecayData, DecayItem};

/// 绘制多条衰变曲线，每条曲线图例显示 tau_b / tau_a 比例
pub fn plot_multiple_decay(all_data: &Vec<(DecayData, f64)>) {
    let mut plot = Plot::new();

    for (data, ratio) in all_data.iter() {
        let seq = data.get_sequence();
        let t = DecayItem::extract_t(seq);
        let na = DecayItem::extract_na(seq);
        let nb = DecayItem::extract_nb(seq);

        // A 核
        let trace_na = Scatter::new(t.clone(), na)
            .name(&format!("A nucleus (τ_b/τ_a = {:.2})", ratio))
            .mode(Mode::LinesMarkers);
        plot.add_trace(trace_na);

        // B 核
        let trace_nb = Scatter::new(t.clone(), nb)
            .name(&format!("B nucleus (τ_b/τ_a = {:.2})", ratio))
            .mode(Mode::LinesMarkers);
        plot.add_trace(trace_nb);
    }

    plot.set_layout(
        Layout::new()
            .title("Decay Curves with Different τ_b/τ_a")
            .x_axis(Axis::new().title("Time"))
            .y_axis(Axis::new().title("Number of Nuclei")),
    );

    std::fs::write("plot_multiple_decay.html", plot.to_html()).unwrap();
    println!("Plot saved to plot_multiple_decay.html");
}
