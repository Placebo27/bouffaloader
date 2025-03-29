use super::error::Error;
use super::config::*;
use embedded_io::Write;
use heapless::String;
use serde_device_tree::error::{Error as DtbError, ErrorType};
use serde_device_tree::ser::patch::Patch;
use serde_device_tree::ser::to_dtb;
use serde_device_tree::{Dtb, DtbPtr, buildin::Node, buildin::StrSeq, from_raw_mut};

pub fn set_bootargs<W: Write>(d: &mut W, new_bootargs: &String<128>) -> Result<(), Error<()>> {
    match DtbPtr::from_raw(OPAQUE_ADDRESS as *mut u8) {
        Ok(ptr) => {
            let size = ptr.align();
            let dtb = Dtb::from(ptr).share();
            let root: Node = from_raw_mut(&dtb).map_err(|_| Error::InvalidDTB)?;
            let patch = Patch::new("/chosen/bootargs", new_bootargs);
            let patches = [patch];
            let mut temp_buffer =
                unsafe { core::slice::from_raw_parts_mut(FIRMWARE_ADDRESS as *mut u8, size * 2) };
            writeln!(d, "temporary buffer {} ready!", size).ok();

            // 使用临时缓冲区调用 to_dtb
            to_dtb(&root, &patches, &mut temp_buffer, d).map_err(|_| Error::InvalidDTB)?;
            writeln!(d, "to dtb success in temporary buffer!").ok();

            // 将临时缓冲区内容写回到 OPAQUE_ADDRESS 指向的内存区域
            let target =
                unsafe { core::slice::from_raw_parts_mut(OPAQUE_ADDRESS as *mut u8, size) };
            target.copy_from_slice(&temp_buffer[..size]);
            writeln!(d, "buffer copied to OPAQUE_ADDRESS!").ok();

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
