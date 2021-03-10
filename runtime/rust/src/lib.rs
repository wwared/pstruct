#[derive(Debug)]
pub enum PError {
    BufTooSmall,
    NotEnoughData,
}

pub type Result<T> = std::result::Result<T, PError>;

pub trait Pstruct: Sized {
    // these all get generated with the macro alongside the struct def
    fn encode(&self) -> Result<Vec<u8>>;
    fn encode_buf(&self, buf: &mut [u8]) -> Result<()>;

    fn decode_new(data: &[u8]) -> Result<Self>;
    fn decode(&mut self, data: &[u8]) -> Result<()>;

    fn size(&self) -> usize;
}

pub trait Primitive: Sized {
    fn encode_le(&self, buf: &mut [u8]) -> Result<()>;
    fn encode_be(&self, buf: &mut [u8]) -> Result<()>;
    fn decode_le(&mut self, data: &[u8]) -> Result<()>;
    fn decode_be(&mut self, data: &[u8]) -> Result<()>;
}

// here we never encode/decode the length
impl<T: Primitive + Default + Copy, const N: usize> Primitive for [T; N] {
    fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn decode_le(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_le(&data[idx * size..])?;
        }
        Ok(())
    }
    fn decode_be(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_be(&data[idx * size..])?;
        }
        Ok(())
    }
}

impl<T: Primitive + Default + Copy> Primitive for Vec<T> {
    // NOTE: for this to work, the vec needs to already be correctly-sized before
    // calling these functions!
    // we can guarantee that in our generated code, but this makes this weird to use
    fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn decode_le(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_le(&data[idx * size..])?;
        }
        Ok(())
    }
    fn decode_be(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_be(&data[idx * size..])?;
        }
        Ok(())
    }
}

impl<T: Primitive + Default + Copy> Primitive for &mut [T] {
    // NOTE: for this to work, the vec needs to already be correctly-sized before
    // calling these functions!
    // we can guarantee that in our generated code, but this makes this weird to use
    fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, elem) in self.iter().enumerate() {
            elem.encode_le(&mut buf[idx * size..])?;
        }
        Ok(())
    }
    fn decode_le(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_le(&data[idx * size..])?;
        }
        Ok(())
    }
    fn decode_be(&mut self, data: &[u8]) -> Result<()> {
        let size = std::mem::size_of::<T>();
        for (idx, item) in self.iter_mut().enumerate() {
            item.decode_be(&data[idx * size..])?;
        }
        Ok(())
    }
}

// pub type BoundString = (String, usize);
// impl Primitive for BoundString {
//     fn size(&self) -> usize {
//         self.1
//     }

//     fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
//         todo!()
//     }

//     fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
//         todo!()
//     }

//     fn decode_le(&mut self, data: &[u8]) -> Result<()> {
//         todo!()
//     }

//     fn decode_be(&mut self, data: &[u8]) -> Result<()> {
//         todo!()
//     }
// }

// use std::ffi::CString;
// pub type BoundCString = (std::ffi::CString, usize); // a cstring and a maximum size
// impl Primitive for BoundCString {
//     fn size(&self) -> usize {
//         self.1
//     }

//     fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
//         todo!()
//     }

//     fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
//         todo!()
//     }

//     fn decode_le(&mut self, data: &[u8]) -> Result<()> {
//         todo!()
//     }

//     fn decode_be(&mut self, data: &[u8]) -> Result<()> {
//         todo!()
//     }
// }

macro_rules! basic_primitive {
    ($ty:ty) => {
        impl Primitive for $ty {
            // maybe we want to pass exactly-sized slices to these functions
            // and use == instead of < in the ifs
            fn encode_le(&self, buf: &mut [u8]) -> Result<()> {
                let size = std::mem::size_of::<$ty>();
                if buf.len() < size {
                    return Err(PError::BufTooSmall);
                }
                buf[0..size].copy_from_slice(&self.to_le_bytes());
                Ok(())
            }
            fn encode_be(&self, buf: &mut [u8]) -> Result<()> {
                let size = std::mem::size_of::<$ty>();
                if buf.len() < size {
                    return Err(PError::BufTooSmall);
                }
                buf[0..size].copy_from_slice(&self.to_be_bytes());
                Ok(())
            }
            fn decode_le(&mut self, data: &[u8]) -> Result<()> {
                use std::convert::TryInto;
                let size = std::mem::size_of::<$ty>();
                if data.len() < size {
                    return Err(PError::NotEnoughData);
                }
                *self = <$ty>::from_le_bytes(data[0..size].try_into().unwrap());
                Ok(())
            }
            fn decode_be(&mut self, data: &[u8]) -> Result<()> {
                use std::convert::TryInto;
                let size = std::mem::size_of::<$ty>();
                if data.len() < size {
                    return Err(PError::NotEnoughData);
                }
                *self = <$ty>::from_be_bytes(data[0..size].try_into().unwrap());
                Ok(())
            }
        }
    };
}

basic_primitive!(u8);
basic_primitive!(u16);
basic_primitive!(u32);
basic_primitive!(u64);
basic_primitive!(i8);
basic_primitive!(i16);
basic_primitive!(i32);
basic_primitive!(i64);
basic_primitive!(f32);
basic_primitive!(f64);

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn simple() {
        let x: u64 = 42069;
        let mut buf: [u8; 8] = [0; 8];
        x.encode_le(&mut buf).unwrap();
        let mut y: u64 = 0;
        y.decode_le(&buf).unwrap();
        assert_eq!(y, x, "u64 decode_le");
    }
}
