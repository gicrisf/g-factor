#[derive(Copy, Clone)]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
}

impl Color {
    pub fn rgb(r: f64, g: f64, b: f64) -> Color {
        Color {
            r: r / 255.0,
            g: g / 255.0,
            b: b / 255.0,
        }
    }
    // TODO: hex

    // Palettes TODO mother function palette
    pub fn original(name: &str) -> Color {
        match name {
            "DarkCyan" => Color::rgb(1.0, 46.0, 64.0),
            "LightCyan" => Color::rgb(79.0, 134.0, 140.0),
            _ => Color::rgb(0.0, 0.0, 0.0),  // Return black if unknown
        }
    }

    pub fn solarized(name: &str) -> Color {
        match name {
            "White" => Color::rgb(238.0, 232.0, 213.0),
            "Orange" => Color::rgb(203.0, 75.0, 22.0),
            "Cyan" => Color::rgb(79.0, 134.0, 140.0),
            "Violet" => Color::rgb(108.0, 113.0, 196.0),
            "Yellow" => Color::rgb(181.0, 137.0, 0.0),
            _ => Color::rgb(0.0, 0.0, 0.0),  // Return black if unknown
        }
    }

    fn as_tuple(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

#[derive(Clone)]
pub struct Spectra { pub exp: Vec<f64>, pub teor: Vec<f64> }
pub struct Axis { ax: Vec<f64>, exists: bool }

impl Axis {
    pub fn get_max(&self) -> f64 { self.ax.iter().fold(0.0, |a: f64, &b| a.max(b)) }
    pub fn get_min(&self) -> f64 { self.ax.iter().fold(std::f64::INFINITY, |a: f64, &b| a.min(b)) }

    pub fn get_abs_max(&self) -> f64 {
        let max = self.get_max();
        let min = self.get_min();

        let abs_max: f64;
        if max > (- min) {
            abs_max = max;
        } else {
            abs_max = min;
        }

        abs_max
    }

    pub fn from(vec: Vec<f64>) -> Axis {
        let exists: bool = !(vec.is_empty());

        Axis {
            ax: vec,
            exists,
        }
    }

    pub fn x_from_y(y: &Axis) -> Axis {
        Axis {
            ax: y.ax.iter().enumerate().map(|(i, _)| {i as f64}).collect(),
            exists: true,
        }
    }
}

pub struct Sizes {  // Pub?
    max_y: f64,
    max_x: f64,
    area_x: f64,
    area_y: f64,
}

#[derive(Copy, Clone)]  // Implement copy trait
pub struct Chart {
    pub width: f64,
    pub height: f64,
    pub padding: f64,
    pub background_color: Color,
    pub color_exp: Color,
    pub color_teor: Color,
    pub line_width: f64,
}

impl Chart {
    pub fn normalize(&self, size: &Sizes, x: &Axis, y: &Axis) -> Vec<(f64, f64, f64)> {
        let data_points = x.ax.iter().zip(y.ax.iter());
        let size_x = size.area_x / size.max_x;
        let size_y = size.area_y / size.max_y;

        data_points.map(|(x, y)| {
            (
                self.padding + size_x * *x,
                self.padding + size.area_y - size_y * *y,
                *y,
            )
        })
        .collect()
    }

    fn plot_line(&self, cr: &cairo::Context, data: Vec<(f64, f64, f64)>) {
        let data_window = data.windows(2);
        for points in data_window {
            let source = points[0];
            let target = points[1];

            // draw the line
            cr.move_to(source.0, source.1);
            cr.line_to(target.0, target.1);
            cr.stroke();
        }
    }

    pub fn draw_spectra(&self, cr: &cairo::Context, spectra: Spectra) -> gtk::Inhibit {
        let exp =  Axis::from(spectra.exp);
        let teor = Axis::from(spectra.teor);

        let (area_x, area_y): (f64, f64) = (
            self.width - self.padding * 2.0,
            self.height / 2.0 - self.padding * 2.0
        );
        let (asc, max_x, max_y): (Axis, f64, f64);

        if exp.exists{
            asc = Axis::x_from_y(&exp);
            max_x = asc.get_max();
            max_y = exp.get_abs_max();
        } else {
            asc = Axis::x_from_y(&teor);
            max_x = asc.get_max();
            max_y = teor.get_abs_max();
        }

        let sizes = Sizes { area_x, area_y, max_x, max_y };

        // Paint background
        let (a, b, c) = self.background_color.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.paint();
        // Draw exp
        let (a, b, c) = self.color_exp.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.set_line_width(self.line_width);

        if exp.exists {
            // Plot exp
            let data: Vec<(f64, f64, f64)> = self.normalize(&sizes, &asc, &exp);
            self.plot_line(cr, data);

            // Change color
            let (a, b, c) = self.color_teor.as_tuple();
            cr.set_source_rgb(a, b, c);
        }

        if teor.exists {
            let data: Vec<(f64, f64, f64)> = self.normalize(&sizes, &asc, &teor);
            self.plot_line(cr, data);
        }

        gtk::Inhibit(false)
    }  // draw spectra
}
