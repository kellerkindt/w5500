use byteorder::{BigEndian, ByteOrder};
use embedded_hal::spi::FullDuplex;

use crate::bus::{ActiveBus, Bus};

const WRITE_MODE_MASK: u8 = 0b11111_1_11;
const READ_MODE_MASK: u8 = 0b_11111_0_11;

const FIXED_DATA_LENGTH_MODE_1: u8 = 0b111111_01;
const FIXED_DATA_LENGTH_MODE_2: u8 = 0b111111_10;
const FIXED_DATA_LENGTH_MODE_4: u8 = 0b111111_11;

pub struct ThreeWire {}

impl ThreeWire {
    pub fn new() -> Self {
        Self {}
    }
}

impl Bus for ThreeWire {}

impl ThreeWire {
    pub fn activate<Spi: FullDuplex<u8>>(self, spi: Spi) -> ActiveThreeWire<Spi> {
        ActiveThreeWire { spi }
    }
}

pub struct ActiveThreeWire<Spi: FullDuplex<u8>> {
    spi: Spi,
}

impl<Spi: FullDuplex<u8>> ActiveBus for ActiveThreeWire<Spi> {
    type Error = Spi::Error;

    /// Transfers a frame with an arbitrary data length in FDM
    ///
    /// This is done by passing chunks of fixed length 4, 2, or 1.  For example if a frame looks like this:
    ///
    /// (address 23) 0xF0 0xAB 0x83 0xB2 0x44 0x2C 0xAA
    ///
    /// This will be sent as separate frames in the chunks
    ///
    /// (address 23) 0xF0 0xAB 0x83 0xB2
    /// (address 27) 44 2C
    /// (address 29) AA
    fn transfer_frame<'a>(
        &mut self,
        block: u8,
        mut address: u16,
        is_write: bool,
        data: &'a mut [u8],
    ) -> Result<&'a mut [u8], nb::Error<Self::Error>> {
        let mut control_phase = block << 3;
        if is_write {
            control_phase &= WRITE_MODE_MASK;
        } else {
            control_phase &= READ_MODE_MASK;
        }

        let mut data_phase = &mut data[..];
        let mut last_length_written: u16;
        while data_phase.len() > 0 {
            if data_phase.len() >= 4 {
                control_phase &= FIXED_DATA_LENGTH_MODE_4;
                last_length_written = 4;
            } else if data_phase.len() >= 2 {
                control_phase &= FIXED_DATA_LENGTH_MODE_2;
                last_length_written = 2;
            } else {
                control_phase &= FIXED_DATA_LENGTH_MODE_1;
                last_length_written = 1;
            }

            let mut address_phase = [0u8; 2];
            BigEndian::write_u16(&mut address_phase, address);
            block!(Self::transfer_bytes(&mut self.spi, &mut address_phase)
                .and_then(|_| Self::transfer_byte(&mut self.spi, &mut control_phase))
                .and_then(|_| Self::transfer_bytes(
                    &mut self.spi,
                    &mut data_phase[..last_length_written as usize]
                )))?;

            address += last_length_written;
            data_phase = &mut data_phase[last_length_written as usize..];
        }
        Ok(data_phase)
    }
}

impl<Spi: FullDuplex<u8>> ActiveThreeWire<Spi> {
    pub fn deactivate(self) -> (ThreeWire, Spi) {
        (ThreeWire::new(), self.spi)
    }
}
