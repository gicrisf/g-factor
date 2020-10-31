use std::iter::Zip;

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

    fn as_tuple(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

// Original Colors
// Color::rgb(1.0, 46.0, 64.0),  // Dark Cyan
// Color::rgb(79.0, 134.0, 140.0),  // Light Cyan

// Colors from solarized palette
// TODO: make enum from this
// Color::rgb(238.0, 232.0, 213.0),  // White
// Color::rgb(203.0, 75.0, 22.0),  // Orange
// Color::rgb(79.0, 134.0, 140.0),  // Cyan
// Color::rgb(108.0, 113.0, 196.0),  // Violet
// Color::rgb(181.0, 137.0, 0.0),  // Yellow

#[derive(Clone)]
pub struct Spectra {
    pub exp: Vec<f64>,
    pub teor: Vec<f64>,
}

impl Spectra {
    pub fn auto_x(&self) -> Vec<f64> {
        // Alternative:
        // 1. Get length of ax;
        // 2. Create range array;
        // 3. Make a vector of f64 from that.
        self.exp.iter().enumerate().map(|(i, _)| {i as f64}).collect()
    }

    pub fn ax_limits(&self) -> f64 {
        let max = self.exp.iter().fold(0.0, |a: f64, &b| a.max(b));
        let min = self.exp.iter().fold(std::f64::INFINITY, |a: f64, &b| a.min(b));

        let y_abs_max: f64;
        if max > (- min) {
            y_abs_max = max;
        } else {
            y_abs_max = min;
        }

        y_abs_max
    }

    // pub fn data_points(&self) -> () {}
}

#[derive(Clone)]
struct Area {
    max_y: f64,
    max_x: f64,
    area_0: f64,
    area_1: f64,
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
    pub fn normalize(&self, size: &Area, x_vec: &Vec<f64>, y_vec: Vec<f64>) -> Vec<(f64, f64, f64)> {
        let data_points = x_vec.iter().zip(y_vec.iter());
        let size_x = size.area_0 / size.max_x;
        let size_y = size.area_1 / size.max_y;

        data_points.map(|(x, y)| {
            (
                self.padding + size_x * *x,
                self.padding + size.area_1 - size_y * *y,
                *y,
            )
        })
        .collect()
    }

    pub fn draw(&self, cr: &cairo::Context, ord: Vec<f64>) -> gtk::Inhibit {
        let chart_area = (self.width - self.padding * 2.0, self.height / 2.0 - self.padding * 2.0);
        let asc: Vec<f64> = auto_x(&ord);

        // Get highest absolute value in the y axis and set relative sizes of axes
        let y_abs_max: f64;
        let y_max = get_ax_max(&ord);
        let y_min = get_ax_min(&ord);

        if y_max > (- y_min) {
            y_abs_max = y_max;
        } else {
            y_abs_max = y_min;
        }

        let size_x = chart_area.0 / get_ax_max(&asc);
        let size_y = (chart_area.1) / y_abs_max;

        // Normalize data
        let data_points = asc.iter().zip(ord.iter());

        let normalized_data: Vec<(f64, f64, f64)> = data_points
        .map(|(x, y)| {
            (
                self.padding + size_x * *x,
                self.padding + chart_area.1 - size_y * *y,
                *y,
            )
        })
        .collect();

        // Paint background
        let (a, b, c) = self.background_color.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.paint();

        // Draw line
        let (a, b, c) = self.color_exp.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.set_line_width(self.line_width);

        let data_window = normalized_data.windows(2);
        for points in data_window {
            let source = points[0];
            let target = points[1];

            // draw the line
            cr.move_to(source.0, source.1);
            cr.line_to(target.0, target.1);
            cr.stroke();
        }

        gtk::Inhibit(false)
    }  // draw

    pub fn draw_spectra(&self, cr: &cairo::Context, spectra: Spectra) -> gtk::Inhibit {
        let asc: Vec<f64> = spectra.auto_x();

        let area = Area {
            area_0: self.width - self.padding * 2.0,
            area_1: self.height / 2.0 - self.padding * 2.0,
            max_x: get_ax_max(&asc),
            max_y: spectra.ax_limits(),
        };

        // Paint background
        let (a, b, c) = self.background_color.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.paint();

        let normalized_data_exp: Vec<(f64, f64, f64)> = self.normalize(&area, &asc, spectra.exp);

        // Draw lines
        let (a, b, c) = self.color_exp.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.set_line_width(self.line_width);

        let data_window0 = normalized_data_exp.windows(2);
        for points in data_window0 {
            let source = points[0];
            let target = points[1];

            // draw the line
            cr.move_to(source.0, source.1);
            cr.line_to(target.0, target.1);
            cr.stroke();
        }

        let normalized_data_teor: Vec<(f64, f64, f64)> = self.normalize(&area, &asc, spectra.teor);

        let (a, b, c) = self.color_teor.as_tuple();
        cr.set_source_rgb(a, b, c);
        cr.set_line_width(self.line_width);

        let data_window1 = normalized_data_teor.windows(2);
        for points in data_window1 {
            let source = points[0];
            let target = points[1];

            // draw the line
            cr.move_to(source.0, source.1);
            cr.line_to(target.0, target.1);
            cr.stroke();
        }

        gtk::Inhibit(false)
    }  // draw spectra
}

pub fn get_ax_max(ax: &Vec<f64>) -> f64 {
    ax.iter().fold(0.0, |a: f64, &b| a.max(b))
}

pub fn get_ax_min(ax: &Vec<f64>) -> f64 {
    ax.iter().fold(std::f64::INFINITY, |a: f64, &b| a.min(b))
}

pub fn auto_x(ax: &Vec<f64>) -> Vec<f64> {
    ax.iter().enumerate().map(|(i, _)| {i as f64}).collect()
}
