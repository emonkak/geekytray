use libdbus_sys as dbus_sys;
use std::borrow::Cow;
use std::error;
use std::ffi::{CStr, CString, NulError};
use std::fmt;
use std::mem;
use std::str::Utf8Error;

use super::message::Message;
use super::types::{Argument, BasicType};
use super::values::{Any, ArgType, BasicValue, DictEntry, ObjectPath, Signature, UnixFd, Variant};

pub struct Reader<'a> {
    iter: dbus_sys::DBusMessageIter,
    message: &'a Message,
}

impl<'a> Reader<'a> {
    pub fn from_message(message: &'a Message) -> Self {
        let iter = unsafe {
            let mut iter = mem::MaybeUninit::uninit();
            dbus_sys::dbus_message_iter_init(message.0, iter.as_mut_ptr());
            iter.assume_init()
        };
        Self { message, iter }
    }

    pub fn has_next(&self) -> bool {
        self.arg_type() != ArgType::Invalid
    }

    pub fn arg_type(&self) -> ArgType {
        ArgType::from(unsafe {
            dbus_sys::dbus_message_iter_get_arg_type(&self.iter as *const _ as *mut _) as u8
        })
    }

    pub fn element_type(&self) -> ArgType {
        ArgType::from(unsafe {
            dbus_sys::dbus_message_iter_get_element_type(&self.iter as *const _ as *mut _) as u8
        })
    }

    pub fn signature(&self) -> Option<Signature> {
        let signature_ptr =
            unsafe { dbus_sys::dbus_message_iter_get_signature(&self.iter as *const _ as *mut _) };
        if signature_ptr.is_null() || unsafe { *signature_ptr } == 0 {
            None
        } else {
            let signature_str = unsafe { CString::from_raw(signature_ptr) };
            Some(Signature::parse(signature_str.to_bytes()).expect("parse signature"))
        }
    }

    pub fn peek<T: Readable>(&self) -> Result<T, Error> {
        if T::compatible(self.arg_type()) {
            T::peek(self)
        } else {
            Err(Error::UnexpectedType {
                expected: T::signature(),
                actual: self.signature(),
            })
        }
    }

    pub fn consume<T: Readable>(&mut self) -> Result<T, Error> {
        let result = self.peek();
        self.next();
        result
    }

    pub fn get_basic<T: BasicType>(&self) -> T {
        assert!(self.arg_type().is_basic());
        let mut value = mem::MaybeUninit::<BasicValue>::uninit();
        let value = unsafe {
            dbus_sys::dbus_message_iter_get_basic(
                &self.iter as *const _ as *mut _,
                value.as_mut_ptr().cast(),
            );
            value.assume_init()
        };
        T::from_basic(value)
    }

    pub fn recurse(&self) -> Reader<'a> {
        assert!(self.arg_type().is_container());
        let subiter = unsafe {
            let mut subiter = mem::MaybeUninit::uninit();
            dbus_sys::dbus_message_iter_recurse(
                &self.iter as *const _ as *mut _,
                subiter.as_mut_ptr(),
            );
            subiter.assume_init()
        };
        Self {
            iter: subiter,
            message: self.message,
        }
    }

    pub fn terminated(&self) -> Result<(), Error> {
        if !self.has_next() {
            Ok(())
        } else {
            Err(Error::TrailingType {
                actual: self.signature(),
            })
        }
    }

    fn next(&mut self) {
        unsafe {
            dbus_sys::dbus_message_iter_next(&mut self.iter);
        }
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Any<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_next() {
            self.consume().ok()
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Utf8Error(Utf8Error),
    NulError(NulError),
    InvalidType,
    UnexpectedEnd {
        expected: Signature,
    },
    TrailingType {
        actual: Option<Signature>,
    },
    UnexpectedType {
        expected: Signature,
        actual: Option<Signature>,
    },
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8Error(error)
    }
}

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Self::NulError(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Utf8Error(error) => error.fmt(f),
            Error::NulError(error) => error.fmt(f),
            Error::InvalidType => f.write_str("Invalid type."),
            Error::TrailingType { actual } => {
                write!(f, "Trailing type `{:?}` found.", actual)
            }
            Error::UnexpectedEnd { expected } => {
                write!(f, "Expected type `{:?}` but nothing else.", expected)
            }
            Error::UnexpectedType { expected, actual } => {
                write!(f, "Expected type `{:?}` but got `{:?}`.", expected, actual,)
            }
        }
    }
}

impl error::Error for Error {}

pub trait Readable: Argument
where
    Self: Sized,
{
    fn peek(reader: &Reader) -> Result<Self, Error>;
}

impl Readable for bool {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for u8 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for i16 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for i32 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for i64 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for u16 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for u32 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for u64 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for f64 {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(reader.get_basic())
    }
}

impl Readable for &str {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(c_str.to_str()?)
    }
}

impl Readable for String {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(c_str.to_string_lossy().into_owned())
    }
}

impl<'a> Readable for Cow<'a, str> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(Cow::Borrowed(c_str.to_str()?))
    }
}

impl Readable for &CStr {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(unsafe { CStr::from_ptr(reader.get_basic()) })
    }
}

impl Readable for CString {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(CString::new(c_str.to_bytes())?)
    }
}

impl<'a> Readable for Cow<'a, CStr> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(Cow::Borrowed(c_str))
    }
}

