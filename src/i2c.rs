use max32660_pac;

const TCLK_STD: f32 = 1.0 / 400_000.0;

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

const READ_BIT: u8 = 0x80;
const WRITE_BIT: u8 = 0x00;

pub struct I2C {
    i2c: max32660_pac::I2C1,
    busy: bool,
    ack: bool,
}

impl I2C {

    pub fn new(i2c: max32660_pac::I2C1) -> Self {
        I2C {
            i2c: i2c,
            busy: false,
            ack: false,
        }
    } 

    pub fn enable(&self, peripheral_clock: u32, i2c_speed: Clock) {
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
    }

    pub fn master_mode(&self) -> &Self{
        unsafe {
            self.i2c.ctrl.modify(|r, w| {
                w.bits(r.bits()).bits(0x2)
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
            ints += (*i as u16);
        }

        unsafe {
            self.i2c.int_en0.write(|w| w.bits(ints as u32))  
        }
        self
    }

    pub fn enable_interrupts1(&self, interrupts: &[Interrupts1]) -> &Self {
        let mut ints = 0;
        for i in interrupts {
            ints += (*i as u16);
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

    pub fn is_busy(self) -> bool {
        self.busy
    }

    pub fn tx_fifo_len(&self) -> u8 {
        self.i2c.tx_ctrl1.read().tx_fifo().bits() as u8
    }

    pub fn rx_fifo_len(&self) -> u8 {
        self.i2c.rx_ctrl1.read().rx_fifo().bits() as u8
    }

    pub fn master_write_blocking(&self, data: &[u8]) {
        // // TODO: Flush TX fifo first?

        // // Write data to buffer
        // for b in data {
        //     unsafe {
        //         self.i2c.fifo.write(|w| w.data().bits(*b))
        //     }
        // }

        // unsafe {
        //     // Put device in master mode
        //     self.i2c.ctrl.modify(|r, w| {
        //         w.bits(r.bits()).mst().set_bit()
        //     });

        //     self.i2c.master_ctrl.modify(|r, w| {
        //         w.bits(r.bits()).start().set_bit()
        //     });
        // }
    }

    pub fn master_read(&mut self, slave_address: u8, memory_address: u8, bytes_to_read: u8) {
        unsafe {
            self.i2c.rx_ctrl1.modify(|r, w| {
                w.bits(r.bits()).rx_cnt().bits(bytes_to_read)
            });

            self.i2c.fifo.write(|w| {
                w.data().bits(slave_address)
            });
            
            self.i2c.master_ctrl.modify(|r, w| {
                w.bits(r.bits()).start().set_bit()
            });
        }

        self.busy = true;
    }

    //pub fn 

}