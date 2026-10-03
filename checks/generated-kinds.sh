# The committed Rust of a kinds declaration is what the pinned ethos-zero
# generates from it, byte for byte.
set -eu

temporary="$(mktemp -d)"
cleanup() {
  rm -rf "$temporary"
}
trap cleanup EXIT

name="$(basename "$declaration" .ethos)"
reply="$("$generator/bin/ethos-zero" "Generate.{ «$declaration» «$temporary» }")"
case "$reply" in
  Generated.*) ;;
  *) echo "$declaration: $reply" >&2; exit 1 ;;
esac
cmp "$temporary/$name.rs" "$committed"
touch "$out"
