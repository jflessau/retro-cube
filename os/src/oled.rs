use crate::state;
use anyhow::Result;
use display_interface_spi::SPIInterface;
use log::{info, trace};
use rppal::{
    gpio::{Gpio, Level},
    spi::{Bus, Mode, SlaveSelect, Spi},
};
use ssd1309::{
    self, builder::Builder, displayrotation::DisplayRotation, mode::graphics::GraphicsMode,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
    time::{Duration, Instant},
};

pub fn render() -> Result<()> {
    // map LED pin

    let mut led = Gpio::new()?.get(26)?.into_output();

    // blink LED to indicate startup
    led.set_high();
    sleep(Duration::from_millis(1000));
    led.set_low();
    sleep(Duration::from_millis(1000));
    led.set_high();

    // map rotary pins

    let rotary_clk = Gpio::new()?.get(16)?.into_input();
    let rotary_dt = Gpio::new()?.get(15)?.into_input();
    let rotary_sw = Gpio::new()?.get(14)?.into_input_pullup();

    // map OLED pins

    let dc = Gpio::new()?.get(2)?.into_output();
    let mut res = Gpio::new()?.get(3)?.into_output();
    let cs = Gpio::new()?.get(8)?.into_output();

    // setup SPI interface

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 9_000_000, Mode::Mode0)?;
    let di = SPIInterface::new(spi, dc, cs);

    // setup display

    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .connect(di)
        .into();

    // reset display

    res.set_high();
    sleep(Duration::from_millis(10));
    res.set_low();
    sleep(Duration::from_millis(10));
    res.set_high();
    sleep(Duration::from_millis(10));

    display.init().expect("Failed to initialize display");

    // setup trigger for exit on ctrl-c

    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
            info!("received ctrl-c, exiting");
        })?;
    }

    // initialize state

    let mut state = state::State::new();
    let mut last_rotary_clk_state = rotary_clk.read();
    let mut last_rotary_sw_state = rotary_sw.read();

    // render state and handle events

    let mut last_nav_at = Instant::now();
    while running.load(Ordering::SeqCst) {
        trace!("main loop tick");

        // detect rotary events

        let clk_state = rotary_clk.read();
        let dt_state = rotary_dt.read();
        if last_nav_at.elapsed().as_millis() > 144 {
            if clk_state != last_rotary_clk_state {
                if dt_state != clk_state {
                    info!("navigate ->");
                    state.update(&mut display, state::Event::NavigateUp);
                    last_nav_at = Instant::now();
                } else {
                    info!("navigate <-");
                    state.update(&mut display, state::Event::NavigateDown);
                    last_nav_at = Instant::now();
                }
            }
        }
        last_rotary_clk_state = clk_state;

        // detect rotary button

        let sw_state = rotary_sw.read();
        if sw_state != last_rotary_sw_state
            && sw_state == Level::Low
            && last_nav_at.elapsed().as_millis() > 144
        {
            state.update(&mut display, state::Event::ToggleSleep);
        }
        last_rotary_sw_state = sw_state;

        // render state

        display.clear();
        state.update(&mut display, state::Event::Tick);
        display.flush().expect("fails to flush");
    }

    // clear display

    display.clear();
    display.flush().expect("fails to flush");

    Ok(())
}
