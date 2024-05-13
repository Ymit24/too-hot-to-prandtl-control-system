# ❄️🎛️ Too Hot To Prandtl Control Software

## About this repo
This repo contains all the software I wrote for my undergraduate senior design project "Too Hot To Prandtl".

<!-- TABLE OF CONTENTS -->
<div>
  <h4>Table of Contents</h4>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</div>


## About the Project
My senior design project, "Too hot to prandtl", is a thermal management solution designed for heterogeniously packaged chips.
It is a fluid based cooler which leverages phase change to increase heat transfer.
Myself, along with four Mechnical Engineering students designed, simulated, fabricated, and tested this system.
As the teams sole Electrical Engineer, I wrote all the software and designed, ordered, assembled, and brought-up the custom embedded system's printed circuit board (PCB).
The hardware for this project is hosted in a seperate repository found [here](https://github.com/Ymit24/prandtl-hardware/tree/main).

## Project Structure
_(Here I use a workspace containing multiple crates which is Rust's terminology for a solution with multiple projects, if you're coming from a .NET background, for example.)_

This project is split between two applications across four crates.

| Crate | Description |
| ----- | ----------- |
| control_system | The application which runs on the host system. This application runs the control algorithm. |
| common | A library crate which contins common definitions such as `Temperture`, `Packet`, etc... |
| embedded_firmware | The embedded firmware application wihch runs on the microcontroller. |
| embedded_firmware_core | A library containing business-logic level code from the firmware which can be tested in isolation. |
| external_dependencies | Contains a local copy of the `arduino_mkrzero` board support crate due to versioning issues. |

#### Control System
The control system crate contains application code for the authoritative control server.
This software runs on a desktop computer (Windows, macOS, Linux) and communicates with the embedded system via USB.
In our prototype, we connected via the internal motherboard's USB2 header.

### Built With

- Host control system:
  - [Tokio](https://docs.rs/tokio/latest/tokio/), designed around interlinked tasks which communicate via channels.
  - [Postcard](https://docs.rs/postcard/latest/postcard/), no_std serialization for packets.
  - [Systemstat](https://docs.rs/systemstat/latest/systemstat/), cross-platform CPU temperature reporting.
  - [Tracing](https://docs.rs/tracing/latest/tracing/), for awesome logging.
- Embedded Firmware:
  - [Cortex M](https://docs.rs/cortex-m/latest/cortex_m/), hardware support crate for Cortex M microcontrollers.
  - [Postcard](https://docs.rs/postcard/latest/postcard/), no_std serialization for packets.
  - [Heapless](https://docs.rs/heapless/latest/heapless/), no_std, heapless data structures.
  - [Fixedstr](https://docs.rs/fixedstr/latest/fixedstr/), no_std, fixed width strings.
  - [Atsamd-hal](https://docs.rs/atsamd-hal/latest/atsamd_hal/), Hardware Abstraction Layer crate for the atsamd family of microcontrollers.
  - [Arduino Mkrzero](https://docs.rs/arduino_mkrzero/latest/arduino_mkrzero/), Arduino MkrZero board support crate.

## Getting Started
This software is designed specifically for use with the custom hardware, both electrical and mechanical. 
It is ill advised to try and set this up on your own due to risk of property damage. This section is here
for future research reference and for documentation.

### Prerequisites
This software doesn't have any major requirements to run on your computer, you just need a recent version of rust.
- Rust 1.77.2 or higher should work.

The embedded firmware was designed to run on a Cortex M0+ atsamd21g18a microcontroller.
The schematic's for the hardware, including pin assignments, can be found [here](https://github.com/Ymit24/prandtl-hardware/tree/main) on the hardware repo.

### Installation
Clone down this repository somewhere convenient. For the host machine, just run it!
```bash
cd too-hot-to-prandtl-control-system
cargo run
```

_Note: for a real production use case, run the control system application as a daemon launched by the OS on startup for maximal reliability._

For the embedded firmware, you'll need to get the software onto the microcontroller.
Personally, I used a [tigard](https://github.com/tigard-tools/tigard).
Connect up the tigard's pins from the cortex header to the PCB with the `Vcc, Gnd, SWIO, SWCLK` (reset pin not needed!).
Make sure to set the tigard to SWD mode and VTG mode.
_Note: This assumes working with the hardware I designed [here](https://github.com/Ymit24/prandtl-hardware/tree/main) and the tigard_.

As per tigard's instructions, you can compile OpenOCD and use it to program the hardware.
I recommend flashing the Arduino Zero's bootloader [here](https://github.com/arduino/ArduinoCore-samd/tree/master/bootloaders/zero) to make flashing easier.

Once the bootloader is installed and working, I double tap the reset button on the board and flash the firmware over USB using the `build_and_upload.pl` perl script.
This script automates compiling the project, copying the binary, and flashing using the `bossac` tool which is the same one used by the arduino IDE.
Depending on your port allocation, you might need to modify the script to flash to whichever port your device connected to.


## Usage
This system is designed to run autonomously on its own so once you start it there is nothing left to do!

## Roadmap
There are many features which I wish I had time to implement that I ran out of time and project scope to implement.
Future development for this project will be concluded May 10th, 2024. Below are a list of ideas that I wanted to implement.

- gRPC API server.
  - Allows a GUI program to display realtime info to the user about the system's performance and control system output. Could even include control to manually tune the system.
- More advanced control algorithms on the host and firmware to better handle possible edge cases.
- Automatically compute control parameters based on system parameters like chip TDP, tubing lengths, fluid properties, pump specs, etc...

## License
This project uses the GNU General Purpose License (GPL). Basically, do whatever you want but you must open source any derivative work!

## Contact
My name is Christian Smith and you can contact me through my linked in [here](https://www.linkedin.com/in/christian-ryan-smith/).
As this was a senior design project which has now concluded, further work and support of this project will not continue.
