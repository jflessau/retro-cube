# retro-cube

A 3D printable cube housing a raspberry pi zero W, an OLED display, buttons, a dial, and a speaker.

## Features

Use the dial to switch between the views:

- **Voice Mail** - Send, receive and listen to voice mails sent by other retro-cube users over WiFi
- **Countdown** - Start and watch a countdown timer
- **Clock** - Display the current time and set an alarm
- **Weather** - Display the current weather for your location

## Hardware

You need a raspberry pi zero W 2 and the sdd1306 OLED display.

### Wiring

| Pi (physical pin) | OLED Pin | Function      |
| ----------------- | -------- | ------------- |
| 1                 | VCC      | Power (3.3 V) |
| 6                 | GND      | Ground        |
| 5                 | RES      | Reset         |
| 19                | SDA      | Data          |
| 3                 | DC       | Data/Command  |
| 23                | SCK      | Clock         |
| 24                | CS       | Chip Select   |

## Software

## Development

Use `cargo run` to build and run the application.

## MacOs

On macOS, you likely need to install `sdl2`, e.g. with Homebrew:

```sh
brew install sdl2
export LIBRARY_PATH="/opt/homebrew/lib:$LIBRARY_PATH"
export C_INCLUDE_PATH="/opt/homebrew/include:$C_INCLUDE_PATH"
```

### Pi Zero 2 W (Raspberry Pi OS LIte 64-bit)

```sh
sudo dpkg --add-architecture armhf
sudo apt-get update
sudo apt-get install libc6:armhf
sudo apt-get install libsdl2-2.0-0:armhf
```
