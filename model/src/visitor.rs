use serde::de::{Error as DeError, Visitor};
use std::{
    convert::TryFrom,
    fmt::{Formatter, Result as FmtResult},
    marker::PhantomData,
};

pub struct NumericEnumVisitor<'a, T> {
    description: &'a str,
    phantom: PhantomData<T>,
}

impl<'a, T> NumericEnumVisitor<'a, T> {
    pub fn new(description: &'a str) -> Self {
        Self {
            description,
            phantom: PhantomData,
        }
    }
}

impl<'de, T: From<u8>> Visitor<'de> for NumericEnumVisitor<'_, T> {
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.description)
    }

    fn visit_u8<E: DeError>(self, value: u8) -> Result<Self::Value, E> {
        Ok(T::from(value))
    }

    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        let smaller = u8::try_from(value).map_err(E::custom)?;

        self.visit_u8(smaller)
    }
}
