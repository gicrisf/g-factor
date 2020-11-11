// Draft
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;  // WARNING: unused
use gtk::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ent::{Radical};

pub struct Tab { pub tab_box: gtk::Box, button: gtk::Button }

impl Tab {
    pub fn new_rad(rad_idx: usize, radgen_sender: glib::Sender<(usize, bool)>) -> Self {
        let close_image = gtk::Image::from_icon_name(Some("window-close"), gtk::IconSize::Button);
        let idx_str = "Radical ".to_owned() + &rad_idx.to_string();
        let label = gtk::Label::new(Some(&idx_str));

        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(true);
        button.add(&close_image);

        button.connect_clicked(move |_| {
            let signal = (rad_idx, false);
            radgen_sender.send(signal);
        });  // Connect clicked button

        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        tab_box.set_property_width_request(150 as i32);
        tab_box.pack_start(&label, false, false, 0);
        tab_box.pack_end(&button, false, false, 0);
        tab_box.show_all();

        Self {
            tab_box,
            button
        }
    }

    pub fn tab_add_btn(radgen_sender: glib::Sender<(usize, bool)>) -> Self {
        let add_image = gtk::Image::from_icon_name(Some("list-add"), gtk::IconSize::Button);
        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(false);
        button.add(&add_image);

        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        tab_box.set_property_width_request(40 as i32);
        tab_box.pack_end(&button, true, true, 0);
        tab_box.set_homogeneous(true);
        tab_box.show_all();

        button.connect_clicked(move |_| {
            let signal = (0, true);  // Index is useless here
            radgen_sender.send(signal);
        });  // Connect clicked button

        Self {
            tab_box,
            button
        }
    }
}

pub struct EntryPar { buffer: gtk::EntryBuffer, widget: gtk::Entry }

impl EntryPar {
    pub fn new(val: f64, field: String, sub_field: String, pos: (gtk::Grid, i32, i32)) -> Self {

        let buffer =  gtk::EntryBuffer::new(Some(&val.to_string()));  // make buffer
        let widget = gtk::Entry::with_buffer(&buffer);  // make entry widget
        pos.0.attach(&widget, pos.1, pos.2, 1, 1);  // attach widget to the grid

        Self { buffer, widget }  // return
    }
}

pub struct Content { pub rad_box: gtk::Box }

