use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Ellipse, PrimitiveStyle, Rectangle},
};
use linux_embedded_hal::I2cdev;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing SSD1306 display...");

    // Initialize I2C - /dev/i2c-1 is the default I2C bus on Raspberry Pi
    let i2c = I2cdev::new("/dev/i2c-1")?;

    // Create the display interface
    let interface = I2CDisplayInterface::new(i2c);

    // Create the display driver for 128x32 display (rotated 90 degrees for portrait)
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate90)
        .into_buffered_graphics_mode();

    // Initialize the display
    display
        .init()
        .map_err(|e| format!("Display init error: {:?}", e))?;

    println!("Display initialized successfully!");

    // Clear the display
    display
        .clear(BinaryColor::Off)
        .map_err(|e| format!("Display clear error: {:?}", e))?;

    // Draw a majestic phallus (portrait orientation: 32x128)

    // Left ball
    let left_ball =
        Circle::new(Point::new(6, 82), 8).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    left_ball.draw(&mut display).unwrap();

    // Right ball
    let right_ball =
        Circle::new(Point::new(18, 82), 8).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    right_ball.draw(&mut display).unwrap();

    // Shaft (rectangle)
    let shaft = Rectangle::new(Point::new(10, 30), Size::new(12, 56))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    shaft.draw(&mut display).unwrap();

    // Tip (ellipse)
    let tip = Ellipse::new(Point::new(6, 18), Size::new(20, 16))
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
    tip.draw(&mut display).unwrap();

    // Splash dots (bonus points!)
    let splash_dots = [
        Point::new(4, 10),
        Point::new(8, 6),
        Point::new(2, 14),
        Point::new(24, 8),
        Point::new(26, 12),
        Point::new(28, 6),
        Point::new(22, 16),
    ];

    for dot_pos in splash_dots.iter() {
        let dot = Circle::new(*dot_pos, 2).into_styled(PrimitiveStyle::with_fill(BinaryColor::On));
        dot.draw(&mut display).unwrap();
    }

    // Flush the buffer to the display
    display
        .flush()
        .map_err(|e| format!("Display flush error: {:?}", e))?;

    println!("Masterpiece drawn successfully!");
    println!("Display will stay on. Press Ctrl+C to exit.");

    // Keep the program running so the display stays on
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
