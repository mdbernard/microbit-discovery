#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{prelude::*, Timer},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
 
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut image = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    let mut row = 0;
    let mut col = 0;

    loop {
        image[row][col] = 0;

        if col < 4 && row == 0 {
            col += 1;
        } else if 0 < col  && row == 4 {
            col -= 1;
        } else if col == 4 && row < 4 {
            row += 1;
        } else if col == 0 && row > 0 {
            row -= 1;
        }

        image[row][col] = 1;

        display.show(&mut timer, image, 1000);
    }
}

