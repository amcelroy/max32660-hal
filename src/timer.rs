use max32660_pac;

#[derive(Debug, Copy, Clone)]
pub enum TimerError {}

pub enum TimerMode {
    OneShot = 0b000,
    Continuous = 0b001,
}

macro_rules! timer {
    ($tim:ty, $name:ident) => {
        pub struct $name {
            peripheral_clk_hz: f32,
            timer: $tim,
        }

        impl $name {
            /// Disables the timer
            pub fn disable(&mut self) {
                self.timer.cn.modify(|_, w| w.ten().clear_bit());
            }

            /// Creates a new timer with a default period of 1 second.
            pub fn new(timer: $tim, peripheral_clk_hz: f32) -> Self {
                Self {
                    peripheral_clk_hz,
                    timer: timer,
                }
            }

            /// Disables the timer and sets a new count based on timer frequency
            pub fn set_freq(&mut self, hz: f32) -> &mut Self {
                self.disable();
                let count = (self.peripheral_clk_hz / hz) as u32;
                unsafe {
                    self.timer.cmp.write(|w| w.bits(count));
                }
                self
            }

            /// Disables the timer and sets a new mode
            pub fn set_mode(&mut self, mode: TimerMode) -> &mut Self {
                self.disable();
                self.timer.cn.modify(|_, w| w.tmode().bits(mode as u8));
                self
            }

            pub fn start_periodic(&mut self, hz: f32) {
                self.set_freq(hz);
                self.set_mode(TimerMode::Continuous);
                self.timer.cn.modify(|_, w| w.ten().set_bit());
            }

            pub fn start_one_shot(&mut self, hz: f32) {
                self.set_freq(hz);
                self.set_mode(TimerMode::OneShot);
                self.timer.cn.modify(|_, w| w.ten().set_bit());
            }
        }
    };
}

timer!(max32660_pac::TMR0, Timer0);
timer!(max32660_pac::TMR1, Timer1);
timer!(max32660_pac::TMR2, Timer2);
