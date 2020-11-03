extern crate glib;
extern crate gio;
extern crate gtk;

use glib::clone;
use gtk::prelude::*;
// use gio::prelude::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use crate::io::{get_from_asciistring};

use crate::plt::{Chart, Spectra};

pub struct Gui {
    pub builder: gtk::Builder,
    pub win: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
    pub open_sender: glib::Sender<String>,
}

impl Gui {
    pub fn new(chart: Chart) -> Self {
        // Loading glade file
        let builder = gtk::Builder::from_string(include_str!("ui.glade"));

        // Windows
        let win: gtk::ApplicationWindow =
            builder.get_object("application_window").expect("err build win");

        win.set_title("g Factor");
        win.set_position(gtk::WindowPosition::Center);
        win.show_all();

        // Drawing Area
        let drawing_area: gtk::DrawingArea =
            builder.get_object("drawing_area").expect("err building drawing_area");

        // Create a simple glib streaming channel
        let (open_sender, open_receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        let da = drawing_area.clone();  // Pass to the next function. TODO: remove
        // Opening a file...
        open_receiver.attach(None, move |msg: String| {
            let new_exp: &str = &msg[..];  // String to &str
            let new_exp = get_from_asciistring(new_exp);  // Extract intensity vector

            // Just send to sim

            // TODO: Remove
            da.connect_draw(move |_da: &gtk::DrawingArea, cr: &cairo::Context| {
                chart.draw_spectra(cr, Spectra { exp: new_exp.clone(), teor: Vec::new() })
            });

            // Returning false here would close the receiver
            // and have senders fail
            glib::Continue(true)
        });

        // TODO: sostituisci con una demo, magari tipo globe
        let da = drawing_area.clone();
        da.connect_draw(move |_da: &gtk::DrawingArea, cr: &cairo::Context| {
            chart.draw_spectra(cr, Spectra { exp: Vec::new(), teor: Vec::new() })
        });

        Self {
            builder,
            win,
            drawing_area,
            open_sender
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
}  // impl GuiData
