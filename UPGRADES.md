# Upgrades

## 0.7.0

This is a breaking public type change. Update to Protos 0.14.0 and replace
raw `f64` and `String` Datomic anatomies with `FiniteDecimal` and
`DatomicString`. The new types retain a Protos-validated canonical Portion;
non-finite decimals and unrepresentable no-escape curly content cannot be
projected. Duplicate map keys now fault at the duplicate key extent rather
than overwriting an earlier value.

## 0.6.1

Update to Protos 0.12.0. Expected Strings now preserve any one canonical
Protos Portion as bare text, including `a.b`, `a!b`, and `a:b`; text with
spaces remains curly-opaque. This completes the intended 0.6 typed String
contract without a compatibility parser.

## 0.6.0

This is a breaking deployment. Rename the dependency and Rust import from
`datom` to `datomic`, update the repository pin to `LiGoldragon/datomic`, and
replace the former scoped-walk traits and typed-text carrier with the one
`Datomic` anatomy and `Text<T>` public edge.

No legacy syntax or API shim exists. Re-author each consumer's anatomy using
Protos `Portion`, then convert text only through `Text::<T>::from(input)` and
the Datomic edge. Convert booleans to `True`/`False`, options to
`None`/`Some.value`, maps to headless alternating guillemet portions, and
finite floats to decimal forms with a decimal point and no exponent.

## 0.5.0

Datom now depends on Protos 0.8.0, whose public `Shape` has a new
`DottedBare` case for headed units such as `Observe.Locks`; update exhaustive
`Shape` matches in consumers.

`i64` now implements the Datom scalar seams. Its accepted and canonical form
is bare ASCII decimal: `0`, positive digits without a leading zero, or `-`
followed by positive digits; values outside the `i64` range, `+`, `-0`, and
leading zeroes are faults.

`DatomHeadedUnit` is the public composition seam for a payloadless enum that
projects as `Head.Unit`. Implement it together with `DatomRoot` for root
families such as `Observe.Locks`; it replaces no existing schema form.

## 0.4.0

Plain Strings no longer accept or project parenthesis-delimited text. Replace
every delimited plain String with curly quotes (`“ … ”`); bare text remains a
String only where the expected type is String. Parentheses are reserved for
the future structured String, Meaning, and are not a Datom plain-String form.

## 0.3.0

Map containers now use headless guillemets: replace `Map.[ … ]` with
`« … »`. Existing map-entry forms inside the container are unchanged.

`DatomRoot` no longer supplies `root_head()`. Root realization and projection
now use the expected root type: an enum root starts directly with its selected
variant; a non-enum root uses its own selected shape.
