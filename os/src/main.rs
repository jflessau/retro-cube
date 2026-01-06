use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use rppal::{
    gpio::Gpio,
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use ssd1309::{self, builder::Builder, mode::graphics::GraphicsMode};

fn main() -> anyhow::Result<()> {
    println!("Starting SSD1309 SPI example...");

    // SPI0, SS0 (not Cs0), 9MHz, Mode0
    let spi = Spi::new(
        Bus::Spi0,
        SlaveSelect::Ss0, // Fixed: Ss0 not Cs0
        9_000_000,
        Mode::Mode0,
    )?;

    // GPIO pins
    let dc = Gpio::new()?.get(2)?.into_output(); // DC (Data/Command) on BCM GPIO 2 (physical pin 3)
    let mut res = Gpio::new()?.get(3)?.into_output(); // RES (Reset) on BCM GPIO 3 (physical pin 5)
    let cs = Gpio::new()?.get(8)?.into_output(); // CS (Chip Select) on BCM GPIO 8 (physical pin 24)

    // SPIInterface: SPI + DC (RST manual)
    let di = SPIInterface::new(spi, dc, cs);

    let mut disp: GraphicsMode<_> = Builder::new().connect(di).into();

    // Reset sequence
    res.set_high();
    std::thread::sleep(std::time::Duration::from_millis(10));
    res.set_low();
    std::thread::sleep(std::time::Duration::from_millis(10));
    res.set_high();
    std::thread::sleep(std::time::Duration::from_millis(10));

    disp.init().expect("Failed to initialize display");
    println!("Display initialized.");

    disp.clear();
    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    Circle::new(Point::new(32, 8), 24)
        .into_styled(style)
        .draw(&mut disp)
        .expect("Failed to draw circle");

    disp.flush().expect("fails to flush");
    println!("Circle displayed.");

    std::thread::sleep(std::time::Duration::from_secs(5));
    disp.clear();
    disp.flush().expect("fails to flush");
    println!("Display cleared.");

    Ok(())
}
