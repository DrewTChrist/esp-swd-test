use esp32_hal::{
    gpio::{
        Bank0GpioRegisterAccess, DualCoreInteruptStatusRegisterAccessBank0, Gpio21Signals,
        Gpio22Signals, GpioPin, Input, Output,
    },
    prelude::{
        _embedded_hal_blocking_delay_DelayMs, _embedded_hal_digital_v2_InputPin,
        _embedded_hal_digital_v2_OutputPin,
    },
};

type SwdPin<DIR, SIG, const NUM: u8> = GpioPin<
    DIR,
    Bank0GpioRegisterAccess,
    DualCoreInteruptStatusRegisterAccessBank0,
    esp32_hal::gpio::InputOutputPinType,
    SIG,
    NUM,
>;

type SwdClock = SwdPin<Output<esp32_hal::gpio::PushPull>, Gpio21Signals, 21>;
type SwdDataIn = SwdPin<Input<esp32_hal::gpio::PullUp>, Gpio22Signals, 22>;
type SwdDataOut = SwdPin<Output<esp32_hal::gpio::PushPull>, Gpio22Signals, 22>;

enum DataPinMode {
    Input,
    Output,
}

pub struct Swd {
    clock_pin: Option<SwdClock>,
    data_in: Option<SwdDataIn>,
    data_out: Option<SwdDataOut>,
    delay: Option<esp32_hal::Delay>,
    delay_ms: u32,
}

impl Swd {
    pub fn new(clock_pin: SwdClock, data_pin: SwdDataOut, delay: esp32_hal::Delay) -> Self {
        Self {
            clock_pin: Some(clock_pin),
            data_in: None,
            data_out: Some(data_pin),
            delay: Some(delay),
            delay_ms: 0,
        }
    }

    pub fn release_delay(&mut self) -> esp32_hal::Delay {
        core::mem::replace(&mut self.delay, None).unwrap()
    }

    pub fn release_clock_pin(&mut self) -> SwdClock {
        core::mem::replace(&mut self.clock_pin, None).unwrap()
    }

    pub fn release_data_pin(&mut self) -> SwdDataOut {
        self.set_data_pin_mode(DataPinMode::Output);
        core::mem::replace(&mut self.data_out, None).unwrap()
    }

    fn read_data_pin(&mut self) -> Option<u32> {
        if self.data_in.is_some() {
            let data_pin = self.data_in.as_mut().unwrap();
            if data_pin.is_high().unwrap() {
                return Some(1);
            } else {
                return Some(0);
            }
        }
        None
    }

    fn read_bits(&mut self, num_bits: usize) -> u32 {
        self.set_data_pin_mode(DataPinMode::Input);
        let mut result = 0;
        for _ in 0..num_bits {
            if let Some(level) = self.read_data_pin() {
                result >>= 1;
                result |= level << num_bits;
            }
            self.write_clock();
        }
        result
    }

    fn write_data_pin(&mut self, bit: u32) {
        if self.data_out.is_some() {
            match bit {
                0 => _ = self.data_out.as_mut().unwrap().set_low(),
                1 => _ = self.data_out.as_mut().unwrap().set_high(),
                _ => {}
            }
        }
    }

    fn write_idle(&mut self) {
        self.write_bits(0, 8);
        _ = self.clock_pin.as_mut().unwrap().set_low();
        //self.set_data_pin_mode(DataPinMode::Output);
        _ = self.data_out.as_mut().unwrap().set_low();
    }

    fn write_clock(&mut self) {
        if self.clock_pin.is_some() && self.delay.is_some() {
            _ = self.clock_pin.as_mut().unwrap().set_low();
            self.delay.as_mut().unwrap().delay_ms(10_u32);
            _ = self.clock_pin.as_mut().unwrap().set_high();
            self.delay.as_mut().unwrap().delay_ms(10_u32);
        }
    }

    fn write_bits(&mut self, mut bits: u32, num_bits: usize) {
        self.set_data_pin_mode(DataPinMode::Output);
        for _ in 0..num_bits {
            self.write_data_pin(bits & 1);
            self.write_clock();
            bits >>= 1;
        }
    }

    fn set_data_pin_mode(&mut self, pin_mode: DataPinMode) {
        match pin_mode {
            DataPinMode::Input => if self.data_out.is_some() {
                    let data_pin = core::mem::replace(&mut self.data_out, None).unwrap();
                    self.data_in = Some(data_pin.into_pull_up_input());
            }
            DataPinMode::Output => if self.data_in.is_some() {
                    let data_pin = core::mem::replace(&mut self.data_in, None).unwrap();
                    self.data_out = Some(data_pin.into_push_pull_output());
            }
        }
    }
}
