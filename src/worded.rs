//! The scalars: each reads from one bare word and writes to one, and every
//! worded type bears Datomic through the one generic interaction.

use protos::{Boolean, Boundary, Decimal, Enclosure, Integer, Separator};

use crate::anatomy::{Datom, Expected, Fault, Found};
use crate::kinds::{Datomic, Worded};
use crate::site::Site;

/// The kind whose capability says whether a word is a run of ASCII digits with no leading zero.
trait Digital {
    fn is_digits(&self) -> bool;
    fn is_canonical_digits(&self) -> bool;
}

impl Digital for str {
    fn is_digits(&self) -> bool {
        for byte in self.bytes() {
            if !byte.is_ascii_digit() {
                return false;
            }
        }
        !self.is_empty()
    }

    fn is_canonical_digits(&self) -> bool {
        self.is_digits() && (self == "0" || !self.starts_with('0'))
    }
}

impl Worded for Integer {
    const EXPECTED: Expected = Expected::Integer;

    fn from_word(word: &str) -> Option<Self> {
        let digits = word.strip_prefix('-').unwrap_or(word);
        if !digits.is_canonical_digits() || word == "-0" {
            return None;
        }
        word.parse().ok()
    }

    fn to_word(&self) -> String {
        self.to_string()
    }
}

impl Worded for Decimal {
    const EXPECTED: Expected = Expected::Decimal;

    fn from_word(word: &str) -> Option<Self> {
        let unsigned = word.strip_prefix('-').unwrap_or(word);
        let (whole, fraction) = unsigned.split_once('.')?;
        if !whole.is_canonical_digits() || !fraction.is_digits() {
            return None;
        }
        let value: f64 = word.parse().ok()?;
        Decimal::try_from(value).ok()
    }

    fn to_word(&self) -> String {
        let written = format!("{}", f64::from(*self));
        match written.split_once('.') {
            Some((whole, fraction)) => {
                let fraction = fraction.trim_end_matches('0');
                if fraction.is_empty() {
                    format!("{whole}.0")
                } else {
                    format!("{whole}.{fraction}")
                }
            }
            None => format!("{written}.0"),
        }
    }
}

impl Worded for Boolean {
    const EXPECTED: Expected = Expected::Boolean;

    fn from_word(word: &str) -> Option<Self> {
        match word {
            "True" => Some(true),
            "False" => Some(false),
            _ => None,
        }
    }

    fn to_word(&self) -> String {
        if *self { "True" } else { "False" }.to_owned()
    }
}

impl Worded for Expected {
    const EXPECTED: Expected = Expected::Word;

    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "Variant" => Self::Variant,
            "Struct" => Self::Struct,
            "Vector" => Self::Vector,
            "Text" => Self::Text,
            "Meaning" => Self::Meaning,
            "Integer" => Self::Integer,
            "Decimal" => Self::Decimal,
            "Boolean" => Self::Boolean,
            "Word" => Self::Word,
            _ => return None,
        })
    }

    fn to_word(&self) -> String {
        match self {
            Self::Variant => "Variant",
            Self::Struct => "Struct",
            Self::Vector => "Vector",
            Self::Text => "Text",
            Self::Meaning => "Meaning",
            Self::Integer => "Integer",
            Self::Decimal => "Decimal",
            Self::Boolean => "Boolean",
            Self::Word => "Word",
        }
        .to_owned()
    }
}

impl Worded for Found {
    const EXPECTED: Expected = Expected::Word;

    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "Variant" => Self::Variant,
            "Struct" => Self::Struct,
            "Vector" => Self::Vector,
            "Text" => Self::Text,
            "Meaning" => Self::Meaning,
            "Word" => Self::Word,
            "Angled" => Self::Angled,
            "Qualified" => Self::Qualified,
            "Chain" => Self::Chain,
            _ => return None,
        })
    }

    fn to_word(&self) -> String {
        match self {
            Self::Variant => "Variant",
            Self::Struct => "Struct",
            Self::Vector => "Vector",
            Self::Text => "Text",
            Self::Meaning => "Meaning",
            Self::Word => "Word",
            Self::Angled => "Angled",
            Self::Qualified => "Qualified",
            Self::Chain => "Chain",
        }
        .to_owned()
    }
}

impl Worded for Separator {
    const EXPECTED: Expected = Expected::Word;

    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "Period" => Self::Period,
            "Exclamation" => Self::Exclamation,
            "Colon" => Self::Colon,
            _ => return None,
        })
    }

    fn to_word(&self) -> String {
        match self {
            Self::Period => "Period",
            Self::Exclamation => "Exclamation",
            Self::Colon => "Colon",
        }
        .to_owned()
    }
}

impl Worded for Enclosure {
    const EXPECTED: Expected = Expected::Word;

    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "Braced" => Self::Braced,
            "Bracketed" => Self::Bracketed,
            "Angled" => Self::Angled,
            _ => return None,
        })
    }

    fn to_word(&self) -> String {
        match self {
            Self::Braced => "Braced",
            Self::Bracketed => "Bracketed",
            Self::Angled => "Angled",
        }
        .to_owned()
    }
}

impl Worded for Boundary {
    const EXPECTED: Expected = Expected::Word;

    fn from_word(word: &str) -> Option<Self> {
        Some(match word {
            "CurlyQuotes" => Self::CurlyQuotes,
            "Parentheses" => Self::Parentheses,
            _ => return None,
        })
    }

    fn to_word(&self) -> String {
        match self {
            Self::CurlyQuotes => "CurlyQuotes",
            Self::Parentheses => "Parentheses",
        }
        .to_owned()
    }
}

impl Datomic for Integer {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Decimal {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Boolean {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Expected {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Found {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Separator {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Enclosure {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}

impl Datomic for Boundary {
    fn incorporate(site: Site<'_>) -> Result<Self, Fault> {
        Self::incorporate_word(site)
    }

    fn conceive(&self) -> Datom {
        self.conceive_word()
    }
}
