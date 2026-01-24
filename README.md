# retro-cube

<img alt="A rectangular 3D printed beige case for a OLED display showing the current time." src="demo.jpg" width="440px"/>

A 3D printable housing for a Raspberry Pi Zero W, an OLED display, and a rotary encoder.

## 🌟 Features

Use the dial to switch between views:

- **Clock** - Current date and time
- **Weather** - Current weather for your location
- **Message** - Messages loaded from a basic-auth protected server
- **Night mode** - Turns the display off and since its an OLED, no light is emitted at all

The server code is also included in this repo.

## 🔨 How to build one

### Hardware

Aside from a Pi Zero W2 you need:

- `1x` SSD1309 OLED display, **SPI**-version, 128x64, 2.4 inch, Pins at the bottom of the display, not left or right. 
- `1x` GIAK KY-040 Rotary Encoder
- `12x` female-to-female jumper wires to connect the OLED to the Pi's GPIO pins

STL files for 3D printing the case can be found in `./3d`. Note that you might need a little glue to properly attach the knob to the rotary encoder.

#### Wiring

This is for the Raspberry Pi 3B+ and Zero 2 W. Other models might have a different pinout.
Consult the manual for both the Pi and the OLED display for your specific Pi model.

OLED Display:

| Pi (physical pin) | OLED pin |
| ----------------- | -------- |
| 6                 | GND      |
| 2                 | VCC      |
| 5                 | RES      |
| 19                | SDA      |
| 3                 | DC       |
| 23                | SCK      |
| 24                | CS       |

Rotary Encoder:

| Pi (physical pin) | Rotary encoder pin |
| ----------------- | ------------------ |
| 39                | GND                |
| 1                 | VCC                |
| 36                | CLK                |
| 10                | DT                 |
| 8                 | SW                 |

### Software

The code running on the Pi is located in `./os`.
The message server is located in `./server`.

#### Server

The server serves the text message for the pi and a web form to update the text message. You can run it on the pi or any other machine the pi can reach.

To run it, rename `./server/.env.example` to `./server/.env` and set the following variables:

| Env var       | Description                                                                  |
| ------------- | ---------------------------------------------------------------------------- |
| AUTH_USERNAME | Username for basic auth. Use the same here as for MESSAGE_USERNAME in `./os` |
| AUTH_PASSWORD | Password for basic auth. Use the same here as for MESSAGE_PASSWORD in `./os` |

Start it with `cargo run`. It serves on `http://localhost:3000`.

#### Os

The os service runs on the pi, controls the display, reads inputs from the rotary encoder and connects to the server to fetch the current message.

To run it on the pi, rename `./os/.env.example` to `./os/.env` and set the following variables:

| Env var                  | Description                                                                     |
| ------------------------ | ------------------------------------------------------------------------------- |
| PI_USER                  | Username for ssh (usually, this is just `pi`)                                   |
| PI_IP                    | The IP address of your Pi                                                       |
| REFETCH_INTERVAL_SECONDS | Seconds to wait between fetching weather and message data                       |
| TIMEZONE                 | Your time zone in [IANA format](https://data.iana.org/time-zones/tzdb/zone.tab) |
| MESSAGE_URL              | URL of the message server in `./server`                                         |
| MESSAGE_USERNAME         | Choose a username                                                               |
| MESSAGE_PASSWORD         | Choose a password                                                               |
| WEATHER_LAT              | Latitude of the location to get the weather for, e.g. `50.2`                    |
| WEATHER_LON              | Longitude of the location to get the weather for, e.g. `12.9`                   |

Then run `just run-remote` which connects via ssh to your pi using the `PI_USER` and `PI_IP` variables. It builds an executable binary within docker on your machine, copies it to your pi and runs it. This step will be much nicer when you have (keybased) paswordless ssh setup with your pi.

## Development

Use `cargo run` for both the os and the server to run the application on your local machine.
For the os service, instead of drawing to the real OLED, this will render the OLED's content into a simulator running in a separate window.

### macOS

On macOS, you likely need to install `sdl2` to run the `./os` service, e.g. with Homebrew:

```sh
brew install sdl2
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
```
