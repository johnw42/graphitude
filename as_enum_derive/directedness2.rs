use std::{fmt::Debug, hash::Hash};

use crate::util::sort_pair_if;

// =============================================================================
// Library
// =============================================================================

pub trait AsEnum<T>: Clone {
    fn as_enum(&self) -> T;
}

// =============================================================================
// Macro Input
// =============================================================================

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, AsEnum)]
pub enum Directedness {
    Directed,
    Undirected,
}

// =============================================================================
// Generated Code
// =============================================================================

impl AsEnum<Directedness> for Directedness {
    fn as_enum(&self) -> Directedness {
        *self
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directed;

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

impl AsEnum<Directedness> for Directed {
    fn as_enum(&self) -> Directedness {
        Directedness::Directed
    }
}

impl AsEnum<Directedness> for Undirected {
    fn as_enum(&self) -> Directedness {
        Directedness::Undirected
    }
}

impl From<Directed> for Directedness {
    fn from(value: Directed) -> Self {
        value.as_enum()
    }
}

impl From<Undirected> for Directedness {
    fn from(value: Undirected) -> Self {
        value.as_enum()
    }
}

impl TryFrom<Directedness> for Directed {
    type Error = ();

    fn try_from(value: Directedness) -> Result<Self, Self::Error> {
        match value {
            Directedness::Directed => Ok(Directed),
            _ => Err(()),
        }
    }
}

impl TryFrom<Directedness> for Undirected {
    type Error = ();

    fn try_from(value: Directedness) -> Result<Self, Self::Error> {
        match value {
            Directedness::Undirected => Ok(Undirected),
            _ => Err(()),
        }
    }
}
