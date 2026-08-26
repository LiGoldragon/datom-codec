# Upgrades

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
