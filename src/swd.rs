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

/// Esp32 gpio type with generic direction (DIR),
/// signal (SIG) and pin number (NUM)
type SwdPin<DIR, SIG, const NUM: u8> = GpioPin<
    DIR,
    Bank0GpioRegisterAccess,
    DualCoreInteruptStatusRegisterAccessBank0,
    esp32_hal::gpio::InputOutputPinType,
    SIG,
    NUM,
>;

/// Concrete clock pin type on pin 21
type SwdClock = SwdPin<Output<esp32_hal::gpio::PushPull>, Gpio21Signals, 21>;
/// Concrete data pin input type on pin 22
type SwdDataIn = SwdPin<Input<esp32_hal::gpio::PullUp>, Gpio22Signals, 22>;
/// Concrete data pin output type on pin 22
type SwdDataOut = SwdPin<Output<esp32_hal::gpio::PushPull>, Gpio22Signals, 22>;

/// Debug port address for core 0
const DP0: u32 = 0x01002927;
/// Debug port address for core 1
const DP1: u32 = 0x11002927;
/// Rescue debug port address
const DPR: u32 = 0xf1002927;

/// Direction of the swd data pin
enum DataPinMode {
    /// Pull up input
    Input,
    /// Push pull output
    Output,
}

/// Struct for holding esp32 peripherals 
/// needed to bit bang the SWD protocol
///
/// There were two suitable methods for
/// managing the direction change of the
/// data pin. An enum with a variant for
/// each direction that contains the pin
/// type would have worked. In this case,
/// I chose to use two fields each with
/// their own input/output type and simply
/// move the pin between those two fields.
pub struct Swd {
    /// Clock cycle pin
    clock_pin: Option<SwdClock>,
    /// Data pin input
    data_in: Option<SwdDataIn>,
    /// Data pin output
    data_out: Option<SwdDataOut>,
    /// Delay peripheral for clock
    delay: Option<esp32_hal::Delay>,
    /// Delay time
    delay_ms: u32,
}

impl Swd {
    /// Creates a new swd struct
    pub fn new(clock_pin: SwdClock, data_pin: SwdDataOut, delay: esp32_hal::Delay) -> Self {
        Self {
            clock_pin: Some(clock_pin),
            data_in: None,
            data_out: Some(data_pin),
            delay: Some(delay),
            delay_ms: 0,
        }
    }

    /// Release the delay peripheral
    pub fn release_delay(&mut self) -> esp32_hal::Delay {
        core::mem::replace(&mut self.delay, None).unwrap()
    }

    /// Release the clock pin
    pub fn release_clock_pin(&mut self) -> SwdClock {
        core::mem::replace(&mut self.clock_pin, None).unwrap()
    }

    /// Release the data pin
    pub fn release_data_pin(&mut self) -> SwdDataOut {
        self.set_data_pin_mode(DataPinMode::Output);
        core::mem::replace(&mut self.data_out, None).unwrap()
    }

    /// Reads the level of the data pin and
    /// returns either an Option with the level
    /// or None
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

    /// Read bits on the data pin
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

    /// Sets the data pin either high or low
    fn write_data_pin(&mut self, bit: u32) {
        if self.data_out.is_some() {
            match bit {
                0 => _ = self.data_out.as_mut().unwrap().set_low(),
                1 => _ = self.data_out.as_mut().unwrap().set_high(),
                _ => {}
            }
        }
    }

    /// Write an idle state over the data line
    fn write_idle(&mut self) {
        self.write_bits(0, 8);
        _ = self.clock_pin.as_mut().unwrap().set_low();
        //self.set_data_pin_mode(DataPinMode::Output);
        _ = self.data_out.as_mut().unwrap().set_low();
    }

    /// Write the clock low and high with a small
    /// delay in between
    fn write_clock(&mut self) {
        if self.clock_pin.is_some() && self.delay.is_some() {
            _ = self.clock_pin.as_mut().unwrap().set_low();
            self.delay.as_mut().unwrap().delay_ms(10_u32);
            _ = self.clock_pin.as_mut().unwrap().set_high();
            self.delay.as_mut().unwrap().delay_ms(10_u32);
        }
    }

    /// Writes some bits to the data pin
    fn write_bits(&mut self, mut bits: u32, num_bits: usize) {
        self.set_data_pin_mode(DataPinMode::Output);
        for _ in 0..num_bits {
            self.write_data_pin(bits & 1);
            self.write_clock();
            bits >>= 1;
        }
    }

    /// Changes the direction of the data pin to
    /// either input or output
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
