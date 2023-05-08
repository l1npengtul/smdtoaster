use lcd::Hardware;
use ruduino::cores::current::port::{B0, D2, D3, D5, D6, D7};
use ruduino::Pin;

type ResetPin = D3;
type EnablePin = D2;

type Data4 = B0;
type Data5 = D5;
type Data6 = D6;
type Data7 = D7;

pub struct LCDHardware {}

impl Hardware for LCDHardware {


    fn rs(&mut self, bit: bool) {
        if bit {
            ResetPin::set_high();
        }
        ResetPin::set_low()
    }

    fn enable(&mut self, bit: bool) {
        if bit {
            EnablePin::set_high()
        }
        EnablePin::set_low()
    }

    fn data(&mut self, data: u8) {
        let mut data = data;
        for p in 0..4 {
            if data & 0x01 == 0x01 {
                match p {
                    0 => Data4::set_high(),
                    1 => Data5::set_high(),
                    2 => Data6::set_high(),
                    3 => Data7::set_high(),
                    _ => unreachable!(),
                }
            } else {
                match p {
                    0 => Data4::set_low(),
                    1 => Data5::set_low(),
                    2 => Data6::set_low(),
                    3 => Data7::set_low(),
                    _ => unreachable!(),
                }
            }
            data >>=1;
        }
    }
}
