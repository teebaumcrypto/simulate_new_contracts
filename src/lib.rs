pub mod anvil_fork;
pub mod settings;
pub mod token_methods;

use crate::settings::Settings;
use lazy_static::lazy_static;

pub fn preload_lazy_static() {
    // load and print the settings
    println!("Settings loading..");
    println!("{}", *SETTINGS);
}

// global configuration struct
lazy_static! {
    pub static ref SETTINGS: Settings = Settings::default();
}
