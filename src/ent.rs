use rand::prelude::*;
use serde::{Serialize, Deserialize};

// Param
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Param {
    pub val: f64,  // Value; starts with 0.0
    pub var: f64,  // Variation; starts with: 0.0
}

impl Param {
    pub fn set(val: f64, var: f64) -> Param {
        Param { val, var, }
    }

    pub fn randomize(&self) -> Param {
        if self.var != 0.0 {
            let mut rng = thread_rng();
            let random: f64 = rng.gen();  // random number in range [0, 1)
            let rnd = 2.0*random-1.0;
            let new_val = self.val + rnd * self.var;
            return Param { val: new_val, var: self.var }
        } else {
            return Param { val: self.val, var: self.var }
        }
    }
}

// Nucleus
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Nucleus {
    pub spin: Param,  // Nuclear spin;
    pub hpf: Param,  // Hyperfine constant;
    pub eqs: Param,  // Equivalent nucleus; Should be u8!
}

impl Nucleus {
    pub fn set(spin: f64, hpf: f64, eqs: f64) -> Nucleus {
        Nucleus {
            spin: Param::set(spin, 0.0),
            hpf: Param::set(hpf, 0.0),
            eqs: Param::set(eqs, 0.0),
        }
    }
}

// Radical
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Radical {
    pub lwa: Param,  // Line width A
    // pub lwb: Param,
    // pub lwc: Param,
    pub lrtz: Param,  // Lorentzian linewidth parameter (%)
    pub amount: Param,  // Relative amount
    pub dh1: Param,
    pub nucs: Vec<Nucleus>,
}

impl Radical {
    pub fn set(lwa: f64, lrtz: f64, amount: f64, dh1: f64, nucs: Vec<Nucleus>) -> Radical {
        Radical {
            lwa: Param::set(lwa, 0.0),
            lrtz: Param::set(lrtz, 0.0),
            amount: Param::set(amount, 0.0),
            dh1: Param::set(dh1, 0.0),
            nucs,
        }
    }

    // Reset potentially aberrant value returned by MC function;
    pub fn check_pars(mut rad: Radical) -> Radical {
        if rad.lwa.val < 0.0 { rad.lwa.val = 0.0 };
        if rad.lrtz.val < 0.0 { rad.lrtz.val = 0.0 };
        if rad.amount.val < 0.0 { rad.amount.val = 0.0 };
        if rad.lrtz.val > 100.0 { rad.lrtz.val = 100.0 };
        rad
    }

    // Radical without nuclei and standard parameters;
    pub fn electron() -> Radical {
        Radical::set(0.5, 100.0, 100.0, 0.0, Vec::new())
    }

    // Debug function!
    pub fn probe() -> Radical {
        let mut rad = Radical::set(0.5, 100.0, 100.0, 0.0, Vec::new());
        rad.nucs.push(Nucleus::set(1.0, 14.0, 1.0));
        rad
    }
}
