use max32660_pac;// as pac;

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

pub struct UartRxFifo {
    size: u8,
    buffer: [u8; 8],
}

use libm::{floorf, powf};

// #[macro_export]
// macro_rules! uart {
//     ($UARTX:ident) => {

        pub struct UART {
            uart: max32660_pac::UART1,
        }

        impl UART {
            pub fn new(uart: max32660_pac::UART1) -> Self {
                UART {
                    uart: uart,
                }
            }

            pub fn enable(&self) {
                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        let bits = r.bits();
                        w.bits(bits).enable().bit(true)
                    })
                }
            }

            pub fn set_baud(&self, peripheral_clk: u32, baud: u32) -> Result<&Self, UartError> {

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
                    self.uart.baud0.write(|w| {
                        w.factor().bits((factor & 0x3) as u8);
                        w.ibaud().bits(ibaud as u16)
                    });

                    self.uart.baud1.write(|w| {
                        w.dbaud().bits(dbaud as u16)
                    })
                }

                Ok(self)
            }

            pub fn enable_interrupts(&self, ints: &[Interrupts]) -> &Self {
                let mut interrupt_final: u16 = 0;
                
                for int in ints {
                    interrupt_final |= *int as u16;
                }

                unsafe {
                    self.uart.int_en.write(|w| {
                        w.bits(interrupt_final as u32)
                    })
                }

                self
            }

            pub fn set_flow_control(&self, enable: bool, polarity: FlowControlPolarity) -> &Self {
                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        let pol = match polarity {
                            FlowControlPolarity::AssertZero => { false },
                            FlowControlPolarity::AssertOne => { true }
                        };

                        w.bits(r.bits()).flow_ctrl().bit(enable).flow_pol().bit(pol)
                    })
                }

                self
            }

            pub fn set_char_size(&self, s: CharSize) -> &Self {
                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        w.bits(r.bits()).char_size().bits(s as u8)
                    })
                }

                self
            }

            pub fn set_parity(&self, enable: bool, parity: Parity, level: ParityLevel) {
                todo!("Configure parity")
            }

            pub fn set_stop_bit(&self, stop: StopBits) -> &Self {
                

                let stop_bool = match stop {
                    StopBits::_1 => { false },
                    _ => { true }
                };

                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        if stop_bool {
                            w.bits(r.bits()).stopbits()._1()
                        }else{
                            w.bits(r.bits()).stopbits()._1_5()
                        }
                    })
                }

                self
            }

            pub fn flush_rx_fifo(&self) {
                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        w.bits(r.bits()).rx_flush().set_bit()
                    })
                }
            }

            pub fn flush_tx_fifo(&self) {
                unsafe {
                    self.uart.ctrl.modify(|r, w| {
                        w.bits(r.bits()).tx_flush().set_bit()
                    })
                }
            }

            pub fn set_rx_fifo_threshold(&self, threshold: FifoThreshold) -> &Self {
                unsafe {
                    self.uart.thresh_ctrl.modify(|r, w| {
                        w.bits(r.bits()).rx_fifo_thresh().bits(threshold as u8)
                    })
                }

                self
            }

            pub fn set_tx_fifo_threshold(&self, threshold: FifoThreshold) {
                unsafe {
                    self.uart.thresh_ctrl.modify(|r, w| {
                        w.bits(r.bits()).tx_fifo_thresh().bits(threshold as u8)
                    })
                }
            }

            pub fn set_rts_fifo_threshold(&self, threshold: FifoThreshold) {
                unsafe {
                    self.uart.thresh_ctrl.modify(|r, w| {
                        w.bits(r.bits()).rts_fifo_thresh().bits(threshold as u8)
                    })
                }
            }

            /// Read a byte from the UART FIFO buffer
            pub fn read(&self) -> u8 {
                self.uart.fifo.read().fifo().bits()
            }

            pub fn rx_fifo_cnt(&self) -> u8 {
                self.uart.status.read().rx_fifo_cnt().bits()
            }
            
            /// Writes a byte to the UART FIFO, returns true if the FIFO is full
            pub fn write(&self, byte: u8) -> bool {
                unsafe {
                    self.uart.fifo.write(|w| {
                        w.fifo().bits(byte)
                    })
                }

                self.tx_fifo_full()
            }

            pub fn write_blocking(&self, bytes: &[u8]) {
                let mut full = false;
                for byte in bytes {
                    full = self.write(*byte);
                    if full {
                        while self.tx_fifo_full(){} // block until there is room
                    }
                }  
            }

            pub fn tx_fifo_full(&self) -> bool {
                self.uart.status.read().tx_full().bit()
            }

            pub fn clear_interrupt(&self, ints: &[Interrupts]) {
                let mut interrupt_final: u32 = 0;
                
                for int in ints {
                    interrupt_final |= *int as u32;
                }

                unsafe {
                    self.uart.int_fl.write(|w| {
                        w.bits(interrupt_final)
                    })
                }
            }
        }
//     }
// }
