use esp32_hal::gpio::{
    Bank0GpioRegisterAccess, DualCoreInteruptStatusRegisterAccessBank0, Gpio21Signals,
    Gpio22Signals, GpioPin, Input, Output,
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

    fn toggle_pin_mode(&mut self) {
        if self.data_in.is_some() {
            let data_pin = core::mem::replace(&mut self.data_in, None).unwrap();
            self.data_out = Some(data_pin.into_push_pull_output());
        } else if self.data_out.is_some() {
            let data_pin = core::mem::replace(&mut self.data_out, None).unwrap();
            self.data_in = Some(data_pin.into_pull_up_input());
        }
    }
}
