mod lang;
use serde::{Deserialize, Serialize, Serializer};

use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum TingkatReward {
    Kecamatan,
    Kabupaten,
    Provinsi,
    Nasional,
    Internasional,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
struct Reward {
    no: u32,
    penerima: String,
    lembaga: String,
    lomba: String,
    juara: u32,
    tingkat: TingkatReward,
}

impl Reward {
}


#[derive(Error, Debug)]
pub enum AppError {
    #[error("reading or writing IO error")]
    InputOutputError(#[from] std::io::Error),

    #[error("error on parsing csv file")]
    CsvError(#[from] csv::Error),
}

#![windows_subsystem = "windows"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod lang;
use lang::SupportedLanguage;

extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

mod docx_filler;
mod ui;

fn main() {
    lang::set_current_lang(SupportedLanguage::EnglishUs);
    ui::init_app();
}
