mod message;
mod weather;

mod state;
use log::{error, info};

#[cfg(feature = "oled")]
mod oled;
#[cfg(not(feature = "oled"))]
mod simulator;

fn main() {
    // setup logging

    dotenv::dotenv().ok();
    env_logger::init();
    info!("start");

    #[cfg(not(feature = "oled"))]
    if let Err(err) = simulator::render() {
        error!("error: {:?}", err);
    }

    #[cfg(feature = "oled")]
    if let Err(err) = oled::render() {
        error!("error: {:?}", err);
    }
}
