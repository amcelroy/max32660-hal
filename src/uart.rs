use embedded_hal as hal;
use max32660_pac;// as pac;
use nb;

use embedded_hal;
use libm::{powf, floorf};

pub enum Parity {
    Even,
    Odd,
    Mark,
    Space,
    None
}

pub enum ParityLevel {
    Zeros = 0,
    Ones = 1,
}

pub enum StopBits {
    _1,
    _1_5,
    _2
}

pub enum FlowControlPolarity {
    AssertZero,
    AssertOne,
}

pub enum CharSize {
    _5,
    _6,
    _7,
    _8,
}

pub enum FifoThreshold {
    _1 = 1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7
}

#[derive(Copy, Clone)]
pub enum Interrupts {
    RxFrameError = 0x1,
    RxParityError = 0x2,
    CtsChange = 0x4,
    RxOverrun = 0x8,
    RxFifoThresh = 0x10,
    TxFifoAlmostEmpty = 0x20,
    TxFifoThresh = 0x40,
    Break = 0x80,
    RxTimeout = 0x100,
    LastBreak = 0x200,
}

#[derive(Debug)]
pub enum UartError {
    BaudRateConfiguration,
}

// #[macro_export]
// macro_rules! uart {
//     ($uart:ident) => {
        pub fn enable_uart(uart: &max32660_pac::UART1, enable: bool) {
            unsafe {
                uart.ctrl.modify(|r, w| {
                    let bits = r.bits();
                    w.bits(bits).enable().bit(enable)
                })
            }
        }

        pub fn set_baud(uart: &max32660_pac::UART1, peripheral_clk: u32, baud: u32) -> Result<(), UartError> {

            let mut div: f32 = 0.0;
            let mut factor = 0;
            for factor in 0..4 {
                div = (peripheral_clk as f32);
                let dividend = powf(2.0, ((7 - factor)) as f32)* (baud as f32);
                div = div / dividend;
                if div > 1.0 {
                    break;
                }
            }   

            if div < 1.0 {
                return Err(UartError::BaudRateConfiguration);
            }

            let ibaud = floorf(div);

            let mut dbaud: f32 = (div - ibaud) * 128.0;
            if dbaud > 3.0 {
                dbaud -= 3.0;
            }else{
                dbaud += 3.0;
            }

            unsafe {
                uart.baud0.write(|w| {
                    w.factor().bits((factor & 0x3) as u8);
                    w.ibaud().bits(ibaud as u16)
                });

                uart.baud1.write(|w| {
                    w.dbaud().bits(dbaud as u16)
                })
            }

            Ok(())
        }

        pub fn enable_interrupts(uart: &max32660_pac::UART1, ints: &[Interrupts]) {
            let mut interrupt_final: u16 = 0;
            
            for int in ints {
                interrupt_final |= *int as u16;
            }

            unsafe {
                uart.int_en.write(|w| {
                    w.bits(interrupt_final as u32)
                })
            }
        }

        pub fn set_flow_control(uart: &max32660_pac::UART1, enable: bool, polarity: FlowControlPolarity) {
            unsafe {
                uart.ctrl.modify(|r, w| {
                    let pol = match polarity {
                        FlowControlPolarity::AssertZero => { false },
                        FlowControlPolarity::AssertOne => { true }
                    };

                    w.bits(r.bits()).flow_ctrl().bit(enable).flow_pol().bit(pol)
                })
            }
        }

        pub fn set_char_size(uart: &max32660_pac::UART1, s: CharSize) {
            unsafe {
                uart.ctrl.modify(|r, w| {
                    w.bits(r.bits()).char_size().bits(s as u8)
                })
            }
        }

        pub fn set_parity(uart: &max32660_pac::UART1, enable: bool, parity: Parity, level: ParityLevel) {
            todo!("Configure parity")
        }

        pub fn set_stop_bit(uart: &max32660_pac::UART1, stop: StopBits) {
            

            let stop_bool = match stop {
                StopBits::_1 => { false },
                _ => { true }
            };

            unsafe {
                uart.ctrl.modify(|r, w| {
                    if stop_bool {
                        w.bits(r.bits()).stopbits()._1()
                    }else{
                        w.bits(r.bits()).stopbits()._1_5()
                    }
                })
            }
        }

        pub fn flush_rx_fifo(uart: &max32660_pac::UART1) {
            unsafe {
                uart.ctrl.modify(|r, w| {
                    w.bits(r.bits()).rx_flush().set_bit()
                })
            }
        }

        pub fn flush_tx_fifo(uart: &max32660_pac::UART1) {
            unsafe {
                uart.ctrl.modify(|r, w| {
                    w.bits(r.bits()).tx_flush().set_bit()
                })
            }
        }

        pub fn set_rx_fifo_threshold(uart: &max32660_pac::UART1, threshold: FifoThreshold) {
            unsafe {
                uart.thresh_ctrl.modify(|r, w| {
                    w.bits(r.bits()).rx_fifo_thresh().bits(threshold as u8)
                })
            }
        }

        pub fn set_tx_fifo_threshold(uart: &max32660_pac::UART1, threshold: FifoThreshold) {
            unsafe {
                uart.thresh_ctrl.modify(|r, w| {
                    w.bits(r.bits()).tx_fifo_thresh().bits(threshold as u8)
                })
            }
        }

        pub fn set_rts_fifo_threshold(uart: &max32660_pac::UART1, threshold: FifoThreshold) {
            unsafe {
                uart.thresh_ctrl.modify(|r, w| {
                    w.bits(r.bits()).rts_fifo_thresh().bits(threshold as u8)
                })
            }
        }

        /// Read a byte from the UART FIFO buffer
        pub fn read(uart: &max32660_pac::UART1) -> u8 {
            uart.fifo.read().fifo().bits()
        }

        /// Writes a byte to the UART FIFO, returns number of bytes in the FIFO
        pub fn write(uart: &max32660_pac::UART1, byte: u8) -> u8 {
            unsafe {
                uart.fifo.write(|w| {
                    w.fifo().bits(byte)
                })
            }

            uart.tx_fifo.read().data().bits()
        }

        pub fn clear_interrupt(uart: &max32660_pac::UART1, ints: &[Interrupts]) {
            let mut interrupt_final: u32 = 0;
            
            for int in ints {
                interrupt_final |= *int as u32;
            }

            unsafe {
                uart.int_fl.write(|w| {
                    w.bits(interrupt_final)
                })
            }
        }
//     }
// }
