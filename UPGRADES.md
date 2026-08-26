# Upgrades

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
