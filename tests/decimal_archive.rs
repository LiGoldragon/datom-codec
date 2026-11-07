//! Under the `rkyv` feature, an archived `Decimal` is read back through `Decimal::try_from`: a
//! non-finite float in the bytes is refused, directly and nested in derived containers, a negative
//! zero reads as zero, and the archived layout of every valid value is unchanged. The raw mirrors
//! below archive a bare `f64` in the position of the decimal, which is the layout of a `Decimal`.

use datom_codec::{Decimal, Decimating};
use rkyv::{Archive, Deserialize, Serialize, rancor::Error};

#[derive(Archive, Serialize)]
struct RawDecimal(f64);

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
struct Holder {
    one: Decimal,
    many: Vec<Decimal>,
    maybe: Option<Decimal>,
}

#[derive(Archive, Serialize)]
struct RawHolder {
    one: RawDecimal,
    many: Vec<RawDecimal>,
    maybe: Option<RawDecimal>,
}

fn raw_bytes(value: f64) -> Vec<u8> {
    rkyv::to_bytes::<Error>(&RawDecimal(value))
        .expect("archives")
        .to_vec()
}

fn decimal(value: f64) -> Decimal {
    Decimal::try_from(value).expect("a finite decimal")
}

const NON_FINITE: [f64; 3] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
const FINITE: [f64; 9] = [
    0.0,
    1.0,
    -1.5,
    3.2,
    f64::MAX,
    f64::MIN,
    f64::MIN_POSITIVE,
    5e-324,
    1.0e300,
];

#[test]
fn a_valid_decimal_archives_to_the_layout_it_always_had() {
    for value in FINITE {
        let bytes = rkyv::to_bytes::<Error>(&decimal(value)).expect("archives");
        assert_eq!(bytes.as_slice(), raw_bytes(value).as_slice(), "{value}");
    }
}

#[test]
fn every_finite_boundary_reads_back_equal() {
    for value in FINITE {
        let bytes = rkyv::to_bytes::<Error>(&decimal(value)).expect("archives");
        let back = rkyv::from_bytes::<Decimal, Error>(&bytes).expect("a finite decimal restores");
        assert_eq!(back, decimal(value), "{value}");
        assert_eq!(back.float().to_bits(), value.to_bits(), "{value}");
    }
}

#[test]
fn a_non_finite_float_in_the_bytes_is_refused() {
    for value in NON_FINITE {
        // The error text is available only with rancor's debug assertions or alloc feature,
        // so the refusal itself is what is asserted.
        assert!(
            rkyv::from_bytes::<Decimal, Error>(&raw_bytes(value)).is_err(),
            "{value}"
        );
    }
}

#[test]
fn a_negative_zero_in_the_bytes_reads_as_zero() {
    let back = rkyv::from_bytes::<Decimal, Error>(&raw_bytes(-0.0)).expect("zero restores");
    assert_eq!(back, decimal(0.0));
    assert_eq!(back.float().to_bits(), 0.0f64.to_bits());
}

fn raw_holder(one: f64, many: &[f64], maybe: Option<f64>) -> Vec<u8> {
    rkyv::to_bytes::<Error>(&RawHolder {
        one: RawDecimal(one),
        many: many.iter().copied().map(RawDecimal).collect(),
        maybe: maybe.map(RawDecimal),
    })
    .expect("archives")
    .to_vec()
}

#[test]
fn a_non_finite_float_nested_in_a_container_is_refused_in_every_position() {
    for bad in NON_FINITE {
        for bytes in [
            raw_holder(bad, &[1.0], Some(2.0)),
            raw_holder(1.0, &[1.0, bad, 3.0], Some(2.0)),
            raw_holder(1.0, &[1.0], Some(bad)),
        ] {
            assert!(
                rkyv::from_bytes::<Holder, Error>(&bytes).is_err(),
                "{bad} nested"
            );
        }
    }
}

#[test]
fn a_finite_container_restores_and_keeps_its_layout() {
    let holder = Holder {
        one: decimal(1.0),
        many: vec![decimal(0.0), decimal(f64::MAX)],
        maybe: Some(decimal(5e-324)),
    };
    let bytes = rkyv::to_bytes::<Error>(&holder).expect("archives");
    let raw = raw_holder(1.0, &[0.0, f64::MAX], Some(5e-324));
    assert_eq!(bytes.as_slice(), raw.as_slice());
    let restored = rkyv::from_bytes::<Holder, Error>(&bytes).expect("a finite container restores");
    assert_eq!(restored, holder);
    let negative_zero = raw_holder(1.0, &[-0.0], None);
    let restored = rkyv::from_bytes::<Holder, Error>(&negative_zero).expect("zero restores");
    assert_eq!(restored.many, vec![decimal(0.0)]);
}

#[test]
fn bytes_that_are_no_archive_are_still_refused() {
    assert!(rkyv::from_bytes::<Decimal, Error>(&[1, 2, 3]).is_err());
}
