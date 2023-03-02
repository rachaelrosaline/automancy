use std::fmt::{self, Display, Formatter};

use bytemuck::{Pod, PodInOption, Zeroable, ZeroableInOption};
use flexstr::SharedStr;
use rune::Any;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use string_interner::backend::StringBackend;
use string_interner::{StringInterner, Symbol};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdRaw(SharedStr, SharedStr);

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Zeroable,
    Pod,
    Any,
    Serialize,
    Deserialize,
)]
pub struct Id(#[rune(get, copy)] usize);

unsafe impl ZeroableInOption for Id {}
unsafe impl PodInOption for Id {}

impl From<Id> for usize {
    fn from(value: Id) -> Self {
        value.0
    }
}

impl From<usize> for Id {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Symbol for Id {
    fn try_from_usize(index: usize) -> Option<Self> {
        Some(Self(index))
    }

    fn to_usize(self) -> usize {
        self.0
    }
}

pub type Interner = StringInterner<StringBackend<Id>>;

impl IdRaw {
    pub const NONE: IdRaw = id_static("automancy", "none");
}

impl Display for IdRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.0, self.1))
    }
}

impl IdRaw {
    pub fn to_id(&self, interner: &mut Interner) -> Id {
        interner.get_or_intern(self.to_string())
    }
}

pub fn id(a: &str, b: &str) -> IdRaw {
    IdRaw(SharedStr::from_ref(a), SharedStr::from_ref(b))
}

pub const fn id_static(a: &'static str, b: &'static str) -> IdRaw {
    IdRaw(SharedStr::from_static(a), SharedStr::from_static(b))
}

impl Serialize for IdRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
    type Value = IdRaw;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("an Id structured as namespace:name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(IdRaw::parse(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(IdRaw::parse(&v))
    }
}

impl<'de> Deserialize<'de> for IdRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor)
    }
}

impl IdRaw {
    pub fn parse(s: &str) -> IdRaw {
        s.split_once(':')
            .map(|(a, b)| id(a, b))
            .unwrap_or_else(|| id("automancy", s))
    }
}
