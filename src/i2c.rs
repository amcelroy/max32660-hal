use max32660_pac;

#[derive(Copy, Clone)]
pub enum Interrupts0 {
    Done = 0x1,
    RxMode = 0x2,
    GenCtrlAddr = 0x4,
    AddrMatch = 0x8,
    RxThresh = 0x10,
    TxThresh = 0x20,
    Stop = 0x40,
    AddrAck = 0x80,
    ArbEr = 0x100,
    ToEr = 0x200,
    AddrEr = 0x400,
    DataEr = 0x800,
    DoNotRespondEr = 0x1000,
    StartEr = 0x2000,
    StopEr = 0x4000,
    TxLockOut = 0x8000,
}

#[derive(Copy, Clone)]
pub enum Interrupts1 {
    RxOverflow = 0x1,
    TxUnderflow = 0x2,
}

pub enum Clock {
    Standard = 100000,
    Fast = 400000,
    FastPlus = 1000000,
}

pub enum IntThresh {
    _0 = 0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
}

pub enum I2CWriteError {
    NACK,
}

pub enum I2CReadError {
    NACK,
}

const READ_BIT: u8 = 0x1;
const WRITE_BIT: u8 = 0x0;
const ADDRESS_10BIT: u8 = 0xF0;

pub struct I2C<const BUFFER_SIZE: usize> {
    i2c: max32660_pac::I2C1,
    busy: bool,
    bytes_read: usize,
    buffer: [u8; BUFFER_SIZE],
}

impl<const BUFFER_SIZE: usize> I2C<BUFFER_SIZE> {

    pub fn new(i2c: max32660_pac::I2C1) -> Self {
        assert!(BUFFER_SIZE < 255, "BUFFER_SIZE should be 0 to 255");
        
        I2C {
            i2c: i2c,
            busy: false,
            bytes_read: 0,
            buffer: [0; BUFFER_SIZE],
        }
    } 

    pub fn is_busy(&self) -> bool {
        self.busy
    }

    pub fn clear_busy(&mut self) {
        self.busy = false;
    }

    pub fn not_busy(&self) -> bool {
        !self.busy
    }

    pub fn enable(&self, peripheral_clock: u32, i2c_speed: Clock) -> &Self{
        let tclock: u32 = (peripheral_clock) / (i2c_speed as u32);

        let t_hi = ((tclock >> 1) - 1) as u8;
        let t_low = ((tclock >> 1) - 1) as u8;

        unsafe {
            self.i2c.clk_hi.write(|w| w.bits(t_hi as u32));
            self.i2c.clk_lo.write(|w| w.bits(t_low as u32));

            self.i2c.ctrl.modify(|r, w| {
                w.bits(r.bits()).i2c_en().en().mst().master_mode()
            }); 
        }
        self
    }

    pub fn master_mode(&self) -> &Self{
        unsafe {
            self.i2c.ctrl.modify(|r, w| {
                w.bits(r.bits()).mst().master_mode()
            });
        }
        self
    }

    pub fn rx_int_threshold(&self, threshold: IntThresh) -> &Self {
        unsafe {
            self.i2c.rx_ctrl0.modify(|r, w| {
                w.bits(r.bits()).rx_thresh().bits(threshold as u8)
            })
        }
        self
    }

    pub fn enable_interrupts0(&self, interrupts: &[Interrupts0]) -> &Self {
        let mut ints = 0;
        for i in interrupts {
            ints += *i as u16;
        }

        unsafe {
            self.i2c.int_en0.write(|w| w.bits(ints as u32))  
        }
        self
    }

    pub fn enable_interrupts1(&self, interrupts: &[Interrupts1]) -> &Self {
        let mut ints = 0;
        for i in interrupts {
            ints += *i as u16;
        }

        unsafe {
            self.i2c.int_en1.write(|w| w.bits(ints as u32))  
        }
        self
    }

    pub fn get_interrupt0(&self) -> u16 {
        self.i2c.int_fl0.read().bits() as u16
    }

    pub fn clear_interrupt0(&self, ints: u16) {
        unsafe {
            self.i2c.int_fl0.write(|w| {
                w.bits(ints as u32)
            });
        }
    }

    pub fn tx_fifo_len(&self) -> u8 {
        self.i2c.tx_ctrl1.read().tx_fifo().bits() as u8
    }

    pub fn rx_fifo_len(&self) -> u8 {
        self.i2c.rx_ctrl1.read().rx_fifo().bits() as u8
    }

    pub fn master_stop(&mut self) {
        self.busy = false;
        self.i2c.master_ctrl.write(|w| {
            w.stop().set_bit()
        });

        unsafe {
            self.i2c.int_fl0.modify(|r, w| {
                w.bits(r.bits()).tx_lock_out().set_bit()
            });
        }

        while !self.i2c.status.read().status().is_idle() {}
    }

