# retro-cube

A 3D printable cube housing a raspberry pi zero W, an OLED display, buttons, a dial, and a speaker.

## Features

Use the dial to switch between the views:

- **Voice Mail** - Send, receive and listen to voice mails sent by other retro-cube users over WiFi
- **Countdown** - Start and watch a countdown timer
- **Clock** - Display the current time and set an alarm
- **Weather** - Display the current weather for your location

## Hardware

Aside from a Pi like the Raspberry Pi 3B+, you need

- `1x` SDD1306 OLED display, **SPI**-version, 128x64, 2.4 inch
- `1x` GIAK KY-040 Rotary Encoder
- `7x` female-to-female jumper wires to connect the OLED to the Pi's GPIO pins

### Wiring

This is for the Raspberry Pi 3B+. Other models might have a different pinout.
Consult the manual both the PI and the OLED display for your specific Pi model.

#### OLED Display

| Pi (physical pin) | OLED pin |
| ----------------- | -------- |
| 6                 | GND      |
| 2                 | VCC      |
| 5                 | RES      |
| 19                | SDA      |
| 3                 | DC       |
| 23                | SCK      |
| 24                | CS       |

#### Rotary Encoder

| Pi (physical pin) | Rotary encoder pin |
| ----------------- | ------------------ |
| 39                | GND                |
| 1                 | VCC                |
| 36                | CLK                |
| 10                | DT                 |
| 8                 | SW                 |

## Software

### Development

The software part of this project is located in `./os`.

If you are just here to build the retro-cube, and don't want to tinker with the code, you can skip this section and instead download the latest release from the release page on this GitHub repository.

#### Requirements

- [Rust](https://www.rust-lang.org/tools/install)
- [Just](https://github.com/casey/just) (can be installed with `cargo install just`)
- [Docker](https://docker.com) (optional)

##### MacOx

On macOS, you likely need to install `sdl2`, e.g. with Homebrew:

```sh
brew install sdl2
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
```

#### Run

Use `cargo run` to run the application on your local machine. Instead of drawing to the real OLED, this will render the OLED's content into a simulator running in a separate window.

#### Build & Deploy to Raspberry Pi

Renane `.env-example` to `.env` and set the following variables:

- `PI_USER`: the username you use to ssh into the pi (usually `pi`)
- `PI_IP`: the hostname or IP address of your pi (e.g. `192.168.189.9\*)

Then run `just deploy` to build the binary and copy it to `/home/pi/os` on the Pi.

If available, this will use the passwordless ssh login via ssh keys. Otherwise, you will be prompted for the password.
