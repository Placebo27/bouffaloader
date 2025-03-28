use super::{Device, config::*};
use heapless::String;
use serde_device_tree::ser::patch::Patch;
use serde_device_tree::ser::to_dtb;
use serde_device_tree::{Dtb, DtbPtr, buildin::Node, buildin::StrSeq, from_raw_mut};
use super::error::Error;
use embedded_hal::digital::OutputPin;
use embedded_io::{Read, Write};
use serde_device_tree::error::{Error as DtbError, ErrorType};

pub fn set_bootargs<
    W: Write,
    R: Read,
    L: OutputPin,
    SPI: core::ops::Deref<Target = bouffalo_hal::spi::RegisterBlock>,
    PADS,
    const I: usize,
>(
    d: &mut Device<W, R, L, SPI, PADS, I>,
    new_bootargs: &String<128>,
) -> Result<(), Error<()>> {
    match DtbPtr::from_raw(OPAQUE_ADDRESS as *mut u8) {
        Ok(ptr) => {
            let size = ptr.align();
            let dtb = Dtb::from(ptr).share();
            let root: Node = from_raw_mut(&dtb).map_err(|_| Error::InvalidDTB)?;
            let patch = Patch::new("/chosen/bootargs", new_bootargs);
            let patches = [patch];
            let mut buffer =
                unsafe { core::slice::from_raw_parts_mut(OPAQUE_ADDRESS as *mut u8, size) };
            writeln!(d.tx, "buffer ready!").ok();
            to_dtb(&root, &patches, &mut buffer).map_err(|_| Error::InvalidDTB)?;
            writeln!(d.tx, "to dtb success!").ok();
            Ok(())
        }
        Err(DtbError::Typed {
            error_type: ErrorType::InvalidMagic { wrong_magic },
            ..
        }) => Err(Error::InvalideMagic(wrong_magic)),
        Err(_) => Err(Error::InvalidDTB),
    }
}

pub fn get_bootargs() -> Result<&'static str, ()> {
    match DtbPtr::from_raw(OPAQUE_ADDRESS as *mut u8) {
        Ok(ptr) => {
            let dtb = Dtb::from(ptr).share();
            let root: Node = from_raw_mut(&dtb).map_err(|_| ())?;
            let result = root
                .chosen()
                .ok_or(())?
                .get_prop("bootargs")
                .ok_or(())?
                .deserialize::<StrSeq>()
                .iter()
                .next()
                .ok_or(())?;
            if let Some(pos) = result.find(':') {
                return Ok(result.split_at(pos).0);
            } else {
                return Err(());
            }
        }
        Err(_) => Err(()),
    }
}
