// Draft
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;  // WARNING: unused
use gtk::prelude::*;
use std::collections::HashMap;

use crate::ent::{Radical};

pub struct Tab {
    pub tab_box: gtk::Box,
    button: gtk::Button,
}

impl Tab {
    pub fn new(label: &str) -> Self {
        let close_image = gtk::Image::from_icon_name(Some("window-close"), gtk::IconSize::Button);

        let label = gtk::Label::new(Some(label));

        let button = gtk::Button::new();
        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(true);
        button.add(&close_image);

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
}

pub struct EntryPar {
    buffer: gtk::EntryBuffer,
    widget: gtk::Entry,
}

impl EntryPar {
    pub fn new(val: f64,
        field: String, sub_field: String, position: (gtk::Grid, i32, i32)) -> Self {

        let buffer =  gtk::EntryBuffer::new(Some(&val.to_string()));  // make buffer
        let widget = gtk::Entry::with_buffer(&buffer);  // make entry widget
        position.0.attach(&widget, position.1, position.2, 1, 1);  // attach widget to the grid

        Self { buffer, widget }  // return
    }
}

pub struct Content { pub rad_box: gtk::Box }

impl Content {
    fn new(rad_idx: usize,
        rad: &Radical, rad_sender: glib::Sender<(usize, Radical)>) -> Self {

        let builder = gtk::Builder::from_string(include_str!("settings.glade"));
        let rad_box: gtk::Box = builder.get_object("rad_box").expect("err building rad_box");
        let rad_grid: gtk::Grid = builder.get_object("rad_grid").expect("err building rad_grid");
        let nuc_grid: gtk::Grid = builder.get_object("nucs_grid").expect("err building nuc_grid");

        let rad_clone = rad.clone();  // RAD CLONE 00

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

            let rad_sender_clone = rad_sender.clone();  // SENDER CLONE 00
            let buffer = entrypar.buffer.clone();  // BUFFER CLONE 00
            let rad_clone_1 = rad_clone.clone(); // RAD CLONE 01

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

                let mut rad_clone_2 = rad_clone_1.clone();  // RAD CLONE 02

                match par_name_clone {
                   ("amount", "val") => rad_clone_2.amount.val = new_val,
                   ("amount", "var") => rad_clone_2.amount.var = new_val,
                   ("dh1", "val") => rad_clone_2.dh1.val = new_val,
                   ("dh1", "var") => rad_clone_2.dh1.var = new_val,
                   ("lwa", "val") => rad_clone_2.lwa.val = new_val,
                   ("lwa", "var") => rad_clone_2.lwa.var = new_val,
                   ("lrtz", "val") => rad_clone_2.lrtz.val = new_val,
                   ("lrtz", "var") => rad_clone_2.lrtz.var = new_val,
                   _ => panic!("unknown field"),
               };

               // Generate and send new radical!
               let new_rad = (rad_idx, rad_clone_2);
               rad_sender_clone.send(new_rad);  // ERROR MANAGEMENT NEEDED
            }); // Connect changed
        }  // for radpar name in radpas_names

        // Nucs
        let rad_clone_1 = rad_clone.clone();  // RAD CLONE 01 bis

        for (nuc_idx, nuc) in rad.nucs.iter().enumerate() {

            let rad_clone_2 = rad_clone_1.clone();  // RAD CLONE 02

            let nucpar_names = [
                ("eqs", "val"),
                ("spin", "val"),
                ("hpf", "val"),
                ("hpf", "var")
                ];

            for (par_idx, par_name) in nucpar_names.iter().enumerate() {

                let rad_clone_3 = rad_clone_2.clone();  // RAD CLONE 03
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

                let rad_sender_clone = rad_sender.clone();  // SENDER CLONE 00 bis
                let buffer = entrypar.buffer.clone();  // BUFFER CLONE 00 bis

                &entrypar.widget.connect_changed(move|_| {

                    let mut rad_clone_4 = rad_clone_3.clone();  // RAD CLONE 04

                    let new_val: Result<f64, std::num::ParseFloatError> =
                        buffer
                        .get_text()  // from buffer
                        .as_str()  // parse as string
                        .parse();  // parse as f64

                    let new_val: f64 = match new_val {
                        Ok(val) => val,
                        Err(error) => 0.0,  // TODO: get old value and reset entry
                    };

                    match par_name_clone {
                       ("eqs", "val") => rad_clone_4.nucs[nuc_idx].eqs.val = new_val,
                       ("spin", "val") => rad_clone_4.nucs[nuc_idx].spin.val = new_val,
                       ("hpf", "val") => rad_clone_4.nucs[nuc_idx].hpf.val = new_val,
                       ("hpf", "var") => rad_clone_4.nucs[nuc_idx].hpf.var = new_val,
                       _ => panic!("unknown field"),
                   };

                   // Generate and send new radical!
                   let new_rad = (rad_idx, rad_clone_4);
                   rad_sender_clone.send(new_rad);  // ERROR MANAGEMENT NEEDED
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
    pub fn new(rads: Vec<Radical>, settings_sender: glib::Sender<Vec<Radical>>) -> Self {
        let window: gtk::Window = gtk::Window::new(gtk::WindowType::Toplevel);
        let notebook = gtk::Notebook::new();

        // Store gtk tabs and inner grids with rad values in a Hashmap
        // Must put this variable in the main struct ! ALERT
        // Store Rad index VS gtk Grid
        let mut boxes: HashMap<usize, gtk::Box> = HashMap::new();  // utile a qualcuno?

        let (rad_sender, rad_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let settings_sender_clone = settings_sender.clone();
        let mut rad_new = rads.clone();

        for (idx, rad) in rads.iter().enumerate() {
            let tab = Tab::new(&("Radical ".to_owned() + &idx.to_string()));
            let content = Content::new(idx, rad, rad_sender.clone());
            boxes.insert(idx, content.rad_box);  // utile a qualcuno?
            notebook.append_page(&boxes[&idx], Some(&tab.tab_box));
        }

        window.add(&notebook);
        window.set_title("g Factor - Radicals");
        window.set_position(gtk::WindowPosition::Center);
        window.show_all();

        // Receiver aggregate radicals and sends to main program

        rad_receiver.attach(None, move |new_rad: (usize, Radical)| {
            // println!("{}", serde_json::to_string(&new_rad).unwrap());
            rad_new[new_rad.0] = new_rad.1;
            settings_sender.send(rad_new.clone());  // ERROR MANAGEMENT!
            glib::Continue(true)
        });

        Self { window }
    }
}
