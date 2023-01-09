#![no_std]

use max32660_pac::{self, UART0};// as pac;

pub mod gpio;
pub mod uart;
pub mod sys;
