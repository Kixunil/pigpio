#[derive(Copy, Clone, Debug)]
pub enum Init {
    PiGpioFailed,
    AlreadyInitialized,
}

#[derive(Copy, Clone, Debug)]
pub struct TooManyPulses;

#[derive(Copy, Clone, Debug)]
pub enum BadSerial {
    InvalidPin,
    InvalidBaud,
    InvalidDataBits,
    InvalidStopBits,
    TooManyBytes,
    InvalidOffset,
    TooManyPulses,
}

impl BadSerial {
    pub(crate) fn from_return_code(code: i32) -> Result<(), Self> {
        use self::BadSerial::*;

        match code {
            ::pigpio_sys::PI_BAD_USER_GPIO => Err(InvalidPin),
            ::pigpio_sys::PI_BAD_WAVE_BAUD => Err(InvalidBaud),
            ::pigpio_sys::PI_BAD_DATABITS => Err(InvalidDataBits),
            ::pigpio_sys::PI_BAD_STOPBITS => Err(InvalidStopBits),
            ::pigpio_sys::PI_TOO_MANY_CHARS => Err(TooManyBytes),
            ::pigpio_sys::PI_BAD_SER_OFFSET => Err(InvalidOffset),
            ::pigpio_sys::PI_TOO_MANY_PULSES => Err(TooManyPulses),
            _ => Ok(()),
        }
    }
}

impl From<TooManyPulses> for BadSerial {
    fn from(_: TooManyPulses) -> Self {
        BadSerial::TooManyPulses
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WaveCreate {
    EmptyWaveform,
    TooManyCbs,
    TooManyOol,
    NoWaveformId,
}

impl WaveCreate {
    pub(crate) fn from_return_code(code: i32) -> Result<i32, Self> {
        use self::WaveCreate::*;

        match code {
            ::pigpio_sys::PI_EMPTY_WAVEFORM => Err(EmptyWaveform),
            ::pigpio_sys::PI_TOO_MANY_CBS => Err(TooManyCbs),
            ::pigpio_sys::PI_TOO_MANY_OOL => Err(TooManyOol),
            ::pigpio_sys::PI_NO_WAVEFORM_ID => Err(NoWaveformId),
            x => Ok(x),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WaveSend {
    InvalidId,
    InvalidMode,
}

impl WaveSend {
    pub(crate) fn from_return_code(code: i32) -> Result<(), Self> {
        match code {
            ::pigpio_sys::PI_BAD_WAVE_ID => Err(WaveSend::InvalidId),
            ::pigpio_sys::PI_BAD_WAVE_MODE => Err(WaveSend::InvalidMode),
            _ => Ok(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SetMode {
    InvalidPin,
    InvalidMode,
}
