extern crate glib;
extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;

// use std::sync::mpsc;
// use std::sync::mpsc::{Sender, Receiver};
// use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
// use std::thread;

mod io;
mod plt;
// mod ent;
// mod sim;
mod ui;

use crate::ui::ui::{Gui};
use crate::plt::{Chart, Color};
// use crate::ent::{Radical};
// use crate::sim::{Simulator};

fn main_app(app: &gtk::Application) {
    /*
    // Sender/Receiver
    let (tx, rx) = mpsc::channel();

    // Simulator
    let mut sim = Simulator::new(tx);
    let locked_sim = Arc::new(Mutex::new(sim));
    let cloned_sim = Arc::clone(&locked_sim);

    // Simulator thread
    thread::spawn(move || {
        start_tokio(locked_sim, rx);
    });
    */

    let chart = Chart {
        width: 1000.0,
        height: 600.0,
        padding: 0.0,
        background_color: Color::original("DarkCyan"),
        color_exp: Color::original("LightCyan"),
        color_teor: Color::solarized("Orange"),
        line_width: 1.25,
    };

    // Start GUI and connect it with signals and actions;
    let gui = Gui::new(chart);
    gui.win.set_application(Some(app));

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


// #[tokio::main]
// async fn start_tokio<'a>(sim: Arc<Mutex<Simulator>>, rx: Receiver<f64>) {}
