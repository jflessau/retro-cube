use display_interface_i2c::I2CInterface;
use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use rppal::i2c::I2c;
use ssd1306::{Ssd1306, prelude::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting SSD1306 SPI example...");
    let spi = Spi::new(
        Spi::BUS_0, // SPI0
        SlaveSelect::Cs0,
        9_000_000, // 9MHz
        8,         // 8 bits per word
    )
    .expect("Failed to open SPI bus.");
    println!("SPI bus opened.");

    let di = SPIInterface::new(
        spi,
        Pin::new(25)?, // DC pin (GPIO 25)
        Pin::new(24)?, // RST pin (GPIO 24)
    );
    println!("SPI interface created.");

    let mut disp =
        Ssd1306::new(di, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    println!("SSD1306 display object created.");

    if let Err(err) = disp.init() {
        panic!("Failed to initialize display: {err:#?}");
    }

    // Clear internal buffer
    disp.clear(Rgb888::BLACK.into())
        .expect("Failed to clear display buffer");
    println!("Display initialized and buffer cleared.");

    // Simple 1‑px outline circle, centered roughly in the display
    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    Circle::new(Point::new(32, 8), 24) // top‑left, diameter 24
        .into_styled(style)
        .draw(&mut disp)
        .expect("Failed to draw circle");
    println!("Circle drawn to buffer.");

    disp.flush().expect("Failed to flush display");
    println!("Display buffer flushed to screen.");

    Ok(())
}

// println!("Starting SSD1306 example...");
// let mut i2c = I2c::new()?;
// println!("I2C bus opened.");
// i2c.set_slave_address(0x3c)
//     .expect("Failed to set I2C slave address.");
// println!("I2C slave address set to 0x3c.");

// let di = I2CInterface::new(i2c, 0x3c, 0x40);
// println!("I2C interface created.");
