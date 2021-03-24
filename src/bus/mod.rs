use core::fmt::Debug;

mod fdm;
mod vdm;
mod vdm_ref;

pub use self::fdm::Fdm;
pub use self::fdm::FdmError;
pub use self::vdm::Vdm;
pub use self::vdm::VdmError;
pub use self::vdm_ref::OutputPinRef;
pub use self::vdm_ref::SpiRef;
pub use self::vdm_ref::VdmRef;

pub trait Bus {
    type Error: Debug;

    fn read_frame(&mut self, block: u8, address: u16, data: &mut [u8]) -> Result<(), Self::Error>;

    fn write_frame(&mut self, block: u8, address: u16, data: &[u8]) -> Result<(), Self::Error>;
}
