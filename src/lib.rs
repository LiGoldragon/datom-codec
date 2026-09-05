//! Datomic: positional typed data over Protos.
//!
//! The datom dialect carries data, strictly typed. Schema-driven and positional:
//! the reader walks the expected type, writing is the exact reverse projection,
//! and all naming lives in the type. A [`Datom`] is the concept between the
//! protoform and the corporate value; a corporate type bears [`Datomic`] and is
//! reached through [`Potential`] on the way in and `textualize` on the way out.
//! Every fault names its layer, its path and its extent in the text.

mod anatomy;
mod conception;
mod containers;
mod dropping;
mod faults;
mod kinds;
mod protosization;
mod site;
mod worded;

pub use anatomy::{Datom, Expected, Fault, Found, Locus, Meaning, Potential, Problem};
pub use kinds::{Carrying, Counted, Datomic, Headed, Positional, Sited, Worded};
pub use site::{Positions, Site, Variant};

pub use protos::{
    Actualizable, Boolean, Boundary, Conceivable, Decimal, Delineation, Enclosure, Extent,
    Incorporable, Integer, Locating, Opaque, Path, Pathed, Protoform, Protosizable, Refusal,
    Separator, Situated, Situating, Situation, Symbol, Text, Texted, Textualizable,
};
