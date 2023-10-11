use max32660_pac;

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

#[derive(PartialEq)]
pub enum InterruptLevelPolarity {
    Low = 0,
    High = 1,
}

#[derive(PartialEq)]
pub enum InterruptEdgePolarity {
    Falling = 0,
    Rising = 1,
}

#[derive(PartialEq)]
pub enum InterruptMode {
    Level = 0,
    Edge = 1,
}

enum PinError {
    AlreadyAllocated,
}

#[derive(Copy, Clone)]
struct Pin {
    pin: Pins,
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

fn pin_mask(pin: u8) -> u32 {
    1 << pin
}

fn pin_clear(input: u32, pin: u8) -> u32 {
    input & !pin_mask(pin)
}

fn pin_set(input: u32, pin: u8) -> u32 {
    input | pin_mask(pin)
}

#[macro_export]
macro_rules! gpio {
    ($GPIOX:ty, $name:ident) => {
        pub struct $name {
            gpio: $GPIOX,
        }

        pub struct OutputPin {
            pin: Pins,
        }

        pub struct InputPin {
            pin: Pins,
        }

        impl $name {
            pub fn new(gpio: $GPIOX) -> Self {
                $name { gpio: gpio }
            }

            fn create_pin(&self, pin: Pins, function: Function) {
                let en_regs = match function {
                    Function::AF1 => [false, false, false],
                    Function::AF2 => [false, true, false],
                    Function::AF3 => [true, true, false],
                    Function::Input => [true, false, false],
                    Function::Output => [true, false, false],
                };

                unsafe {
                    let mut reg = 0;
                    self.gpio.en.modify(|r, w| {
                        if en_regs[0] {
                            reg = pin_set(r.bits(), pin as u8);
                        } else {
                            reg = pin_clear(r.bits(), pin as u8);
                        }
                        w.bits(reg)
                    });
                    self.gpio.en1.modify(|r, w| {
                        if en_regs[1] {
                            reg = pin_set(r.bits(), pin as u8);
                        } else {
                            reg = pin_clear(r.bits(), pin as u8);
                        }
                        w.bits(reg)
                    });
                    self.gpio.en2.modify(|r, w| {
                        if en_regs[2] {
                            reg = pin_set(r.bits(), pin as u8);
                        } else {
                            reg = pin_clear(r.bits(), pin as u8);
                        }
                        w.bits(reg)
                    });
                }
            }

            pub fn create_alternate_function_pin(
                &self,
                pin: Pins,
                function: Function,
            ) -> AlternateFunctionPin {
                self.create_pin(pin, function);
                AlternateFunctionPin {}
            }

            /// Create's an input pin, see GPIO -> InputMode Configuration
            pub fn create_input_pin(&mut self, pin: Pins, resistor: Resistor) -> InputPin {
                self.create_pin(pin, Function::Input);

                let mut pad = false;
                let mut ps = false;

                match resistor {
                    Resistor::HiZ => {
                        pad = false;
                        ps = false;
                    }
                    Resistor::PullUp => {
                        pad = true;
                        ps = true;
                    }
                    Resistor::PullDown => {
                        pad = true;
                        ps = false;
                    }
                }

                // Note pad_cfg2 is not used for this part
                unsafe {
                    self.gpio.pad_cfg1.modify(|r, w| {
                        let mut reg = r.bits();
                        if pad {
                            reg = pin_set(reg, pin as u8);
                        } else {
                            reg = pin_clear(reg, pin as u8);
                        }
                        w.bits(reg)
                    });

                    self.gpio.ps.modify(|r, w| {
                        let mut reg = r.bits();
                        if ps {
                            reg = pin_set(reg, pin as u8);
                        } else {
                            reg = pin_clear(reg, pin as u8);
                        }
                        w.bits(reg)
                    });
                }

                InputPin {
                    pin: pin,
                }
            }

            pub fn create_output_pin(&self, pin: Pins, drive: DriveStrength) -> OutputPin {
                self.create_pin(pin, Function::Output);

                let ds = match drive {
                    DriveStrength::_1ma => [false, false],
                    DriveStrength::_2ma => [true, false],
                    DriveStrength::_4ma => [false, true],
                    DriveStrength::_6ma => [true, true],
                    DriveStrength::_I2C_2ma => [false, false],
                    DriveStrength::_I2C_10ma => [true, false],
                };

                unsafe {
                    self.gpio.out_en.modify(|r, w| {
                        let mut reg = r.bits();
                        reg = pin_set(reg, pin as u8);
                        w.bits(reg)
                    });

                    self.gpio.ds.modify(|r, w| {
                        let reg = r.bits();
                        if ds[0] {
                            w.bits(pin_set(reg, pin as u8))
                        } else {
                            w.bits(pin_clear(reg, pin as u8))
                        }
                    });

                    self.gpio.ds1.modify(|r, w| {
                        let reg = r.bits();
                        if ds[1] {
                            w.bits(pin_set(reg, pin as u8))
                        } else {
                            w.bits(pin_clear(reg, pin as u8))
                        }
                    });
                }

                OutputPin {
                    pin: pin,
                }
            }
        }

        impl OutputPin {
            pub fn pin_high(self) {
                unsafe {
                    (*<$GPIOX>::ptr())
                        .out_set
                        .write(|w| w.bits(pin_set(0, self.pin as u8)));
                }
            }

            pub fn pin_low(self) {
                unsafe {
                    (*<$GPIOX>::ptr())
                        .out_clr
                        .write(|w| w.bits(pin_set(0, self.pin as u8)));
                }
            }
        }

        impl InputPin {
            pub fn read(self) -> bool {
                unsafe {
                    let mut reg = (*<$GPIOX>::ptr()).in_.read().bits();
                    reg &= 1 << self.pin as u8;
                    if reg == 0 {
                        false
                    } else {
                        true
                    }
                }
            }

            pub fn is_interrupting(self) -> bool {
                unsafe {
                    let mut reg = (*<$GPIOX>::ptr()).int_stat.read().bits();
                    reg &= 1 << self.pin as u8;
                    if reg == 0 {
                        false
                    } else {
                        true
                    }
                }
            }

            pub fn enable_interrupt(&self) {
                unsafe {
                    (*<$GPIOX>::ptr()).int_en.modify(|r, w| {
                        let mut reg = r.bits();
                        reg = pin_set(reg, self.pin as u8);
                        w.bits(reg)
                    });
                }
            }

            pub fn disable_interrupt(&self) {
                unsafe {
                    (*<$GPIOX>::ptr()).int_en.modify(|r, w| {
                        let mut reg = r.bits();
                        reg = pin_clear(reg, self.pin as u8);
                        w.bits(reg)
                    });
                }
            }

            pub fn clear_interrupt(&self) {
                unsafe {
                    (*<$GPIOX>::ptr()).int_clr.modify(|r, w| {
                        let mut reg = r.bits();
                        reg = pin_set(reg, self.pin as u8);
                        w.bits(reg)
                    });
                }
            }

            fn set_level_edge_interrupt(&self, mode: InterruptMode) {
                unsafe {
                    (*<$GPIOX>::ptr()).int_mod.modify(|r, w| {
                        let mut reg = r.bits();

                        if mode == InterruptMode::Edge {
                            reg = pin_set(reg, self.pin as u8);
                        } else {
                            reg = pin_clear(reg, self.pin as u8);
                        }

                        w.bits(reg)
                    });
                }
            }

            pub fn enable_level_interrupt(&self, level: InterruptLevelPolarity) {
                self.clear_interrupt();
                self.disable_interrupt();
                self.set_level_edge_interrupt(InterruptMode::Level);

                unsafe {
                    (*<$GPIOX>::ptr()).int_pol.modify(|r, w| {
                        let mut reg = r.bits();

                        if level == InterruptLevelPolarity::High {
                            reg = pin_set(reg, self.pin as u8);
                        } else {
                            reg = pin_clear(reg, self.pin as u8);
                        }

                        w.bits(reg)
                    });

                    self.enable_interrupt();
                }
            }

            pub fn enable_edge_interrupt(self, edge: InterruptEdgePolarity) {
                self.clear_interrupt();
                self.disable_interrupt();
                self.set_level_edge_interrupt(InterruptMode::Edge);

                unsafe {
                    (*<$GPIOX>::ptr()).int_pol.modify(|r, w| {
                        let mut reg = r.bits();

                        if edge == InterruptEdgePolarity::Rising {
                            reg = pin_set(reg, self.pin as u8);
                        } else {
                            reg = pin_clear(reg, self.pin as u8);
                        }

                        w.bits(reg)
                    });

                    self.enable_interrupt();
                }
            }
        }
    };
}

gpio!(max32660_pac::GPIO0, Gpio0);
