//! The special representation: a type whose datom is another type's datom.
//! A digest of thirty-two bytes is written as its lowercase hex string; a
//! ticket number is written as its decimal digits, each a word. Each goes to
//! text and back unchanged, alone and inside a derived struct, and a
//! representation that names no value of the type is refused at its path.

use datom_codec::{
    Actualizing, Budget, Composing, Datomizable, Error, ErrorKind, ErrorLayer, Path, Potential,
    Represented,
};
use protos::{Protosizable, ReaderBudget, Textualizable};

/// Thirty-two bytes in Rust; in datom, a String of sixty-four lowercase hex
/// digits.
#[derive(Debug, PartialEq, Represented)]
struct Digest([u8; 32]);

const HEX: &[u8; 16] = b"0123456789abcdef";

fn nibble(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    }
}

fn refused(expected: &str, value: &str) -> ErrorKind {
    ErrorKind::Value {
        expected: expected.to_owned(),
        value: value.to_owned(),
    }
}

impl Represented for Digest {
    type Representation = String;
    fn represent(&self) -> String {
        let mut hex = String::with_capacity(64);
        for byte in self.0 {
            hex.push(char::from(HEX[usize::from(byte >> 4)]));
            hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        hex
    }
    fn from_representation(hex: String) -> Result<Self, ErrorKind> {
        if hex.len() != 64 {
            return Err(refused("Digest", &hex));
        }
        let mut bytes = [0; 32];
        for (byte, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks(2)) {
            match (nibble(pair[0]), nibble(pair[1])) {
                (Some(high), Some(low)) => *byte = (high << 4) | low,
                _ => return Err(refused("Digest", &hex)),
            }
        }
        Ok(Self(bytes))
    }
}

/// One decimal digit, written as its word.
#[derive(Debug, Clone, Copy, PartialEq, Composing, Datomizable)]
enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

const DIGITS: [Digit; 10] = [
    Digit::Zero,
    Digit::One,
    Digit::Two,
    Digit::Three,
    Digit::Four,
    Digit::Five,
    Digit::Six,
    Digit::Seven,
    Digit::Eight,
    Digit::Nine,
];

/// A ticket number in Rust; in datom, a Vector of its decimal digits, each a
/// word, with no leading Zero but the number zero itself.
#[derive(Debug, PartialEq, Represented)]
struct Ticket(u32);

impl Represented for Ticket {
    type Representation = Vec<Digit>;
    fn represent(&self) -> Vec<Digit> {
        self.0
            .to_string()
            .bytes()
            .map(|digit| DIGITS[usize::from(digit - b'0')])
            .collect()
    }
    fn from_representation(digits: Vec<Digit>) -> Result<Self, ErrorKind> {
        let spelled: String = digits
            .iter()
            .map(|digit| char::from(b'0' + *digit as u8))
            .collect();
        if spelled.is_empty() || (spelled.len() > 1 && spelled.starts_with('0')) {
            return Err(refused("Ticket", &spelled));
        }
        spelled
            .parse()
            .map(Self)
            .map_err(|_| refused("Ticket", &spelled))
    }
}

/// A derived struct whose positions are both represented.
#[derive(Debug, PartialEq, Composing, Datomizable)]
struct Commit {
    digest: Digest,
    ticket: Ticket,
}

fn budget() -> Budget {
    Budget {
        remaining: 4_096,
        reader: ReaderBudget { remaining: 4_096 },
        depth: 0,
        maximum_depth: 4_096,
    }
}

fn counting_digest() -> Digest {
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = (index as u8) * 7 + 3;
    }
    Digest(bytes)
}

const COUNTING_HEX: &str = "030a11181f262d343b424950575e656c737a81888f969da4abb2b9c0c7ced5dc";

fn text_of<T: Datomizable>(value: &T) -> String {
    value.datomize(Path::new()).protosize().textualize()
}

fn read<T: Composing>(text: &str) -> Result<T, Error> {
    Potential::<T>::from(text).actualize(&mut budget())
}

fn composition(path: Path, kind: ErrorKind) -> Error {
    Error {
        layer: ErrorLayer::Composition,
        path,
        kind,
    }
}

