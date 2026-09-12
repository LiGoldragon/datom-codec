//! Datomic: positional typed data over Protos.
//!
//! The datom dialect carries data, strictly typed. Schema-driven and positional:
//! the reader walks the expected type, writing is the exact reverse projection,
//! and all naming lives in the type. A [`Datom`] is the concept between the
//! protoform and the corporate value; a corporate type bears [`Datomizable`] and
//! [`Composing`] — stating its positions through [`Compositional`] when its form
//! is a struct — and is reached through [`Potential`] on the way in and
//! `textualize` on the way out.
//! Every fault names its layer, its path and its extent in the text.

mod composition;
mod core;
mod decimal;
mod dropping;
mod positional;
mod projection;

pub use composition::{DatomForming, Meaning, Scalar, Variantizing};
pub use core::{
    Actualizing, Budget, Budgeting, Composable, Composing, CompositionDepthing, Compositional,
    Datom, DatomPositioning, Datomizable, Error, ErrorKind, ErrorLayer, ErrorRaising, Form, Naming,
    Pathing, Positioning, Positions, Potential, PotentialExtenting, ProtosExtenting,
};
pub use decimal::{Decimal, DecimalRefusing, Decimating};
pub use derive::{Composing, Datomizable};
pub use protos::Symbol;
pub type Integer = i64;
pub type Path = Vec<Integer>;
pub type Opaque = String;
