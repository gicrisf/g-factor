extern crate glib;
extern crate gio;
extern crate gtk;

use glib::clone;
use gtk::prelude::*;
// use gio::prelude::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::collections::HashMap;

use crate::io::{get_from_asciistring};
use crate::plt::{Chart, Spectra};
use crate::sim::{Simulator};
use crate::ent::{Radical};
use crate::ui::settings::{create_page};

pub struct Gui {
    // Main window
    pub builder: gtk::Builder,
    pub win: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
    pub open_sender: glib::Sender<String>,
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

        let sim1 = sim.clone();
        let mut sim_rads = sim1.rads.lock().unwrap();
        *sim_rads = vec![Radical::probe(), Radical::electron()];

        // Drawing Area
        let drawing_area: gtk::DrawingArea =
            builder.get_object("drawing_area").expect("err building drawing_area");

        // Create a simple glib streaming channel
        let (open_sender, open_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let da = drawing_area.clone();  // Pass to the next function. TODO: remove
        // Opening a file...
        let sim1 = sim.clone();
        open_receiver.attach(None, move |msg: String| {
            let new_exp: &str = &msg[..];  // String to &str
            let new_exp = get_from_asciistring(new_exp);  // Extract intensity vector
            let mut exp_mut = sim1.exp.lock().unwrap();
            *exp_mut = new_exp;  // Pass it to simulator

            // TODO: Draw

            // Returning false here would close the receiver
            // and have senders fail
            glib::Continue(true)
        });

        Self {
            builder,
            win,
            drawing_area,
            open_sender,
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

        let radicals_guard = self.sim.rads.lock().unwrap();
        let radicals_clone = radicals_guard.clone();
        let settings_btn_clone = settings_btn.clone();
        // On clicked button
        settings_btn.connect_clicked(move |_| {
            let settings_win: gtk::Window = gtk::Window::new(gtk::WindowType::Toplevel);
            let notebook = gtk::Notebook::new();

            // Store gtk tabs and inner grids with rad values in a Hashmap
            // Must put this variable in the main struct ! ALERT
            // Store Rad index VS gtk Grid
            let mut boxes: HashMap<usize, gtk::Box> = HashMap::new();

            for (idx, rad) in radicals_clone.iter().enumerate() {
                let (content, tab) = create_page(&idx, rad);
                boxes.insert(idx, content.rad_box);
                notebook.append_page(&boxes[&idx], Some(&tab.tab_box));
            }

            settings_win.add(&notebook);
            settings_win.set_title("g Factor - Radicals");
            settings_win.set_position(gtk::WindowPosition::Center);
            settings_win.show_all();

            // Ghosting button when the Settings are opened
            let settings_btn_clone_1 = settings_btn_clone.clone();
            settings_btn_clone.set_sensitive(false);

            settings_win.connect_delete_event(move |_, _| {
                // Assign values here
                settings_btn_clone_1.set_sensitive(true);
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
