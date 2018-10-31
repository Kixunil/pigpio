//! Safe FFI bindings to pigpio library
//!
//! **Important**: These are work in progress as I've only implemented what I required for my project.
//! Feel free to help by adding more.

extern crate pigpio_sys;

pub mod error;

/// Existence of this struct proves that `pigpio` library was initialized and gives access to its
/// functions.
///
/// There can only be one instance of this type and the type is not `Sync` because of the inherent
/// thread-unsafety of `pigpio` library. This type is `Send` because to the best of my knowledge
/// it's safe, but I don't provide 100% guarantee. If you need to send it between threads, check it
/// yourself first.
///
/// If it's found that this shouldn't be `Send`, the `Send` impl will be removed.
pub struct PiGpio {
    _private_not_sync: std::marker::PhantomData<*mut ()>,
}

// As far as I understand, this should be safe
unsafe impl Send for PiGpio {}

impl PiGpio {
    /// Tries to initialize `pigpio` library.
    ///
    /// This function will try to initialize the library if it wasn't done already and construct
    /// `PiGpio` to prove the library is initialized. In case of it being already initialized or
    /// failure to initialize it, error is returned. Retries are allowed in case of failure.
    pub fn init() -> Result<Self, error::Init> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        unsafe {
            static REF_COUNT: AtomicUsize = AtomicUsize::new(0);

            if REF_COUNT.compare_and_swap(0, 1, Ordering::SeqCst) == 0 {
                if pigpio_sys::gpioInitialise() != pigpio_sys::PI_INIT_FAILED {
                    Ok(PiGpio { _private_not_sync: Default::default(), })
                } else {
                    REF_COUNT.store(0, Ordering::SeqCst);
                    Err(error::Init::PiGpioFailed)
                }
            } else {
                Err(error::Init::AlreadyInitialized)
            }
        }
    }

    /// Sets mode of the given pin.
    pub fn set_mode(&self, pin: u8, mode: PinMode) -> Result<(), error::SetMode> {
        match unsafe { pigpio_sys::gpioSetMode(pin.into(), mode as u32) } {
            pigpio_sys::PI_BAD_GPIO => Err(error::SetMode::InvalidPin),
            pigpio_sys::PI_BAD_MODE => Err(error::SetMode::InvalidMode),
            _ => Ok(()),
        }
    }

    /// Deletes all waveforms.
    pub fn wave_clear(&self) {
        unsafe {
            pigpio_sys::gpioWaveClear();
        }
    }

    /// Adds a series of pulses to the waveform.
    pub fn wave_add_generic(&self, pulses: &mut [Pulse]) -> Result<(), error::TooManyPulses> {
        unsafe {
            if (pulses.len() as u64) > u64::from(u32::max_value()) {
                Err(error::TooManyPulses)
            } else {
                if pigpio_sys::gpioWaveAddGeneric(pulses.len() as u32, pulses.as_mut_ptr()) != pigpio_sys::PI_TOO_MANY_PULSES {
                    Ok(())
                } else {
                    Err(error::TooManyPulses)
                }
            }
        }
    }

    /// Adds serial data to the waveform.
    ///
    /// In case data_bits is bigger than 8, it must be encoded in two consecutive bytes as native endian.
    /// `offset` signifies the number of microseconds this waveform starts after.
    pub fn wave_add_serial(&self, pin: u8, baud: u32, data_bits: u8, stop_bits:u8, offset: u32, data: &mut [u8]) -> Result<(), error::BadSerial> {
        unsafe {
            if (data.len() as u64) > u64::from(u32::max_value()) {
                Err(error::BadSerial::TooManyBytes)
            } else {
                let result = pigpio_sys::gpioWaveAddSerial(
                    pin.into(),
                    baud.into(),
                    data_bits.into(),
                    stop_bits.into(),
                    offset,
                    data.len() as u32,
                    data.as_mut_ptr()
                );

                error::BadSerial::from_return_code(result)
            }
        }
    }

    /// Creates waveform from added data.
    pub fn wave_create<'a>(&'a self) -> Result<Wave<'a>, error::WaveCreate> {
        let result = unsafe { pigpio_sys::gpioWaveCreate() };
        let id = error::WaveCreate::from_return_code(result)?;
        Ok(Wave { pigpio: Default::default(), id: id as u32, })
    }

    /// Checks to see if waveform has ended
    pub fn wave_tx_busy(&self) -> bool {
        unsafe { pigpio_sys::gpioWaveTxBusy() != 0 }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum PinMode {
    Input = 0,
    Output = 1,
    Alt0 = 4,
    Alt1 = 5,
    Alt2 = 6,
    Alt3 = 7,
    Alt4 = 3,
    Alt5 = 2,
}

/// A wave that can be transmitted.
pub struct Wave<'a> {
    // The referencce isn't actually needed, so we make it zero-sized to optimize it
    pigpio: std::marker::PhantomData<&'a PiGpio>,
    id: u32,
}

impl<'a> Wave<'a> {
    /// Transmit the wave using given mode.
    ///
    /// The wave mode is always SYNC because non-SYNC mode would be unsafe.
    pub fn tx_send(&self, mode: WaveMode) -> Result<(), error::WaveSend> {
        let result = unsafe { pigpio_sys::gpioWaveTxSend(self.id, mode.sync()) };
        error::WaveSend::from_return_code(result)
    }
}

impl<'a> Drop for Wave<'a> {
    fn drop(&mut self) {
        unsafe {
            pigpio_sys::gpioWaveDelete(self.id);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WaveMode {
    OneShot,
    Repeat,
}

impl WaveMode {
    fn sync(self) -> u32 {
        match self {
            WaveMode::OneShot => ::pigpio_sys::PI_WAVE_MODE_ONE_SHOT_SYNC,
            WaveMode::Repeat => ::pigpio_sys::PI_WAVE_MODE_REPEAT_SYNC,
        }
    }
}

type Pulse = pigpio_sys::gpioPulse_t;
