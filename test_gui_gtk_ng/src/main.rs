// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    env,
    path::{Path, PathBuf},
};

use pw_pathux::expand_home_dir_or_mine;

use pw_gix::{
    gtk::{self, prelude::*},
    gtkx::window::RememberGeometry,
    recollections,
    wrapper::*,
};

const DEFAULT_CONFIG_DIR_PATH: &str = "~/.config/test_gui_gtk_ng";

const DCDP_OVERRIDE_ENVAR: &str = "COLOUR_MATH_NG_TEST_GUI_GTK_CONFIG_DIR";

fn abs_default_config_dir_path() -> PathBuf {
    expand_home_dir_or_mine(&Path::new(DEFAULT_CONFIG_DIR_PATH))
}

pub fn config_dir_path() -> PathBuf {
    match env::var(DCDP_OVERRIDE_ENVAR) {
        Ok(dir_path) => {
            if dir_path.is_empty() {
                abs_default_config_dir_path()
            } else if dir_path.starts_with('~') {
                expand_home_dir_or_mine(&Path::new(&dir_path))
            } else {
                dir_path.into()
            }
        }
        Err(_) => abs_default_config_dir_path(),
    }
}

pub fn gui_config_dir_path() -> PathBuf {
    config_dir_path().join("gui")
}

pub fn recollection_file_path() -> PathBuf {
    gui_config_dir_path().join("recollections")
}

use colour_math_gtk_ng::colour::beigui::hue_wheel::Shape;
use colour_math_gtk_ng::{
    attributes::ColourAttributeDisplayStackBuilder,
    colour::{ColouredShape, ScalarAttribute, RGB},
    hue_wheel::GtkHueWheelBuilder,
    manipulator::ColourManipulatorGUIBuilder,
    rgb_entry::RGBHexEntryBuilder,
};
use colour_math_ng::{HueConstants, Prop, HCV};
use std::rc::Rc;

fn main() {
    gtk::init().expect("nowhere to go if Gtk++ initialization fails");
    recollections::init(&recollection_file_path());
    let win = gtk::Window::new(gtk::WindowType::Toplevel);
    win.set_geometry_from_recollections("main_window", (600, 400));
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let attributes = vec![
        ScalarAttribute::Value,
        ScalarAttribute::Chroma,
        ScalarAttribute::Greyness,
        ScalarAttribute::Warmth,
    ];

    let cads = ColourAttributeDisplayStackBuilder::new()
        .attributes(&attributes)
        .build();
    cads.set_colour(Some(&RGB::from([0.1, 0.4, 0.7])));
    cads.set_target_colour(Some(&RGB::from([0.7, 0.4, 0.7])));
    vbox.pack_start(&cads.pwo(), true, true, 0);

    let rgb_hex_entry = RGBHexEntryBuilder::<u16>::new()
        .initial_colour(&RGB::from([0.1, 0.4, 0.7]))
        .editable(true)
        .build();
    let cads_c = Rc::clone(&cads);
    rgb_hex_entry.connect_colour_changed(move |c| cads_c.set_colour(Some(&c)));
    vbox.pack_start(&rgb_hex_entry.pwo(), false, false, 0);

    let colour_manipulator = ColourManipulatorGUIBuilder::new().build();
    let hex_entry_c = Rc::clone(&rgb_hex_entry);
    colour_manipulator.connect_changed(move |c| hex_entry_c.set_colour(&c));
    let cads_c = Rc::clone(&cads);
    colour_manipulator.connect_changed(move |c| cads_c.set_colour(Some(&c)));
    vbox.pack_start(&colour_manipulator.pwo(), true, true, 0);

    let gtk_hue_wheel = GtkHueWheelBuilder::new()
        .attributes(&attributes)
        .menu_item_specs(&[("add", ("Add", None, Some("Add something")).into(), 0)])
        .build();
    vbox.pack_start(&gtk_hue_wheel.pwo(), true, true, 0);
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::RED,
        "Red",
        "Pure Red",
        Shape::Square,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::YELLOW,
        "Yellow",
        "Pure Yellow",
        Shape::Diamond,
    ));
    gtk_hue_wheel.add_item(ColouredShape::new(
        &HCV::new_grey(Prop::ONE / 2),
        "Grey",
        "Midle Grey",
        Shape::Circle,
    ));

    vbox.show_all();
    win.add(&vbox);
    win.connect_destroy(|_| gtk::main_quit());
    win.show();
    gtk::main()
}
