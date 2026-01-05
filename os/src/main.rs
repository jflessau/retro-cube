// mod message;
mod state;
// mod weather;
#[cfg(feature = "pi")]
use anyhow::Result;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};

#[cfg(feature = "simulator")]
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::Keycode,
};
use log::info;
use panic_semihosting as _;

#[cfg(feature = "pi")]
use ssd1309::prelude::*;
#[cfg(feature = "simulator")]
use state::Event;

fn main() {
    // setup logging

    dotenv::dotenv().ok();
    env_logger::init();
    info!("start");

    // start on simulator or hardware

    #[cfg(feature = "simulator")]
    simulate();

    #[cfg(not(feature = "simulator"))]
    run();
}

#[cfg(feature = "simulator")]
fn simulate() {
    info!("start simulator");

    let mut display: SimulatorDisplay<_> =
        SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64)).into();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::Custom {
            color_off: Rgb888::BLACK,
            color_on: Rgb888::GREEN,
        })
        .build();
    let mut window = Window::new("Click to move circle", &output_settings);

    let mut state = state::State::new();

    'running: loop {
        display.clear(BinaryColor::Off);

        state.update(&mut display, Event::Tick);
        window.update(&mut display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => break 'running,
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Up => state.update(&mut display, Event::NavigateUp),
                        Keycode::Down => state.update(&mut display, Event::NavigateDown),
                        _ => {}
                    };
                }
                _ => {}
            }
        }
    }
}

#[cfg(feature = "pi")]
fn run() -> Result<()> {
    use display_interface_i2c::I2CInterface;
    use embedded_graphics::{
        mono_font::{MonoTextStyle, ascii::FONT_6X10},
        pixelcolor::BinaryColor,
        prelude::*,
        text::Text,
    };
    use linux_embedded_hal::I2cdev;
    use ssd1309::{Builder, mode::GraphicsMode};

    // Open I2C bus (typically /dev/i2c-1 on Raspberry Pi)
    let i2c = I2cdev::new("/dev/i2c-1")?;

    // Create display interface
    let interface = I2CInterface::new(i2c, 0x3C, 0x40);

    // Initialize display in graphics mode
    let mut display: GraphicsMode<_> = Builder::new().connect(interface).into();

    // Reset (if you have GPIO for reset pin)
    // display.reset(&mut reset_pin, &mut delay)?;

    // Initialize the display
    display.init()?;
    display.flush()?;

    // Draw text
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("Hello OLED!", Point::new(0, 10), style).draw(&mut display)?;

    // Update display
    display.flush()?;

    Ok(())
}
