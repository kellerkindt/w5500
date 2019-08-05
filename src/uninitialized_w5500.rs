use embedded_hal::spi::FullDuplex;
use embedded_hal::digital::v2::OutputPin;
use bus::{Bus, FourWire, ThreeWire};
use w5500::W5500;

pub struct UninitializedW5500<Spi: FullDuplex<u8>, SpiBus: Bus<Spi>> {
    bus: SpiBus,
    spi: Spi,
}

impl<Spi: FullDuplex<u8>, SpiBus: Bus<Spi>> UninitializedW5500<Spi, SpiBus> {
    pub fn initialize() -> W5500 {
        // TODO actually initialize chip
        W5500 {}
    }
}

impl<Spi: FullDuplex<u8>, ChipSelect: OutputPin> UninitializedW5500<Spi, FourWire<ChipSelect>> {
    pub fn new(spi: Spi, cs: ChipSelect) -> Self {
        Self { spi, bus: FourWire::new(cs) }
    }
    pub fn deactivate(self) -> (Spi, ChipSelect) {
        (self.spi, self.bus.release())
    }
}

impl<Spi: FullDuplex<u8>> UninitializedW5500<Spi, ThreeWire> {
    pub fn new(spi: Spi) -> Self {
        Self { spi, bus: ThreeWire::new() }
    }
    pub fn deactivate(self) -> Spi {
        self.spi
    }
}