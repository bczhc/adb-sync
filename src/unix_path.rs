use bincode::de::{BorrowDecoder, Decoder};
use bincode::enc::Encoder;
use bincode::error::{DecodeError, EncodeError};
use bincode::{BorrowDecode, Decode, Encode};
use cfg_if::cfg_if;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::os;
/// `bincode` doesn't support (de)serializing non-UTF-8 `PathBuf`s
use std::path::PathBuf;

#[derive(Debug)]
pub struct UnixPath(pub PathBuf);

impl Display for UnixPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl<P: Into<PathBuf>> From<P> for UnixPath {
    fn from(value: P) -> Self {
        Self(value.into())
    }
}

impl UnixPath {
    fn to_bytes(&self) -> &[u8] {
        cfg_if! {
            if #[cfg(unix)] {
                os::unix::ffi::OsStrExt::as_bytes(self.0.as_os_str())
            } else {
                panic!("Not supported");
            }
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        cfg_if! {
            if #[cfg(unix)] {
                Self(PathBuf::from(<OsStr as os::unix::ffi::OsStrExt>::from_bytes(bytes)))
            } else {
                panic!("Not supported");
            }
        }
    }
}

impl Encode for UnixPath {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(self.to_bytes(), encoder)?;
        Ok(())
    }
}

impl Decode for UnixPath {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes = <Vec<u8> as Decode>::decode(decoder)?;
        Ok(UnixPath::from_bytes(&bytes))
    }
}

// TODO: Why it requires `BorrowDecode`?
impl<'de> BorrowDecode<'de> for UnixPath {
    fn borrow_decode<D: BorrowDecoder<'de>>(decoder: &mut D) -> Result<Self, DecodeError> {
        let bytes = <Vec<u8> as Decode>::decode(decoder)?;
        Ok(UnixPath::from_bytes(&bytes))
    }
}