#[test]
fn a_digest_is_written_as_its_hex_string_and_read_back() {
    let digest = counting_digest();
    assert_eq!(text_of(&digest), COUNTING_HEX);
    assert_eq!(read::<Digest>(COUNTING_HEX).unwrap(), digest);
    for bytes in [[0; 32], [0xff; 32]] {
        let text = text_of(&Digest(bytes));
        assert_eq!(read::<Digest>(&text).unwrap(), Digest(bytes), "{text}");
    }
}

#[test]
fn a_represented_datom_is_the_datom_of_its_representation() {
    let at: Path = vec![2, 1];
    let digest = counting_digest();
    assert_eq!(
        digest.datomize(at.clone()),
        digest.represent().datomize(at.clone())
    );
    let ticket = Ticket(742);
    assert_eq!(ticket.datomize(at.clone()), ticket.represent().datomize(at));
}

#[test]
fn a_ticket_is_written_as_its_digit_words_and_read_back() {
    assert_eq!(text_of(&Ticket(742)), "[ Seven Four Two ]");
    assert_eq!(read::<Ticket>("[ Seven Four Two ]").unwrap(), Ticket(742));
    assert_eq!(text_of(&Ticket(0)), "[ Zero ]");
    for number in [0, 9, 10, 742, 1_000_000, u32::MAX] {
        let text = text_of(&Ticket(number));
        assert_eq!(read::<Ticket>(&text).unwrap(), Ticket(number), "{text}");
    }
}

#[test]
fn represented_positions_round_trip_inside_a_derived_struct() {
    let commit = Commit {
        digest: counting_digest(),
        ticket: Ticket(42),
    };
    let text = text_of(&commit);
    assert_eq!(read::<Commit>(&text).unwrap(), commit, "{text}");
    let written = format!("{{ {COUNTING_HEX} [ Four Two ] }}");
    assert_eq!(read::<Commit>(&written).unwrap(), commit);
}

#[test]
fn a_digest_that_names_no_thirty_two_bytes_is_refused_at_its_path() {
    let short = &COUNTING_HEX[..62];
    let long = format!("{COUNTING_HEX}00");
    let odd = &COUNTING_HEX[..63];
    let upper = COUNTING_HEX.to_uppercase();
    let foreign = format!("{}g0", &COUNTING_HEX[..62]);
    for text in [short, long.as_str(), odd, upper.as_str(), foreign.as_str()] {
        assert_eq!(
            read::<Digest>(text).unwrap_err(),
            composition(vec![], refused("Digest", text)),
            "{text}"
        );
    }
}

#[test]
fn a_digest_in_a_form_its_representation_refuses_is_refused_as_that_form() {
    assert_eq!(
        read::<Digest>("[ 00 ]").unwrap_err(),
        composition(
            vec![],
            ErrorKind::Form {
                expected: "String".into(),
                found: "Vector".into(),
            }
        )
    );
}

#[test]
fn a_ticket_that_names_no_number_is_refused_at_its_path() {
    for (text, spelled) in [
        ("[]", ""),
        ("[ Zero Seven ]", "07"),
        (
            "[ Four Two Nine Four Nine Six Seven Two Nine Six ]",
            "4294967296",
        ),
    ] {
        assert_eq!(
            read::<Ticket>(text).unwrap_err(),
            composition(vec![], refused("Ticket", spelled)),
            "{text}"
        );
    }
}

#[test]
fn a_ticket_word_that_is_no_digit_is_refused_by_the_representation() {
    assert_eq!(
        read::<Ticket>("[ Seven Eleven ]").unwrap_err(),
        composition(
            vec![1],
            ErrorKind::Variant {
                expected: "Digit".into(),
                found: "Eleven".into(),
            }
        )
    );
    assert_eq!(
        read::<Ticket>("Seven").unwrap_err(),
        composition(
            vec![],
            ErrorKind::Form {
                expected: "Vector".into(),
                found: "Bare".into(),
            }
        )
    );
}

#[test]
fn a_refusal_inside_a_struct_names_the_position_it_arose_at() {
    let bad_digest = format!("{{ {} [ Four Two ] }}", &COUNTING_HEX[..62]);
    assert_eq!(
        read::<Commit>(&bad_digest).unwrap_err(),
        composition(vec![0], refused("Digest", &COUNTING_HEX[..62]))
    );
    let bad_ticket = format!("{{ {COUNTING_HEX} [ Zero Four Two ] }}");
    assert_eq!(
        read::<Commit>(&bad_ticket).unwrap_err(),
        composition(vec![1], refused("Ticket", "042"))
    );
}
