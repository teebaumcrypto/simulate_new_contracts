pub mod anvil_fork;
pub mod settings;
pub mod token_methods;

use crate::settings::Settings;
use ethers::types::U256;
use lazy_static::{initialize, lazy_static};

pub fn preload_lazy_static() {
    // load and print the settings
    println!("Settings loading..");
    println!("{}", *SETTINGS);
    initialize(&ONE_ETH);
    initialize(&TEN_ETH);
}

// global configuration struct
lazy_static! {
    pub static ref SETTINGS: Settings = Settings::default();
}

// global 1 eth as u256
lazy_static! {
    pub static ref ONE_ETH: U256 = U256::from(1e18 as u64);
}

// global 10 eth as u256
lazy_static! {
    pub static ref TEN_ETH: U256 = U256::from(10e18 as u64);
}
