# Upgrades

## 0.3.0

Map containers now use headless guillemets: replace `Map.[ … ]` with
`« … »`. Existing map-entry forms inside the container are unchanged.

`DatomRoot` no longer supplies `root_head()`. Root realization and projection
now use the expected root type: an enum root starts directly with its selected
variant; a non-enum root uses its own selected shape.
