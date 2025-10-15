
pub struct DecayData {
    sequence: Vec<DecayItem>,
    decay_config: DecayConfig,
    sequence_length: usize,
}
#[derive(Clone, Debug)]
// pub struct DecayItem {
//     x: f64,
//     t: f64,
// }
pub struct DecayItem {
    pub na: f64, // A核
    pub nb: f64, // B核
    pub t: f64,
}

// impl DecayItem {
//     pub fn new(x: f64, t: f64) -> Self {
//         DecayItem { x, t }
//     }
//     pub fn get_x(&self) -> f64 {
//         self.x
//     }
//     pub fn get_t(&self) -> f64 {
//         self.t
//     }
//     pub fn extract_x(v: &Vec<DecayItem>) -> Vec<f64> {
//         v.iter().map(|x| x.get_x()).collect()
//     }
//     pub fn extract_t(v: &Vec<DecayItem>) -> Vec<f64> {
//         v.iter().map(|x| x.get_t()).collect()
//     }
//     pub fn extract_xt(v: &Vec<DecayItem>) -> (Vec<f64>, Vec<f64>) {
//         (Self::extract_t(v), Self::extract_x(v))
//     }
// }

impl DecayItem {
    pub fn new(na: f64, nb: f64, t: f64) -> Self {
        DecayItem { na, nb, t }
    }
    pub fn get_na(&self) -> f64 { self.na }
    pub fn get_nb(&self) -> f64 { self.nb }
    pub fn get_t(&self) -> f64 { self.t }
    pub fn extract_na(v: &Vec<DecayItem>) -> Vec<f64> {
        v.iter().map(|item| item.get_na()).collect()
    }

    pub fn extract_nb(v: &Vec<DecayItem>) -> Vec<f64> {
        v.iter().map(|item| item.get_nb()).collect()
    }

    pub fn extract_t(v: &Vec<DecayItem>) -> Vec<f64> {
        v.iter().map(|item| item.get_t()).collect()
    }

    pub fn extract_na_nb_t(v: &Vec<DecayItem>) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        (Self::extract_na(v), Self::extract_nb(v), Self::extract_t(v))
    }
}

impl DecayData {
    pub fn new(decay_config: &DecayConfig, decay_item: &DecayItem) -> Self {
        //let _length: usize = (decay_config.decay_time / decay_config.dt) as usize;
        DecayData {
            sequence: vec![decay_item.clone()],
            decay_config: decay_config.clone(),
            sequence_length: (decay_config.decay_time / decay_config.dt) as usize,
            //sequence_length: _length,
        }
    }
    pub fn get_decay_config(&self) -> &DecayConfig {
        &self.decay_config
    }
    pub fn get_sequence(&self) -> &Vec<DecayItem> {
        &self.sequence
    }
    pub fn get_name(&self) -> &str {
        self.decay_config.get_atom_name()
    }
}

#[derive(Clone, Debug)]
pub struct DecayConfig {
    atom_name: String,
    decay_constant: f64,
    dt: f64,
    decay_time: f64,
    init_number_of_atoms: usize,
    //euler: DecayItem,
    //number_of_steps: usize,
}

impl DecayConfig {
    pub fn new(atom_name: &str) -> Self {
        DecayConfig {
            atom_name: atom_name.to_string(),
            decay_constant: 0.1,
            dt: 0.01,
            decay_time: 10.0,
            init_number_of_atoms: 1000,
        }
    }
    pub fn set_decay_constant(mut self, decay_constant: f64) -> Self {
        self.decay_constant = decay_constant;
        self
    }
    pub fn set_dt(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }
    pub fn set_init_number_of_atoms(mut self, init_number_of_atoms: usize) -> Self {
        self.init_number_of_atoms = init_number_of_atoms;
        self
    }
    pub fn set_decay_time(mut self, decay_time: f64) -> Self {
        self.decay_time = decay_time;
        self
    }
    pub fn build(self) -> Self {
        let _number_of_stesp: usize = (self.decay_time / self.dt) as usize;
        DecayConfig {
            //number_of_atoms: vec![self.init_number_of_atoms],
            //time_series: vec![0.0],
            atom_name: self.atom_name,
            decay_constant: self.decay_constant,
            dt: self.dt,
            decay_time: self.decay_time,
            init_number_of_atoms: self.init_number_of_atoms,
        }
    }
}

impl DecayConfig {
    pub fn get_atom_name(&self) -> &str {
        &self.atom_name
    }
}

pub trait Iteration {
    fn iterate(&mut self, integral_method: impl Fn(&DecayItem) -> DecayItem);
    fn get_sequence_length(&self) -> usize;
    fn get_last_item(&self) -> &DecayItem;
}

impl Iteration for DecayData {
    fn iterate(&mut self, integral_method: impl Fn(&DecayItem) -> DecayItem) {
        for _i in 0..self.get_sequence_length() {
            //self.sequence.push(self.decay_config.euler(self.get_last_item()));
            let _item = integral_method(self.get_last_item());
            self.sequence.push(_item);
        }
    }
    fn get_sequence_length(&self) -> usize {
        self.sequence_length
    }
    fn get_last_item(&self) -> &DecayItem {
        //&self.sequence[self.get_sequence_length() - 1]
        self.sequence.last().unwrap()
    }
}

// pub struct IntegralMethod;

// impl IntegralMethod {
//     pub fn euler(decay_config: &DecayConfig) -> impl Fn(&DecayItem) -> DecayItem {
//         |v: &DecayItem| {
//             let _decay_speed: f64 = -v.x / decay_config.decay_constant;
//             DecayItem {
//                 x: v.x + _decay_speed * decay_config.dt,
//                 t: v.t + decay_config.dt,
//             }
//         }
//     }
// }

pub struct IntegralMethod;

impl IntegralMethod {
    pub fn euler_double_decay(config: &DecayConfig, tau_b: f64) -> impl Fn(&DecayItem) -> DecayItem {
        let tau_a = config.decay_constant;
        let dt = config.dt;
        move |v: &DecayItem| {
            let na_new = v.na - dt * v.na / tau_a;
            let nb_new = v.nb + dt * (v.na / tau_a - v.nb / tau_b);
            DecayItem { na: na_new, nb: nb_new, t: v.t + dt }
        }
    }
}