    fn _10bit_address_to_bytes(address: u16, read: bool) -> [u8; 2] {
        let mut retval = [0; 2];

        // 10 bit requires 2 bytes:
        // first byte: 1 1 1 1 0 A9 A8 R/W
        // 2nd byte: A7 A6 A5 A4 A3 A2 A1 A0
        
        //                  Get A9 A8       >> shift to bit 2 and 1, then clear 0 bit

        let upper = ((address & 0x300) >> 7) & !0x1;
        retval[0] = (upper as u8) | ADDRESS_10BIT | (if read { READ_BIT } else { WRITE_BIT });
        retval[1] = (address & 0x7F) as u8;

        retval
    }

    fn buffer_tx_data(&mut self, data: &[u8], data_offset: &mut usize) -> usize {
        while self.i2c.status.read().tx_full().bit_is_clear() && (*data_offset < data.len()) {
            unsafe {
                self.i2c.fifo.write(|w| w.data().bits(data[*data_offset]));
                *data_offset += 1;
            }
        }
        *data_offset
    }

    pub fn master_write(&mut self, address: u16, data: &[u8], _10_bit: bool)-> Result<(), I2CWriteError> {
        // See Max32660 User Guide Section 14.4.6.2 I2C Master Tx Operation

        self.busy = true;

        let mut data_offset = 0;

        unsafe {
            if _10_bit {
                let address_as_array = Self::_10bit_address_to_bytes(address, false);

                self.i2c.master_ctrl.modify(|r, w| {
                    w.bits(r.bits()).sl_ex_addr().set_bit()
                });

              
                for byte in address_as_array {
                    self.i2c.fifo.write(|w| w.data().bits(byte));
                }
            }else{
                let address_u8 = (address as u8) << 1;
                self.i2c.fifo.write(|w| w.data().bits(address_u8));
            }

            // 1. Write initial data to buffer, R/W bit is 0
            data_offset = self.buffer_tx_data(data, &mut data_offset);
            
            // 2. Send start
            self.i2c.master_ctrl.modify(|r, w| {
                w.bits(r.bits()).start().set_bit()
            });

            // Keep adding data as needed
            let mut keep_buffering = self.i2c.status.read().tx_empty().bit_is_clear();
            while keep_buffering {
                if data_offset < data.len() {
                    data_offset = self.buffer_tx_data(data, &mut data_offset);
                }

                if self.i2c.status.read().status().is_tx() {
                    keep_buffering = true;
                } else {
                    keep_buffering = false;
                }
                
                if self.i2c.status.read().status().is_nack() {
                    self.master_stop();
                    return Err(I2CWriteError::NACK);
                }
            }

            self.master_stop();
            
            Ok(())
        }
    }

    pub fn master_read(&mut self, address: u16, bytes_to_read: u8, _10_bit: bool) -> Result<usize, I2CReadError> {
        // See Max32660 User Guide Section 14.4.6.1 I2C Master Receiver Operation

        self.busy = true;
        self.bytes_read = 0;

        let mut bytes_to_read_adjusted: usize = bytes_to_read as usize;
        if bytes_to_read == 0 {
            bytes_to_read_adjusted = 256;
        }

        unsafe {
            // 1. Write number of bytes to fetch
            self.i2c.rx_ctrl1.modify(|r, w| {
                w.bits(r.bits()).rx_cnt().bits(bytes_to_read)
            });

            // Write address to TX FIFO
            if _10_bit {
                let address_as_array = Self::_10bit_address_to_bytes(address, true);

                self.i2c.master_ctrl.modify(|r, w| {
                    w.bits(r.bits()).sl_ex_addr().set_bit()
                });

                for byte in address_as_array {
                    self.i2c.fifo.write(|w| w.data().bits(byte));
                }
            }else{
                let address_u8 = (address as u8) << 1;
                self.i2c.fifo.write(|w| w.data().bits(address_u8 | READ_BIT));
            }

            // Start TX of Address with R/W bit set to 1
            self.i2c.master_ctrl.modify(|r, w| {
                w.bits(r.bits()).start().set_bit()
            });

            let mut keep_reading = true;

            while keep_reading {

                while self.i2c.status.read().rx_empty().bit_is_clear() {
                    self.buffer[self.bytes_read] = self.i2c.fifo.read().data().bits();
                    self.bytes_read += 1;
                }

                if self.bytes_read >= bytes_to_read_adjusted {
                    keep_reading = false;
                }

                if self.i2c.status.read().status().is_idle() {
                    keep_reading = false;
                }

                if self.i2c.status.read().status().is_nack() {
                    keep_reading = false;
                }

                if self.i2c.status.read().status().is_nack() && self.bytes_read < bytes_to_read_adjusted {
                    self.master_stop();
                    return Err(I2CReadError::NACK);
                }
            }

            self.master_stop();

            Ok(self.bytes_read)
        }
    }

    //pub fn 

}