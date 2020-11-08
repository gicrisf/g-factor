extern crate glib;
extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;

// use std::sync::{Arc, Mutex};
use std::thread;

mod io;
mod plt;
mod ent;
mod sim;
mod ui;

use crate::ui::ui::{Gui};
use crate::plt::{Chart, Color};
use crate::sim::{Simulator};
use crate::ent::{Radical};

fn main_app(app: &gtk::Application) {

    let sim = Simulator::new();

    // Simulator thread
    let sim1 = sim.clone();
    let _sim_thread = thread::spawn(move || {
        let mut m = sim1.teor.lock().unwrap();
        *m = sim1.calcola(vec![Radical::probe(), Radical::electron()]);
    });

    let chart = Chart {
        width: 1000.0,
        height: 600.0,
        padding: 0.0,
        background_color: Color::original("DarkCyan"),
        color_exp: Color::original("LightCyan"),
        color_teor: Color::solarized("Orange"),
        line_width: 1.25,
    };

    // Start GUI
    let gui = Gui::new(chart, sim);
    gui.win.set_application(Some(app));
    gui.connect_buttons();

    // Create Menu
    let menu_bar = gui.build_menu();
    app.set_menubar(Some(&menu_bar));

    // Add open action to menu
    let open_action = gui.open_action();
    app.add_action(&open_action);
}

fn main() {
    gtk::init().expect("Failed to initialize GTK.");
    let application = gtk::Application::new(Some("splitted.example"), Default::default())
        .expect("Failed to initialize application.");
    application.connect_activate(move |app| { main_app(app) });
    application.run(&args().collect::<Vec<_>>());
}
