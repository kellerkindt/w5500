use crate::bus::Bus;
use crate::register::socketn;
use embedded_nal::Ipv4Addr;

pub struct Socket {
    pub index: u8,
    register: u8,
    tx_buffer: u8,
    rx_buffer: u8,
}

impl Socket {
    pub fn new(index: u8) -> Self {
        /*
         * Socket 0 is at address    0x01
         * Socket 0 TX is at address 0x02
         * Socket 0 RX is at address 0x03
         * skip                      0x04
         * Socket 1 is at address    0x05
         * Socket 1 TX is at address 0x06
         * Socket 1 RX is at address 0x07
         * ...
         */
        let block = index * 4;
        Socket {
            index,
            register: block + 1,
            tx_buffer: block + 2,
            rx_buffer: block + 3,
        }
    }

    pub fn register(&self) -> u8 {
        self.register
    }
    pub fn tx_buffer(&self) -> u8 {
        self.tx_buffer
    }
    pub fn rx_buffer(&self) -> u8 {
        self.rx_buffer
    }

    pub fn set_mode<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        mode: socketn::Protocol,
    ) -> Result<(), SpiBus::Error> {
        let mode = [mode as u8];
        bus.write_frame(self.register(), socketn::MODE, &mode)?;
        Ok(())
    }

    pub fn get_status<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<u8, SpiBus::Error> {
        let mut data = [0u8];
        bus.read_frame(self.register(), socketn::STATUS, &mut data)?;
        Ok(data[0])
    }

    pub fn reset_interrupt<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        code: socketn::Interrupt,
    ) -> Result<(), SpiBus::Error> {
        let data = [code as u8];
        bus.write_frame(self.register(), socketn::INTERRUPT, &data)?;
        Ok(())
    }

    pub fn has_interrupt<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        code: socketn::Interrupt,
    ) -> Result<bool, SpiBus::Error> {
        let mut data = [0u8];
        bus.read_frame(self.register(), socketn::INTERRUPT, &mut data)?;
        Ok(data[0] & code as u8 != 0)
    }

    pub fn set_source_port<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        port: u16,
    ) -> Result<(), SpiBus::Error> {
        let data = port.to_be_bytes();
        bus.write_frame(self.register(), socketn::SOURCE_PORT, &data)?;
        Ok(())
    }

    pub fn set_destination_ip<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        ip: Ipv4Addr,
    ) -> Result<(), SpiBus::Error> {
        let data = ip.octets();
        bus.write_frame(self.register(), socketn::DESTINATION_IP, &data)?;
        Ok(())
    }

    pub fn set_destination_port<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        port: u16,
    ) -> Result<(), SpiBus::Error> {
        let data = port.to_be_bytes();
        bus.write_frame(self.register(), socketn::DESTINATION_PORT, &data)?;
        Ok(())
    }

    pub fn get_tx_read_pointer<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<u16, SpiBus::Error> {
        let mut data = [0u8; 2];
        bus.read_frame(self.register(), socketn::TX_DATA_READ_POINTER, &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_tx_read_pointer<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        pointer: u16,
    ) -> Result<(), SpiBus::Error> {
        let data = pointer.to_be_bytes();
        bus.write_frame(self.register(), socketn::TX_DATA_READ_POINTER, &data)?;
        Ok(())
    }

    pub fn get_tx_write_pointer<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
    ) -> Result<u16, SpiBus::Error> {
        let mut data = [0u8; 2];
        bus.read_frame(self.register(), socketn::TX_DATA_WRITE_POINTER, &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_tx_write_pointer<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        pointer: u16,
    ) -> Result<(), SpiBus::Error> {
        let data = pointer.to_be_bytes();
        bus.write_frame(self.register(), socketn::TX_DATA_WRITE_POINTER, &data)?;
        Ok(())
    }

    pub fn get_rx_read_pointer<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<u16, SpiBus::Error> {
        let mut data = [0u8; 2];
        bus.read_frame(self.register(), socketn::RX_DATA_READ_POINTER, &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    pub fn set_rx_read_pointer<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        pointer: u16,
    ) -> Result<(), SpiBus::Error> {
        let data = pointer.to_be_bytes();
        bus.write_frame(self.register(), socketn::RX_DATA_READ_POINTER, &data)?;
        Ok(())
    }

    pub fn set_interrupt_mask<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        mask: u8,
    ) -> Result<(), SpiBus::Error> {
        let data = [mask];
        bus.write_frame(self.register(), socketn::INTERRUPT_MASK, &data)?;
        Ok(())
    }

    pub fn command<SpiBus: Bus>(
        &self,
        bus: &mut SpiBus,
        command: socketn::Command,
    ) -> Result<(), SpiBus::Error> {
        let data = [command as u8];
        bus.write_frame(self.register(), socketn::COMMAND, &data)?;
        Ok(())
    }

    pub fn get_receive_size<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<u16, SpiBus::Error> {
        loop {
            // Section 4.2 of datasheet, Sn_TX_FSR address docs indicate that read must be repeated until two sequential reads are stable
            let mut sample_0 = [0u8; 2];
            bus.read_frame(self.register(), socketn::RECEIVED_SIZE, &mut sample_0)?;
            let mut sample_1 = [0u8; 2];
            bus.read_frame(self.register(), socketn::RECEIVED_SIZE, &mut sample_1)?;
            if sample_0 == sample_1 && sample_0[0] >= 8 {
                break Ok(u16::from_be_bytes(sample_0));
            }
        }
    }

    pub fn get_tx_free_size<SpiBus: Bus>(&self, bus: &mut SpiBus) -> Result<u16, SpiBus::Error> {
        let mut data = [0; 2];
        bus.read_frame(self.register(), socketn::TX_FREE_SIZE, &mut data)?;
        Ok(u16::from_be_bytes(data))
    }
}
