// #[macro_export]
// macro_rules! gpio_macro {
//    ($chip_crate: ident, $GPIOX: ident) => {

//    } 
// }

use embedded_hal;
use bitterly::Register32;
use max32660_pac::generic::Reg;

#[derive(Copy, Clone)]
pub enum Ports {
    _0,
}

#[derive(Copy, Clone)]
pub enum Pins {
    _0 = 0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
}

enum PinError {
    AlreadyAllocated,
}

struct Pin<'a> {
    pin: Pins,
    gpio: &'a max32660_pac::GPIO0,
}

pub struct OutputPin<'a> {
    pin: Pin<'a>,
}

pub struct InputPin<'a> {
    pin: Pin<'a>,
}

pub struct AlternateFunctionPin {}

#[derive(Copy, Clone)]
pub enum DriveStrength {
    _1ma,
    _2ma,
    _4ma,
    _6ma,
    _I2C_2ma,
    _I2C_10ma,
}

#[derive(Copy, Clone)]
pub enum Function {
    Input,
    Output,
    AF1,
    AF2,
    AF3,
}

#[derive(Copy, Clone)]
pub enum Resistor {
    HiZ,
    PullUp,
    PullDown,
}   

fn toBit32(pin: Pins) -> bitterly::Bit32 {
    match (pin as u32) {
        0 => { bitterly::Bit32::_0 },
        1 => { bitterly::Bit32::_1 },
        2 => { bitterly::Bit32::_2 },
        3 => { bitterly::Bit32::_3 },
        4 => { bitterly::Bit32::_4 },
        5 => { bitterly::Bit32::_5 },
        6 => { bitterly::Bit32::_6 },
        7 => { bitterly::Bit32::_7 },
        8 => { bitterly::Bit32::_8 },
        9 => { bitterly::Bit32::_9 },
        10 => { bitterly::Bit32::_10 },
        11 => { bitterly::Bit32::_11 },
        12 => { bitterly::Bit32::_12 },
        13 => { bitterly::Bit32::_13 },
        _ => { bitterly::Bit32::_0 }
    }
}

fn interrupts(gpio0: max32660_pac::GPIO0, enable: bool) {

}

fn create_pin(gpio0: &max32660_pac::GPIO0, pin: Pins, function: Function) {
    let en_regs = match function {
        Function::AF1 => { [false, false, false] },
        Function::AF2 => { [false, true, false] },
        Function::AF3 => { [true, true, false] },
        Function::Input => { [true, false, false] },
        Function::Output => { [true, false, false] }
    };
    
    unsafe {
        let mut reg = 0;
        gpio0.en.modify(|r, w| {  
            if en_regs[0] {
                reg = Register32::new(r.bits()).set(toBit32(pin)).value();
            }else{
                reg = Register32::new(r.bits()).clear(toBit32(pin)).value();
            } 
            w.bits(reg)
        });
        gpio0.en1.modify(|r, w| {
            if en_regs[1] {
                reg = Register32::new(r.bits()).set(toBit32(pin)).value();
            }else{
                reg = Register32::new(r.bits()).clear(toBit32(pin)).value();
            } 
            w.bits(reg)
        });
        gpio0.en2.modify(|r, w| {
            if en_regs[2] {
                reg = Register32::new(r.bits()).set(toBit32(pin)).value();
            }else{
                reg = Register32::new(r.bits()).clear(toBit32(pin)).value();
            } 
            w.bits(reg)
        });
    }
}


/// Create's an input pin, see GPIO -> InputMode Configuration
pub fn create_input_pin(gpio0: &max32660_pac::GPIO0, pin: Pins, resistor: Resistor) -> InputPin {
    create_pin(gpio0, pin, Function::Input);

    let mut pad = false;
    let mut ps = false;

    match resistor {
        Resistor::HiZ => { pad = false; ps = false; }
        Resistor::PullUp => { pad = true; ps = true; }
        Resistor::PullDown => { pad = true; ps = false; }
    }

    // Note pad_cfg2 is not used for this part
    unsafe {
        gpio0.pad_cfg1.modify(|r, w| {
            let mut reg = Register32::new(r.bits());
            if pad {
                reg = reg.set(toBit32(pin));
            }else{
                reg = reg.clear(toBit32(pin));
            }
            w.bits(reg.value())
        });

        gpio0.ps.modify(|r, w| {
            let mut reg = Register32::new(r.bits());
            if ps {
                reg = reg.set(toBit32(pin));
            }else{
                reg = reg.clear(toBit32(pin));
            }
            w.bits(reg.value())
        });
    }

    InputPin{
        pin: Pin {
            pin: pin,
            gpio: gpio0,
        } 
    }
}

pub enum InputPinErrors {
    UnspecifiedError,
}

impl InputPin<'_> {
    fn read(self) -> Result<bool, InputPinErrors> {
        let reg = Register32::new(self.pin.gpio.in_.read().bits());
        Ok(reg.is_set(toBit32(self.pin.pin)))
    }
}

pub fn create_output_pin(gpio0: &max32660_pac::GPIO0, pin: Pins, drive: DriveStrength) -> OutputPin {
    create_pin(gpio0, pin, Function::Output);

    let ds = match drive {
        DriveStrength::_1ma => { [ false, false ] },
        DriveStrength::_2ma => { [true, false ] },
        DriveStrength::_4ma => { [false, true] },
        DriveStrength::_6ma => { [true, true] },
        DriveStrength::_I2C_2ma => { [false, false] },
        DriveStrength::_I2C_10ma => { [true, false] },
    };

    unsafe {
        gpio0.out_en.modify(|r, w| {
            let mut reg = Register32::new(r.bits());
            reg = reg.set(toBit32(pin));
            w.bits(reg.value())
        });

        gpio0.ds.modify(|r, w| {
            let reg = Register32::new(r.bits());
            if ds[0] {
                w.bits(reg.set(toBit32(pin)).value())   
            }else{
                w.bits(reg.clear(toBit32(pin)).value())  
            }
        });

        gpio0.ds1.modify(|r, w| {
            let reg = Register32::new(r.bits());
            if ds[1] {
                w.bits(reg.set(toBit32(pin)).value())   
            }else{
                w.bits(reg.clear(toBit32(pin)).value())  
            }
        });
    }

    OutputPin{
        pin: Pin {
            pin: pin,
            gpio: gpio0,
        } 
    }
}

pub enum OutputPinErrors {
    UnspecifiedError,
}

impl OutputPin<'_> {
    fn pin_high(self) -> Result<(), OutputPinErrors> {
        unsafe {
            self.pin.gpio.out_set.write(|w| {
                w.bits(toBit32(self.pin.pin) as u32)
            });
        }
        Ok(())
    }

    fn pin_low(self) -> Result<(), OutputPinErrors> {
        unsafe {
            self.pin.gpio.out_clr.write(|w| {
                w.bits(toBit32(self.pin.pin) as u32)
            });
        }
        Ok(())
    }
}

pub fn create_alternate_function_pin(gpio0: &max32660_pac::GPIO0, pin: Pins, function: Function) -> AlternateFunctionPin {
    create_pin(gpio0, pin, function);
    AlternateFunctionPin {  }
}
