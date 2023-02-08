// #[macro_export]
// macro_rules! gpio_macro {
//    ($chip_crate: ident, $GPIOX: ident) => {

//    } 
// }

use embedded_hal;
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

fn interrupts(gpio0: max32660_pac::GPIO0, enable: bool) {

}

fn pin_mask(pin: u8) -> u32 {
    1 << pin
}

fn pin_clear(input: u32, pin: u8) -> u32 {
    input & !pin_mask(pin)
}

fn pin_set(input: u32, pin: u8) -> u32 {
    input & pin_mask(pin)
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
                reg = pin_set(r.bits(), pin as u8);
            }else{
                reg = pin_clear(r.bits(), pin as u8);
            } 
            w.bits(reg)
        });
        gpio0.en1.modify(|r, w| {
            if en_regs[1] {
                reg = pin_set(r.bits(), pin as u8);
            }else{
                reg = pin_clear(r.bits(), pin as u8);
            } 
            w.bits(reg)
        });
        gpio0.en2.modify(|r, w| {
            if en_regs[2] {
                reg = pin_set(r.bits(), pin as u8);
            }else{
                reg = pin_clear(r.bits(), pin as u8);
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
            let mut reg = r.bits();
            if pad {
                reg = pin_set(reg, pin as u8);
            }else{
                reg = pin_clear(reg, pin as u8);
            }
            w.bits(reg)
        });

        gpio0.ps.modify(|r, w| {
            let mut reg = r.bits();
            if ps {
                reg = pin_set(reg, pin as u8);
            }else{
                reg = pin_clear(reg, pin as u8);
            }
            w.bits(reg)
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
        let mut reg = self.pin.gpio.in_.read().bits();
        reg = reg >> self.pin.pin as u8;
        if reg == 0 {
            Ok(false)
        }else{
            Ok(true)
        }
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
            let mut reg = r.bits();
            reg = pin_set(reg, pin as u8);
            w.bits(reg)
        });

        gpio0.ds.modify(|r, w| {
            let reg = r.bits();
            if ds[0] {
                w.bits(pin_set(reg, pin as u8))   
            }else{
                w.bits(pin_clear(reg, pin as u8))  
            }
        });

        gpio0.ds1.modify(|r, w| {
            let reg = r.bits();
            if ds[1] {
                w.bits(pin_set(reg, pin as u8))   
            }else{
                w.bits(pin_clear(reg, pin as u8))  
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
                w.bits(pin_set(0, self.pin.pin as u8))
            });
        }
        Ok(())
    }

    fn pin_low(self) -> Result<(), OutputPinErrors> {
        unsafe {
            self.pin.gpio.out_clr.write(|w| {
                w.bits(pin_set(0, self.pin.pin as u8))
            });
        }
        Ok(())
    }
}

pub fn create_alternate_function_pin(gpio0: &max32660_pac::GPIO0, pin: Pins, function: Function) -> AlternateFunctionPin {
    create_pin(gpio0, pin, function);
    AlternateFunctionPin {  }
}
