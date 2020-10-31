
    pub fn mc_fit(&self) {
        let mc_rads = self.rads;
        let newteor = self.calcola(mc_rads);  // Basta prendere quello gia' calcolato, no?
        let (mut somma, mut somma1, mut somma2): (f64, f64, f64) = (0.0, 0.0, 0.0);
        let start: usize = 1;
        let fine: usize = self.points as usize + 1;
        self.iters+=1;

        // Randomize Par

        // Start MC
        for j in start..fine {
            somma1 += newteor[j].powi(2);
            somma2 += self.exp[j].abs() * newteor[j].abs();
        }

        let norma: f64;
        if somma1 == 0.0 { norma = 0.0 } else { norma = somma2/somma1 };

        for j in start..fine {
            newteor[j] *= norma;
            let diff = (self.exp[j] - newteor[j]).powi(2);
            somma+=diff;
        }

        let newsigma =(somma/(fine-start) as f64).sqrt();

        for mut rad in mc_rads {
            rad.lwa.randomize();
            rad.amount.randomize();
            rad.lrtz.randomize();
            rad.dh1.randomize();

            for nuc in rad.nucs {
                nuc.hpf.randomize();
            }

            rad = Radical::check_pars(rad);  // is this legal?

            // Conditional reassignment
            if newsigma < self.sigma {
                self.sigma = newsigma;
                self.rads = mc_rads;
            }
        }  // for rad
        self.calcola(self.rads);
    }  // mc
