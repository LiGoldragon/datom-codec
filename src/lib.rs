//! Datomic: positional typed data over Protos.
//!
//! The datom dialect carries data, strictly typed. Schema-driven and positional:
//! the reader walks the expected type, writing is the exact reverse projection,
//! and all naming lives in the type. A [`Datom`] is the concept between the
//! protoform and the corporate value; a corporate type bears [`Datomizable`] and
//! [`Compositional`], and is reached through [`Potential`] on the way in and
//! `textualize` on the way out.
//! Every fault names its layer, its path and its extent in the text.

mod core;

pub use core::{
    Actualizing, Budget, Budgeting, Composable, Compositional, Datom, DatomForming,
    DatomPositioning, Datomizable, Error, ErrorKind, Form, Meaning, Naming, Pathing, Positioning,
    Positions, Potential, ProtosForming, Scalar, Variantizing,
};
pub use derive::{Compositional, Datomizable};
pub use protos::Symbol;
pub type Integer = i64;
pub type Path = Vec<Integer>;
pub type Opaque = String;
