use embedded_hal as hal;
use max32660_pac;// as pac;
use nb;

// pub enum Error {
//     Overrun,
// }

// use max32660_pac::UART0;

// pub struct Serial<UART> { uart: UART }

// pub type Serial1 = Serial<UART0>;

// impl hal::serial::Read<u8> for Serial<UART0> {
//     type Error = Error;

//     fn read(&mut self) -> nb::Result<u8, Error> {
//         Ok(0)
//     }
// }

// impl hal::serial::Write<u8> for Serial<UART0> {
//     type Error = Error;

//     fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
//         Ok(())
//     }

//     fn flush(&mut self) -> nb::Result<(), Error> {
//         Ok(())
//     }
// }

// #[macro_export]
// macro_rules! uart_macro {
//     () => {
        
//     };
// }

// //pub type Serial0<max32660_pac::uart0> 