impl<'a> Readable for ObjectPath<'a> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let c_str = unsafe { CStr::from_ptr(reader.get_basic()) };
        Ok(ObjectPath(c_str.to_string_lossy()))
    }
}

impl Readable for ArgType {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(ArgType::from(reader.get_basic::<u8>()))
    }
}

impl<T: Readable> Readable for Vec<T> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut array_reader = reader.recurse();
        let mut elements = Vec::new();
        while array_reader.has_next() {
            let element = array_reader.consume()?;
            elements.push(element);
        }
        Ok(elements)
    }
}

impl Readable for () {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let sub_reader = reader.recurse();
        sub_reader.terminated()
    }
}

impl<A, B> Readable for (A, B)
where
    A: Readable,
    B: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b))
    }
}

impl<A, B, C> Readable for (A, B, C)
where
    A: Readable,
    B: Readable,
    C: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c))
    }
}

impl<A, B, C, D> Readable for (A, B, C, D)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d))
    }
}

impl<A, B, C, D, E> Readable for (A, B, C, D, E)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
    E: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        let e = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d, e))
    }
}

impl<A, B, C, D, E, F> Readable for (A, B, C, D, E, F)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
    E: Readable,
    F: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        let e = sub_reader.consume()?;
        let f = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d, e, f))
    }
}

impl<A, B, C, D, E, F, G> Readable for (A, B, C, D, E, F, G)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
    E: Readable,
    F: Readable,
    G: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        let e = sub_reader.consume()?;
        let f = sub_reader.consume()?;
        let g = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d, e, f, g))
    }
}

impl<A, B, C, D, E, F, G, H> Readable for (A, B, C, D, E, F, G, H)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
    E: Readable,
    F: Readable,
    G: Readable,
    H: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        let e = sub_reader.consume()?;
        let f = sub_reader.consume()?;
        let g = sub_reader.consume()?;
        let h = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d, e, f, g, h))
    }
}

impl<A, B, C, D, E, F, G, H, I> Readable for (A, B, C, D, E, F, G, H, I)
where
    A: Readable,
    B: Readable,
    C: Readable,
    D: Readable,
    E: Readable,
    F: Readable,
    G: Readable,
    H: Readable,
    I: Readable,
{
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let a = sub_reader.consume()?;
        let b = sub_reader.consume()?;
        let c = sub_reader.consume()?;
        let d = sub_reader.consume()?;
        let e = sub_reader.consume()?;
        let f = sub_reader.consume()?;
        let g = sub_reader.consume()?;
        let h = sub_reader.consume()?;
        let i = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok((a, b, c, d, e, f, g, h, i))
    }
}

impl<T: Readable> Readable for Variant<T> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let sub_reader = reader.recurse();
        Ok(Variant(sub_reader.peek()?))
    }
}

impl<K: Readable, V: Readable> Readable for DictEntry<K, V> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        let mut sub_reader = reader.recurse();
        let key = sub_reader.consume()?;
        let value = sub_reader.consume()?;
        sub_reader.terminated()?;
        Ok(DictEntry(key, value))
    }
}

impl Readable for UnixFd {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        Ok(UnixFd(reader.get_basic()))
    }
}

impl<'a> Readable for Any<'a> {
    fn peek(reader: &Reader) -> Result<Self, Error> {
        match reader.arg_type() {
            ArgType::Boolean => Ok(Self::Boolean(reader.peek()?)),
            ArgType::Byte => Ok(Self::Byte(reader.peek()?)),
            ArgType::Int16 => Ok(Self::Int16(reader.peek()?)),
            ArgType::Int32 => Ok(Self::Int32(reader.peek()?)),
            ArgType::Int64 => Ok(Self::Int64(reader.peek()?)),
            ArgType::Uint16 => Ok(Self::Uint16(reader.peek()?)),
            ArgType::Uint32 => Ok(Self::Uint32(reader.peek()?)),
            ArgType::Uint64 => Ok(Self::Uint64(reader.peek()?)),
            ArgType::Double => Ok(Self::Double(reader.peek()?)),
            ArgType::String => Ok(Self::String(Cow::Borrowed(reader.peek()?))),
            ArgType::ObjectPath => Ok(Self::ObjectPath(reader.peek()?)),
            ArgType::Signature => Ok(Self::Signature(reader.peek()?)),
            ArgType::Array => {
                let mut values = Vec::new();
                let mut sub_reader = reader.recurse();
                let signature = sub_reader.signature().expect("get signature");
                while sub_reader.has_next() {
                    let element = sub_reader.consume::<Any<'a>>()?;
                    assert_eq!(element.signature(), signature);
                    values.push(element);
                }
                sub_reader.terminated()?;
                Ok(Self::Array(values, signature))
            }
            ArgType::Struct => {
                let mut values = Vec::new();
                let mut sub_reader = reader.recurse();
                while sub_reader.has_next() {
                    let element = sub_reader.consume()?;
                    values.push(element);
                }
                sub_reader.terminated()?;
                Ok(Self::Struct(values))
            }
            ArgType::Variant => Ok(Self::Variant(Box::new(reader.recurse().peek()?))),
            ArgType::DictEntry => Ok(Self::DictEntry(Box::new(reader.peek()?))),
            ArgType::UnixFd => Ok(Self::UnixFd(reader.peek()?)),
            ArgType::Invalid => Err(Error::InvalidType),
        }
    }
}
