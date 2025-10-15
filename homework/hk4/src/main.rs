use std::io;
mod decay_data;
use decay_data::{DecayData, DecayItem, DecayConfig, Iteration, IntegralMethod};

mod draw_decay_data;
use draw_decay_data::plot_multiple_decay;

fn main() {

    let tau_a = 10.0; // 固定 τ_a
    
    // let init_uranium_decay_item: DecayItem = DecayItem::new(1000.0, 0.0);
    println!("请输入多个 τ_b / τ_a 比例，用空格分开:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let ratios: Vec<f64> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse::<f64>().unwrap())
        .collect();

    let decay_time = 200.0; // 可手动输入

    let init_item = DecayItem::new(1000.0, 0.0, 0.0);
    let decay_config = DecayConfig::new("A->B")
        .set_decay_constant(tau_a)
        .set_dt(1.0)
        .set_decay_time(decay_time)
        .set_init_number_of_atoms(1000)
        .build();

    let all_data_with_ratio: Vec<(DecayData, f64)> = ratios
        .iter()
        .map(|&ratio| {
            let tau_b = tau_a * ratio;
            let mut data = DecayData::new(&decay_config, &init_item);
            data.iterate(IntegralMethod::euler_double_decay(&decay_config, tau_b));
            (data, ratio)
        })
        .collect();

    plot_multiple_decay(&all_data_with_ratio);
}


//     let init_decay_item1: DecayItem = DecayItem::new(1000.0, 0.0, 0.0);

//     // let uranium_decay_config: DecayConfig = DecayConfig::new("Uranium")
//     //     .set_decay_constant(10.0)
//     //     .set_dt(1.0)
//     //     .set_decay_time(20.0)
//     //     .set_init_number_of_atoms(1000)
//     //     .build();
//     let decay_config1: DecayConfig = DecayConfig::new("A→B")
//         .set_decay_constant(10.0)       // tau_a
//         .set_dt(1.0)
//         .set_decay_time(20.0)
//         .set_init_number_of_atoms(1000)
//         .build();

//     // let mut uranium_decay_data: DecayData = DecayData::new(&uranium_decay_config, &init_uranium_decay_item);
//     let mut decay_data1: DecayData = DecayData::new(&decay_config1, &init_decay_item1);
//     // dbg!(&uranium_decay_config);
//     dbg!(&decay_config1);
//     // dbg!(uranium_decay_data.get_decay_config());
//     dbg!(decay_data1.get_decay_config());
//     // uranium_decay_data.iterate(IntegralMethod::euler(&uranium_decay_config));
//     let decay_data1 = simulate(&decay_config1, &init_decay_item1, tau_b);    
//     // dbg!(uranium_decay_data.get_sequence());


//     // let init_uranium_decay_item_2: DecayItem = DecayItem::new(1000.0, 0.0);
//     let init_decay_item2: DecayItem = DecayItem::new(1000.0, 0.0, 0.0);
//     // let uranium_decay_config_2: DecayConfig = 
//         // uranium_decay_config.clone()
//     let decay_config2: DecayConfig = 
//         decay_config1.clone()
//         .set_decay_constant(20.0)
//         .build();

//     // let mut uranium_decay_data_2: DecayData = DecayData::new(&uranium_decay_config_2, &init_uranium_decay_item_2); 
//     // uranium_decay_data_2.iterate(IntegralMethod::euler(&uranium_decay_config_2));
//     let decay_data2 = simulate(&decay_config2, &init_decay_item2, tau_b);

//     // dbg!(uranium_decay_data_2.get_sequence());
//     // let x: Vec<f64> = DecayItem::extract_x(uranium_decay_data_2.get_sequence());

//     // draw(&uranium_decay_data_2);
//     // draw(&uranium_decay_data);
//     // plot_two_decay(&uranium_decay_data, &uranium_decay_data_2);
//     draw(&decay_data1);
//     // draw(&decay_data2);
//     plot_two_decay(&decay_data1, &decay_data2);
// }
