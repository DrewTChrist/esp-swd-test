use esp32_hal::{
    gpio::{
        Bank0GpioRegisterAccess, DualCoreInteruptStatusRegisterAccessBank0, Gpio21Signals,
        Gpio22Signals, GpioPin, Input, Output,
    },
    prelude::_embedded_hal_digital_v2_OutputPin,
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
}

impl Swd {
    pub fn new(
        clock_pin: Option<SwdClock>,
        data_in: Option<SwdDataIn>,
        data_out: Option<SwdDataOut>,
    ) -> Self {
        Self {
            clock_pin,
            data_in,
            data_out,
        }
    }

    fn write_data_pin(&mut self, bit: u32) {
        if self.data_out.is_some() {
            match bit {
                0 => {
                    self.data_out.as_mut().unwrap().set_low();
                },
                1 => {
                    self.data_out.as_mut().unwrap().set_high();
                },
                _ => {}
            }
        }
    }

    fn write_clock(&mut self) {
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
            DataPinMode::Input => {
                if self.data_out.is_some() {
                    let data_pin = core::mem::replace(&mut self.data_out, None).unwrap();
                    self.data_in = Some(data_pin.into_pull_up_input());
                }
            }
            DataPinMode::Output => {
                if self.data_in.is_some() {
                    let data_pin = core::mem::replace(&mut self.data_in, None).unwrap();
                    self.data_out = Some(data_pin.into_push_pull_output());
                }
            }
        }
    }
}
