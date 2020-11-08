// Draft
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;

use std::collections::HashMap;

use crate::ent::{Radical};

pub struct Tab {
    pub tab_box: gtk::Box,
    pub button: gtk::Button,
}

impl Tab {
    pub fn new(label: &str) -> Self {
        let close_image = gtk::Image::from_icon_name(Some("window-close"), gtk::IconSize::Button);
        let button = gtk::Button::new();
        let label = gtk::Label::new(Some(label));
        let tab_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        button.set_relief(gtk::ReliefStyle::None);
        button.set_focus_on_click(false);
        button.add(&close_image);

        tab_box.pack_start(&label, false, false, 0);
        tab_box.pack_start(&button, false, false, 0);
        tab_box.show_all();

        Self {
            tab_box,
            button
        }
    }
}

pub struct Content {
    pub rad_box: gtk::Box,
    rad_grid: gtk::Grid,
    nucs_grid: gtk::Grid,
    val_entries: [gtk::Entry; 4],
    var_entries: [gtk::Entry; 4],
    nuc_entries: Vec<[gtk::Entry; 4]>,
}

impl Content {
    pub fn new(rad: &Radical) -> Self {
        let builder = gtk::Builder::from_string(include_str!("settings.glade"));
        let rad_box: gtk::Box = builder.get_object("rad_box").expect("err building rad_box");
        let rad_grid: gtk::Grid = builder.get_object("rad_grid").expect("err building rad_grid");
        let nucs_grid: gtk::Grid = builder.get_object("nucs_grid").expect("err building nucs_grid");

        // Vals
        let radpar_vals: [f64; 4] = [rad.amount.val, rad.dh1.val, rad.lwa.val, rad.lrtz.val];
        let val_buffers: [gtk::EntryBuffer; 4] = make_buffers(radpar_vals);
        let val_entries: [gtk::Entry; 4] = make_entries(val_buffers.clone());

        for (idx, buffer) in radpar_vals.iter().enumerate() {
            rad_grid.attach(&val_entries[idx], 1, idx as i32 + 1, 1, 1);
            let bufclones = val_buffers.clone();
            &val_entries[idx].connect_changed(move|_| {
                let mut radpar_vals_clone = radpar_vals.clone();
                let pie: f64 = bufclones[idx].get_text().as_str().parse().unwrap();  // MANAGE ERRORS!
                radpar_vals_clone[idx] = pie;
                println!("{:?}", radpar_vals_clone);
            });
        }

        // Vars
        let radpar_vars: [f64; 4] = [rad.amount.var, rad.dh1.var, rad.lwa.var, rad.lrtz.var];
        let var_buffers: [gtk::EntryBuffer; 4] = make_buffers(radpar_vars);
        let var_entries: [gtk::Entry; 4] = make_entries(var_buffers.clone());

        for (idx, buffer) in var_buffers.iter().enumerate() {
            rad_grid.attach(&var_entries[idx], 2, idx as i32 + 1, 1, 1);
        }

        // Nuclei
        let mut nuc_entries = Vec::new();
        for (idx, nuc) in rad.nucs.iter().enumerate() {
            let nucpars: [f64; 4] = [nuc.eqs.val, nuc.spin.val, nuc.hpf.val, nuc.hpf.var];
            let nucpar_buffers: [gtk::EntryBuffer; 4] = make_buffers(nucpars);
            let nucpar_entries: [gtk::Entry; 4] = make_entries(nucpar_buffers.clone());

            for (idx, buffer) in nucpar_buffers.iter().enumerate() {
                nucs_grid.attach(&nucpar_entries[idx], idx as i32, 1, 1, 1);
            }

            nuc_entries.push(nucpar_entries);
        }

        Self {
            rad_box,
            rad_grid,
            nucs_grid,
            val_entries,
            var_entries,
            nuc_entries,
        }
    }  // new
}

pub fn create_page(idx: &usize, rad: &Radical) -> (Content, Tab) {
    let tab = Tab::new(&idx.to_string());
    let content = Content::new(rad);
    (content, tab)
}

pub fn make_buffers(values: [f64; 4]) -> [gtk::EntryBuffer; 4] {
    let mut buffers = [
        gtk::EntryBuffer::new(Some("")),
        gtk::EntryBuffer::new(Some("")),
        gtk::EntryBuffer::new(Some("")),
        gtk::EntryBuffer::new(Some("")),
    ];

    for (idx, value) in values.iter().enumerate() {
        buffers[idx] = gtk::EntryBuffer::new(Some(&value.to_string()));
    }

    buffers
}

pub fn make_entries(buffers: [gtk::EntryBuffer; 4]) -> [gtk::Entry; 4] {
    let mut entries: [gtk::Entry; 4] = [
        gtk::Entry::new(),
        gtk::Entry::new(),
        gtk::Entry::new(),
        gtk::Entry::new(),
        ];

    for (idx, buffer) in buffers.iter().enumerate() {
        entries[idx] = gtk::Entry::with_buffer(buffer);
    }

    entries
}
