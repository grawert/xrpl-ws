BEGIN {
  marker     = "(Any|Copy|Default|Freeze|Same|StructuralEq|StructuralPartialEq|Unpin|UnsafeUnpin|VZip|CastableFrom)"
  eq_partial = "(Eq|PartialEq)"
  send_sync  = "(Send|Sync|RefUnwindSafe|UnwindSafe)"
  borrow     = "(AsMut|AsRef|Borrow|BorrowMut|Clone|CloneToUninit|ToOwned)"
  convert    = "(From|FromIterator|Into|IntoIterator|TryFrom|TryInto|ToString)"
  compare    = "(Ord|PartialOrd)"
  io_common  = "(Read|Write|Seek|BufRead)"
  display    = "(Debug|Hash)"
  misc       = "(DeserializeOwned|Fn|FnMut|FnOnce|Read|Termination)"

  traits = "^(" marker "|" eq_partial "|" send_sync "|" borrow "|" convert "|" compare "|" io_common "|" display "|" misc ")$"
}

/^- \*\*[^*]+\*\*$/ {
  trait = $0
  gsub(/^- \*\*/, "", trait)
  gsub(/\*\*.*$/, "", trait)
  gsub(/^[[:space:]]*/, "", trait)
  gsub(/[[:space:]]*$/, "", trait)
  if (trait ~ traits) {
    skip = 1
    next
  }
  skip = 0
}

/^[^[:space:]]/ { skip = 0 }

skip { next }

{ print }
