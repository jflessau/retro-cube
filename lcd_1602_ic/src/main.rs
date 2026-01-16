use rppal::i2c::I2c;
use std::thread;
use std::time::Duration;

// LCD Commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_FUNCTIONSET: u8 = 0x20;

// Entry Mode flags
const LCD_ENTRYLEFT: u8 = 0x02;

// Display Control flags
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKOFF: u8 = 0x00;

// Function Set flags
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_5X8DOTS: u8 = 0x00;

// PCF8574 pin mapping (standard)
const EN: u8 = 0b00000100; // Enable bit
const RS: u8 = 0b00000001; // Register select bit
const BACKLIGHT: u8 = 0b00001000; // Backlight bit

struct Lcd {
    i2c: I2c,
}

impl Lcd {
    fn new(address: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(address)?;

        let mut lcd = Lcd { i2c };
        lcd.init()?;
        Ok(lcd)
    }

    fn write_byte(&mut self, data: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.i2c.write(&[data | BACKLIGHT])?;
        Ok(())
    }

    fn pulse_enable(&mut self, data: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.write_byte(data | EN)?;
        thread::sleep(Duration::from_micros(1));
        self.write_byte(data & !EN)?;
        thread::sleep(Duration::from_micros(50));
        Ok(())
    }

    fn write_4bits(&mut self, data: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.write_byte(data)?;
        self.pulse_enable(data)?;
        Ok(())
    }

    fn send(&mut self, data: u8, mode: u8) -> Result<(), Box<dyn std::error::Error>> {
        let high_bits = data & 0xF0;
        let low_bits = (data << 4) & 0xF0;
        self.write_4bits(high_bits | mode)?;
        self.write_4bits(low_bits | mode)?;
        Ok(())
    }

    fn command(&mut self, cmd: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.send(cmd, 0)?;
        thread::sleep(Duration::from_micros(100));
        Ok(())
    }

    fn write_char(&mut self, ch: u8) -> Result<(), Box<dyn std::error::Error>> {
        self.send(ch, RS)?;
        thread::sleep(Duration::from_micros(100));
        Ok(())
    }

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        thread::sleep(Duration::from_millis(50));

        // Initialize LCD in 4-bit mode
        self.write_4bits(0x03 << 4)?;
        thread::sleep(Duration::from_millis(5));
        self.write_4bits(0x03 << 4)?;
        thread::sleep(Duration::from_micros(150));
        self.write_4bits(0x03 << 4)?;
        thread::sleep(Duration::from_micros(150));
        self.write_4bits(0x02 << 4)?;
        thread::sleep(Duration::from_micros(150));

        // Configure LCD
        self.command(LCD_FUNCTIONSET | LCD_4BITMODE | LCD_2LINE | LCD_5X8DOTS)?;
        self.command(LCD_DISPLAYCONTROL | LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF)?;
        self.clear()?;
        self.command(LCD_ENTRYMODESET | LCD_ENTRYLEFT)?;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.command(LCD_CLEARDISPLAY)?;
        thread::sleep(Duration::from_millis(2));
        Ok(())
    }

    fn set_cursor(&mut self, col: u8, row: u8) -> Result<(), Box<dyn std::error::Error>> {
        let row_offsets = [0x00, 0x40];
        let offset = row_offsets[row as usize] + col;
        self.command(0x80 | offset)?;
        Ok(())
    }

    fn print(&mut self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        for ch in text.chars() {
            if ch.is_ascii() {
                self.write_char(ch as u8)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize LCD at address 0x27
    let mut lcd = Lcd::new(0x27)?;

    // Clear display and write message
    lcd.clear()?;
    lcd.print("hello world")?;

    // Write to second line
    lcd.set_cursor(0, 1)?;
    lcd.print("LCD working!")?;

    println!("✅ 'hello world' written to LCD1602!");

    Ok(())
}
