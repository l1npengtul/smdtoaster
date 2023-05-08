use avr_delay::delay_us;
use ruduino::cores::atmega328::Spi;
use ruduino::modules::HardwareSpi;
use ruduino::cores::current::port::B2;
use ruduino::delay::delay;
use ruduino::Pin;

type CSPin = B2;

pub struct Temperature {}

impl Temperature {
    pub fn setup() {
        Spi::setup_master(4000000);
    }

    pub fn read_temperature() -> u16 {
        B2::set_high();
        delay_us(100);
        B2::set_low();
        let a = Spi::receive_byte();
        let b = Spi::receive_byte();
        let mut c = u16::from_le_bytes([a, b]);
        c &= 0b01111111_11111000;
        c >>= 3;

        c >> 2 // divide by 4
    }
}