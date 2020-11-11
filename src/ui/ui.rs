extern crate glib;
extern crate gio;
extern crate gtk;

use glib::clone;
use gtk::prelude::*;
// use gio::prelude::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::sync::Arc;

use crate::io::{get_from_asciistring};
use crate::plt::{Chart, Spectra};
use crate::sim::{Simulator};
use crate::ent::{Radical};
use crate::ui::settings::{Settings};

pub struct Gui {
    // Main window
    pub builder: gtk::Builder,
    pub win: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
    pub open_sender: glib::Sender<String>,
    pub nucpar_sender: glib::Sender<(usize, usize, String, String, f64)>,
    pub radpar_sender: glib::Sender<(usize, String, String, f64)>,
    pub radgen_sender: glib::Sender<(usize, bool)>,  // Index + "insert to" or "remove from"
    pub sim: Simulator,
    pub chart: Chart,
}

impl Gui {
    pub fn new(chart: Chart, sim: Simulator) -> Self {
        // Loading glade file
        let builder = gtk::Builder::from_string(include_str!("ui.glade"));

        // Windows
        let win: gtk::ApplicationWindow =
            builder.get_object("application_window").expect("err build win");

        win.set_title("g Factor");
        win.set_position(gtk::WindowPosition::Center);
        win.show_all();

        let sim_rads = Arc::clone(&sim.rads);
        let mut sim_rads_guard = sim_rads.lock().unwrap();
        // sim_rads_guard.retain(|_e| false);  // Erase all
        sim_rads_guard.push(Radical::probe());
        sim_rads_guard.push(Radical::electron());

        // Drawing Area
        let drawing_area: gtk::DrawingArea =
            builder.get_object("drawing_area").expect("err building drawing_area");

        let (open_sender, open_receiver) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // RADICALS
        // Change radical parameters
        let (radpar_sender, radpar_receiver) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // Change nucleus parameters
        let (nucpar_sender, nucpar_receiver) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // Add or Del whole radicals
        let (radgen_sender, radgen_receiver) =
            glib::MainContext::channel(glib::PRIORITY_DEFAULT);


        // let da = drawing_area.clone();  // Pass to the next function. TODO: remove
        // Opening a file...
        let sim_clone = sim.clone();
        open_receiver.attach(None, move |msg: String| {
            let new_exp: &str = &msg[..];  // String to &str
            let new_exp = get_from_asciistring(new_exp);  // Extract intensity vector
            let mut exp_mut = sim_clone.exp.lock().unwrap();
            *exp_mut = new_exp;  // Pass it to simulator

            // TODO: Draw

            // Returning false here would close the receiver
            // and have senders fail
            glib::Continue(true)
        });

        // Add or Del whole radicals RECEIVER ACTION
        let sim_rads_clone = Arc::clone(&sim.rads);
        radgen_receiver.attach(None, move |signal: (usize, bool)| {
            let mut rads = sim_rads_clone.lock().unwrap();
            if signal.1 { rads.push(Radical::electron()); }  // Add to the last position
            else { rads.remove(signal.0); }  // Remove from index
            glib::Continue(true)
        });

        // Receive and change Params!
        let sim_rads_clone = Arc::clone(&sim.rads);
        radpar_receiver.attach(None, move |new_par: (usize, String, String, f64)| {
            // Debugger
            // println!("Radical n.{}\n{}\n{}\n{}\n", new_par.0, new_par.1, new_par.2, new_par.3);
            let mut rads = sim_rads_clone.lock().unwrap();
            rads[new_par.0] = rads[new_par.0].set_radpar(new_par.1, new_par.2, new_par.3);
            glib::Continue(true)
        });

        let sim_rads_clone = Arc::clone(&sim.rads);
        nucpar_receiver.attach(None, move |signal: (usize, usize, String, String, f64)| {
            let mut rads = sim_rads_clone.lock().unwrap();
            rads[signal.0] = rads[signal.0].set_nucpar(signal.1, signal.2, signal.3, signal.4);
            glib::Continue(true)
        });

        Self {
            builder,
            win,
            drawing_area,
            open_sender,
            nucpar_sender,
            radpar_sender,
            radgen_sender,
            sim,
            chart,
        }  // return Gui
    }  // new(application)

