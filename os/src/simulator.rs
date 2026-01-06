use crate::state;

use anyhow::Result;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
    sdl2::Keycode,
};
use log::info;
use state::Event;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub fn render() -> Result<()> {
    // setup simulator

    info!("start simulator");
    let mut display: SimulatorDisplay<_> =
        SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64)).into();
    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::Custom {
            color_off: Rgb888::BLACK,
            color_on: Rgb888::GREEN,
        })
        .build();
    let mut window = Window::new("retro-cube display simulator", &output_settings);

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

    // render state and handle events

    while running.load(Ordering::SeqCst) {
        display.clear(BinaryColor::Off);

        state.update(&mut display, Event::Tick);
        window.update(&mut display);

        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::KeyDown { keycode, .. } => {
                    match keycode {
                        Keycode::Up => state.update(&mut display, Event::NavigateUp),
                        Keycode::Down => state.update(&mut display, Event::NavigateDown),
                        Keycode::RETURN => state.update(&mut display, Event::ToggleSleep),
                        _ => {}
                    };
                }
                _ => {}
            }
        }
    }

    Ok(())
}
