#![windows_subsystem = "windows"]
mod piagamdispendik;
#[macro_use]
extern crate lazy_static;
extern crate serde_xml_rs;
extern crate serde_derive;

#[macro_use]
mod lang;
use lang::SupportedLanguage;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

mod ui;

fn main() {
    lang::set_current_lang(SupportedLanguage::EnglishUs);
    ui::init_app();
}