    pub fn build_menu(&self) -> gio::Menu {
        let menu_bar = gio::Menu::new();
        let file_menu = gio::Menu::new();
        file_menu.append(Some("Open"), Some("app.open"));
        menu_bar.append_submenu(Some("File"), &file_menu);

        menu_bar
    }  // build_system_menu

    pub fn open_action(&self) -> gio::SimpleAction {
        let window: &gtk::ApplicationWindow = &self.win;
        let sender: glib::Sender<String> = self.open_sender.clone();

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

            let open_sender = sender.clone();
            file_chooser.connect_response(move |file_chooser, response| {
                if response == gtk::ResponseType::Ok {
                    let filename = file_chooser.get_filename().expect("Couldn't get filename");
                    let file = File::open(&filename).expect("Couldn't open file");
                    let mut reader = BufReader::new(file);
                    let mut contents = String::new();
                    let _ = reader.read_to_string(&mut contents);

                    // Send contents to the main Context
                    open_sender.send(contents).unwrap();
                }
                file_chooser.close();
            });

            file_chooser.show_all();
        }));
        open
    }  // return open_action

    pub fn connect_buttons(&self) {
        // SETTINGS BUTTON
        let settings_btn: gtk::Button =
            self.builder.get_object("settings_btn").expect("err building settings_btn");

        // let radicals = Arc::clone(&self.sim.rads);

        let settings_btn_clone = settings_btn.clone();  // SETs BTN CLONE 00
        let nucpar_sender = self.nucpar_sender.clone();
        let radpar_sender = self.radpar_sender.clone();
        let radgen_sender = self.radgen_sender.clone();
        let arc_rads_clone = self.sim.rads.clone();

        // On clicked button
        settings_btn.connect_clicked(move |_| {
            let settings = Settings::new(
                Arc::clone(&arc_rads_clone),
                nucpar_sender.clone(),
                radpar_sender.clone(),
                radgen_sender.clone(),
            );

            let settings_btn_clone_1 = settings_btn_clone.clone();  // SETs BTN CLONE 01
            settings_btn_clone.set_sensitive(false); // Ghosting button

            settings.window.connect_delete_event(move |_, _| {
                // TODO: assign radicals to mutex
                settings_btn_clone_1.set_sensitive(true);  // Restore button
                gtk::Inhibit(false)
            });

        });

        // EXPERIMENTAL BUTTON
        let exp_btn: gtk::Button =
            self.builder.get_object("experimental_btn").expect("err building exp_btn");

        // Clone vars to move them in the next closure
        let sim_clone = self.sim.clone();
        let da = self.drawing_area.clone();
        let chart = self.chart.clone();
        let sim_rads_clone = self.sim.rads.clone();

        exp_btn.connect_clicked(move |_| {
            // Move sim mutex in here
            let mut sim_teor_guard = sim_clone.teor.lock().unwrap();
            let sim_exp_guard = sim_clone.exp.lock().unwrap();
            let sim_rads_guard = sim_rads_clone.lock().unwrap();
            // Update teor with internal rads
            let sim_rads_deref = &*sim_rads_guard;
            *sim_teor_guard = sim_clone.calcola(sim_rads_deref.clone());
            // Move sim vectors in the next closure
            let sim_teor_clone = sim_teor_guard.clone();
            let sim_exp_clone = sim_exp_guard.clone();
            // Draw spectra
            da.connect_draw(move |_da: &gtk::DrawingArea, cr: &cairo::Context| {
                chart.draw_spectra(cr, Spectra { exp: sim_exp_clone.clone(), teor: sim_teor_clone.clone() })
            });
        });

        // MONTECARLO BUTTON
        let mc_go_btn: gtk::Button =
            self.builder.get_object("mc_go_btn").expect("err building mc_go_button");

        let sim = self.sim.clone();

        // On clicked button
        mc_go_btn.connect_clicked(move |_| {
            println!("Result pre-scope: {}", *sim.mc_go.lock().unwrap());

            {
                let mut sim_mcgo_guard = sim.mc_go.lock().unwrap();
                *sim_mcgo_guard = !*sim_mcgo_guard;
                println!("Result in-scope: {}", sim_mcgo_guard);
            }

            println!("Result post-scope: {}", *sim.mc_go.lock().unwrap());
        });
    }

}  // impl GuiData