impl Content {
    fn new(
        rad_idx: usize,
        rad: &Radical,
        // rad_idx, field, subfield, new value
        radpar_sender: glib::Sender<(usize, String, String, f64)>,
        // rad_idx, nuc_idx, field, subfield, new value
        nucpar_sender: glib::Sender<(usize, usize, String, String, f64)>,
    ) -> Self {

        let builder = gtk::Builder::from_string(include_str!("settings.glade"));
        let rad_box: gtk::Box = builder.get_object("rad_box").expect("err building rad_box");
        let rad_grid: gtk::Grid = builder.get_object("rad_grid").expect("err building rad_grid");
        let nuc_grid: gtk::Grid = builder.get_object("nucs_grid").expect("err building nuc_grid");

        let radpar_names = [
            ("amount", "val"), ("amount", "var"),
            ("dh1", "val"), ("dh1", "var"),
            ("lwa", "val"), ("lwa", "var"),
            ("lrtz", "val"), ("lrtz", "var"),
            ];

        for par_name in radpar_names.iter() {
            let par_name_clone = par_name.clone();  // PAR NAME CLONE 00

            let (row, par) = match par_name.0 {
               "amount" => (1, &rad.amount),
               "dh1" => (2, &rad.dh1),
               "lwa" => (3, &rad.lwa),
               "lrtz" => (4, &rad.lrtz),
               _ => panic!("unknown field"),
           };

           let (col, par) = match par_name.1 {
               "val" => (1, par.val),
               "var" => (2, par.var),
               _ => panic!("Non val nor var! WTF"),
           };

            let entrypar = EntryPar::new(
                par,
                String::from(par_name.0),
                String::from(par_name.1),
                (rad_grid.clone(), col, row)
            );

            let buffer = entrypar.buffer.clone();  // BUFFER CLONE
            let radpar_sender_clone = radpar_sender.clone();  // SENDER CLONE
            &entrypar.widget.connect_changed(move|_| {
                let new_val: Result<f64, std::num::ParseFloatError> =
                    buffer
                    .get_text()  // from buffer
                    .as_str()  // parse as string
                    .parse();  // parse as f64

                let new_val: f64 = match new_val {
                    Ok(val) => val,
                    Err(error) => 0.0,  // TODO: get old value and reset entry
                };

               let exp_radpar = (
                   rad_idx,
                   String::from(par_name_clone.0),
                   String::from(par_name_clone.1),
                   new_val
               );
               radpar_sender_clone.send(exp_radpar);
            }); // Connect changed
        }  // for radpar name in radpas_names

        // Nucs
        for (nuc_idx, nuc) in rad.nucs.iter().enumerate() {
            let nucpar_names = [
                ("eqs", "val"),
                ("spin", "val"),
                ("hpf", "val"),
                ("hpf", "var")
                ];

            for (par_idx, par_name) in nucpar_names.iter().enumerate() {
                let par_name_clone = par_name.clone();  // PAR NAME CLONE 00 bis

                let par = match par_name.0 {
                   "eqs" => &nuc.eqs,
                   "spin" => &nuc.spin,
                   "hpf" => &nuc.hpf,
                   _ => panic!("unknown field"),
               };

               let par = match par_name.1 {
                   "val" => par.val,
                   "var" => par.var,
                   _ => panic!("Non val nor var! WTF"),
               };

                let entrypar = EntryPar::new(
                    par,
                    String::from(par_name.0),
                    String::from(par_name.1),
                    (nuc_grid.clone(), par_idx as i32, 1)
                );

                let buffer = entrypar.buffer.clone();  // BUFFER CLONE
                let nucpar_sender_clone = nucpar_sender.clone();  // SENDER CLONE
                &entrypar.widget.connect_changed(move|_| {
                    let new_val: Result<f64, std::num::ParseFloatError> =
                        buffer
                        .get_text()  // from buffer
                        .as_str()  // parse as string
                        .parse();  // parse as f64

                    let new_val: f64 = match new_val {
                        Ok(val) => val,
                        Err(error) => 0.0,  // TODO: get old value and reset entry
                    };

                    let exp_nucpar = (
                        rad_idx,
                        nuc_idx,
                        String::from(par_name_clone.0),
                        String::from(par_name_clone.1),
                        new_val
                    );

                    nucpar_sender_clone.send(exp_nucpar);
                }); // Connect changed
            }  // for name in nucpar names
        }  // for nuc in nucs

        Self { rad_box }
    }  // new
}

pub struct Settings {
    pub window: gtk::Window,
}

impl Settings {
    pub fn new(
        rads: Arc<Mutex<Vec<Radical>>>,
        nucpar_sender: glib::Sender<(usize, usize, String, String, f64)>,
        radpar_sender: glib::Sender<(usize, String, String, f64)>,
        radgen_sender: glib::Sender<(usize, bool)>,
        ) -> Self {

        let window: gtk::Window = gtk::Window::new(gtk::WindowType::Toplevel);
        let notebook = gtk::Notebook::new();

        let rads_guard = Arc::clone(&rads);
        let mut rads_guard_clone = rads_guard.lock().unwrap().clone();

        for (idx, rad) in rads_guard_clone.iter().enumerate() {
            let tab = Tab::new_rad(idx, radgen_sender.clone());
            let content = Content::new(idx, rad, radpar_sender.clone(), nucpar_sender.clone());
            notebook.append_page(&content.rad_box, Some(&tab.tab_box));
        }

        // Append-radical button
        let add_tab = Tab::tab_add_btn(radgen_sender.clone());  // Add radical when clicked
        let add_content = gtk::Box::new(gtk::Orientation::Horizontal, 0);  // void box
        notebook.append_page(&add_content, Some(&add_tab.tab_box));

        // Show window
        window.add(&notebook);
        window.set_title("g Factor - Radicals");
        window.set_position(gtk::WindowPosition::Center);
        window.show_all();
        Self { window }
    }
}
