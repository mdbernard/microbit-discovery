#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;

use cortex_m_rt::entry;
use heapless::Vec;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;

use microbit::hal::prelude::*;

use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
    hal::uarte,
    hal::uarte::{Baudrate, Instance, Parity},
};

mod serial_setup;
use serial_setup::UartePort;

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

const ACCELEROMETER_ADDR: u8 = 0b0011001;
const MAGNETOMETER_ADDR: u8 = 0b0011110;

const ACCELEROMETER_ID_REG: u8 = 0x0f;  // WHO_AM_I_A
const MAGNETOMETER_ID_REG: u8 = 0x4f;  // WHO_AM_I_M

const ENTER_KEY: u8 = 0x0D;  // ASCII for CR

enum Command {
    Accelerometer,
    Magnetometer,
}


fn read_command<T: Instance>(serial: &mut UartePort<T>) -> Command {
    let mut buffer: Vec<u8, 13> = Vec::new();

    loop {
        let byte = nb::block!(serial.read()).unwrap();

        write!(serial, "{}", core::str::from_utf8(&[byte]).unwrap()).unwrap();
        nb::block!(serial.flush()).unwrap();

        if byte == ENTER_KEY {
            if buffer.as_slice() == [b'a', b'c', b'c', b'e', b'l', b'e', b'r', b'o', b'm', b'e', b't', b'e', b'r'] {
                write!(serial, "\r\n").unwrap();
                nb::block!(serial.flush()).unwrap();
                return Command::Accelerometer;
            }
            if buffer.as_slice() == [b'm', b'a', b'g', b'n', b'e', b't', b'o', b'm', b'e', b't', b'e', b'r'] {
                write!(serial, "\r\n").unwrap();
                nb::block!(serial.flush()).unwrap();
                return Command::Magnetometer;
            }
            write!(serial, "\r\nUnknown command\r\n").unwrap();
            nb::block!(serial.flush()).unwrap();
            buffer.clear();
            continue;
        }

        if buffer.push(byte).is_err() {
            write!(serial, "error: buffer full\r\n").unwrap();
            buffer.clear();
        }
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut acc = [0];
    let mut mag = [0];

    // First write the address + register onto the bus, then read the chip's responses
    i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc).unwrap();
    i2c.write_read(MAGNETOMETER_ADDR, &[MAGNETOMETER_ID_REG], &mut mag).unwrap();

    assert_eq!(acc[0], 0b110011);
    assert_eq!(mag[0], 0b1000000);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let mut command: Command;

    loop {
        command = read_command(&mut serial);

        match command {
            Command::Accelerometer => {
                if sensor.accel_status().unwrap().xyz_new_data {
                    let data = sensor.accel_data().unwrap();
                    write!(&mut serial, "Acceleration: x {} y {} z {}\r\n", data.x, data.y, data.z).unwrap();
                    nb::block!(serial.flush()).unwrap();
                } else {
                    write!(&mut serial, "Acceleration: no new data\r\n").unwrap();
                    nb::block!(serial.flush()).unwrap();
                }
            },
            Command::Magnetometer => {
                if sensor.mag_status().unwrap().xyz_new_data {
                    let data = sensor.mag_data().unwrap();
                    write!(&mut serial, "Magnetic Field: x {} y {} z {}\r\n", data.x, data.y, data.z).unwrap();
                    nb::block!(serial.flush()).unwrap();
                } else {
                    write!(&mut serial, "Magnetic Field: no new data\r\n").unwrap();
                    nb::block!(serial.flush()).unwrap();
                }
            },
        };
    }
}
