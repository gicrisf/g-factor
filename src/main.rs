extern crate glib;
extern crate gio;
extern crate gtk;
use glib::clone;

use gtk::prelude::*;
use gio::prelude::*;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::env::args;
use std::thread;
use std::sync::mpsc;

mod io;
mod plt;
mod ent;
mod sim;

use crate::io::{get_from_asciistring};
use crate::plt::{Chart, Color, Spectra};
use crate::ent::{Radical, Nucleus};
use crate::sim::{Simulator};

// Menu
pub fn build_system_menu(application: &gtk::Application) {
    let menu_bar = gio::Menu::new();
    let file_menu = gio::Menu::new();
    file_menu.append(Some("Open"), Some("app.open"));
    menu_bar.append_submenu(Some("File"), &file_menu);
    application.set_menubar(Some(&menu_bar));
}

pub fn add_actions(
    application: &gtk::Application,
    window: &gtk::ApplicationWindow,
    sender: glib::Sender<String>,
) {
    let open = gio::SimpleAction::new("open", None);
    open.connect_activate(clone!(@weak window => move |_, _| {
        // TODO move this to a impl?
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );
        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);

        let sender_open = sender.clone();
        file_chooser.connect_response(move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                let filename = file_chooser.get_filename().expect("Couldn't get filename");
                let file = File::open(&filename).expect("Couldn't open file");
                let mut reader = BufReader::new(file);
                let mut contents = String::new();
                let _ = reader.read_to_string(&mut contents);

                // Send contents to the main Context
                sender_open.send(contents).unwrap();
            }
            file_chooser.close();
        });

        file_chooser.show_all();
    }));

    // We need to add all the actions to the application so they can be taken into account.
    application.add_action(&open);
}

fn build_ui(application: &gtk::Application,
        tx_exp: Sender<Vec<f64>>,
        rx_teor: Arc<Mutex<Receiver<Spectra>>>,
        ) {

    let builder = gtk::Builder::from_string(include_str!("ui.glade"));
    let win: gtk::ApplicationWindow = builder.get_object("application_window").expect("err build win");
    win.set_application(Some(application));

    // Create a simple glib streaming channel
    let (sender, on_open_glib_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // DrawingArea
    let da: gtk::DrawingArea =
        builder.get_object("drawing_area").expect("err building drawing_area");

    let chart = Chart {
        width: 1000.0,
        height: 600.0,
        padding: 0.0,
        background_color: Color::rgb(1.0, 46.0, 64.0),  // Dark Cyan
        color_exp: Color::rgb(79.0, 134.0, 140.0),  // Light Cyan
        color_teor: Color::rgb(203.0, 75.0, 22.0),  // Orange
        line_width: 1.25,
    };

    let da1 = da.clone();  // Pass to the next function

    // Opening a file...
    on_open_glib_receiver.attach(None, move |msg: String| {
        let new_exp: &str = &msg[..];  // String to &str
        let new_exp = get_from_asciistring(new_exp);  // Extract intensity vector

        tx_exp.send(new_exp.clone()).unwrap();  // Send vector to simulator

        // Draw experimental spectrum
        da1.connect_draw(move |_da: &gtk::DrawingArea, cr: &cairo::Context| {
            chart.draw(cr, new_exp.clone())
        });

        // Returning false here would close the receiver
        // and have senders fail
        glib::Continue(true)
    });

    // Call the mutex lock to get the MutexGuard;
    let rx_teor_guard = rx_teor.lock().unwrap();

    // MutexGuard is wrapped in a LockResult that we handle with the call to unwrap;
    let teor = rx_teor_guard.recv().unwrap();

    // Draw the new teor spectrum
    da.connect_draw(move |_da: &gtk::DrawingArea, cr: &cairo::Context| {
        chart.draw_spectra(cr, teor.clone())
    });

    // The lock is released automatically when a MutexGuard goes out of scope;
    // After dropping the lock, we can print the mutex value;

    // Build GUI menu bar
    build_system_menu(application);
    add_actions(
        application,
        &win,  // ApplicationWindow
        sender,  // glib::Sender
    );

    // ApplicationWindow final settings
    win.set_title("G FACTOR");
    win.set_position(gtk::WindowPosition::Center);
    win.show_all();
}

// Init app
fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.cairotest"),
        Default::default(),
    )
    .expect("Initialization failed...");

    // Sender/Receiver for Simulator operations only
    // let (tx_par, rx_par) = mpsc::channel();  // Sender/Receiver for parameters
    let (tx_exp, rx_exp) = mpsc::channel();  // Sender/Receiver for exp spectra

    // TEOR
    let (tx_teor, rx_teor) = mpsc::channel();  // Sender/Receiver for plotting spectra
    let rx_teor = Arc::new(Mutex::new(rx_teor));  // We create an Arc-Mutex for rx_teor;

    // Simulator thread
    thread::spawn(move || {
        // Get parameters from the GUI panel
        let mut zero_rad = Radical::probe();
        let rads = vec![zero_rad];

        // Init simulator
        let mut sim = Simulator::new();

        sim.teor = sim.calcola(rads);
        sim.exp = sim.calcola(vec![Radical::electron()]);
        // tx_teor.send((sim.exp.clone(), sim.teor.clone())).unwrap();
        let sp = Spectra {
            exp: sim.exp.clone(),
            teor: sim.teor.clone(),
        };
        tx_teor.send(sp).unwrap();

        loop {
            sim.exp = rx_exp.recv().unwrap();
        }
    });

    // GUI thread (main)
    application.connect_activate(move |app| {
        build_ui(
            app,
            tx_exp.clone(),
            rx_teor.clone()
        );
    });

    application.run(&args().collect::<Vec<_>>());
}
