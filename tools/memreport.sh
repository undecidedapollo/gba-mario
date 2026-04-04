#!/bin/sh
# Memory report for the mario GBA build.
# Parses target/mario.map and prints per-region usage with top consumers.
#
# Usage:
#   tools/memreport.sh                 # default release build
#   tools/memreport.sh --debug         # debug build
#   tools/memreport.sh --top 30        # show top 30 items per region
#   tools/memreport.sh --all           # show every item
#   tools/memreport.sh --map PATH      # use a specific map file

set -e

# ---- args ----
PROFILE="release"
TOP=10
SHOW_ALL=0
MAP=""

while [ $# -gt 0 ]; do
  case "$1" in
    --debug) PROFILE="debug"; shift ;;
    --release) PROFILE="release"; shift ;;
    --top) TOP="$2"; shift 2 ;;
    --all) SHOW_ALL=1; shift ;;
    --map) MAP="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

# cd to repo root (the directory above this script)
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

[ -z "$MAP" ] && MAP="target/mario.map"
ELF="target/thumbv4t-none-eabi/$PROFILE/mario"

if [ ! -f "$MAP" ]; then
  echo "map file not found: $MAP" >&2
  echo "build first with 'cargo gba-build' (or pass --map PATH)" >&2
  exit 1
fi

# ---- region sizes (prefer the ELF if present, else pull from the map) ----
if [ -f "$ELF" ] && command -v arm-none-eabi-size >/dev/null 2>&1; then
  SIZE_OUT="$(arm-none-eabi-size -A -x "$ELF")"
  get_sz() {
    echo "$SIZE_OUT" | awk -v s="$1" '
      function hex2dec(h,   n,i,c,v) {
        sub(/^0[xX]/, "", h); n=0
        for(i=1;i<=length(h);i++){ c=tolower(substr(h,i,1))
          v=(c>="0"&&c<="9")?c+0:index("abcdef",c)+9; n=n*16+v }
        return n
      }
      $1==s { print hex2dec($2) }'
  }
  TEXT=$(get_sz .text)
  RODATA=$(get_sz .rodata)
  DATA=$(get_sz .data)
  EWRAM=$(get_sz .ewram)
  BSS=$(get_sz .bss)
else
  # fall back to map header lines
  HEX2DEC='function hex2dec(h,   n,i,c,v) {
    sub(/^0[xX]/, "", h); n=0
    for(i=1;i<=length(h);i++){ c=tolower(substr(h,i,1))
      v=(c>="0"&&c<="9")?c+0:index("abcdef",c)+9; n=n*16+v }
    return n }'
  EWRAM=$(awk "$HEX2DEC"'  /\.ewram$/  && NF==5 { print hex2dec($3); exit }' "$MAP")
  DATA=$(awk "$HEX2DEC"'   /\.data$/   && NF==5 { print hex2dec($3); exit }' "$MAP")
  BSS=$(awk "$HEX2DEC"'    /\.bss$/    && NF==5 { print hex2dec($3); exit }' "$MAP")
  TEXT=$(awk "$HEX2DEC"'   /\.text$/   && NF==5 { print hex2dec($3); exit }' "$MAP")
  RODATA=$(awk "$HEX2DEC"' /\.rodata$/ && NF==5 { print hex2dec($3); exit }' "$MAP")
fi

: "${TEXT:=0}" "${RODATA:=0}" "${DATA:=0}" "${EWRAM:=0}" "${BSS:=0}"

EWRAM_CAP=$((256 * 1024))
IWRAM_CAP=$((32 * 1024))
IWRAM_USED=$((DATA + BSS))
EWRAM_FREE=$((EWRAM_CAP - EWRAM))
IWRAM_FREE=$((IWRAM_CAP - IWRAM_USED))
ROM_USED=$((TEXT + RODATA + DATA + EWRAM))

pct() { awk -v u="$1" -v c="$2" 'BEGIN{ printf "%.1f", (u*100.0)/c }'; }
fmt() { awk -v n="$1" 'BEGIN{ printf "%'"'"'d", n }' 2>/dev/null || printf "%d" "$1"; }

echo "=== mario memory report ($PROFILE) ==="
printf "\n"
printf "  Region  %10s %10s %10s  %6s\n" "used" "free" "capacity" "%"
printf "  ------  %10s %10s %10s  %6s\n" "--------" "--------" "--------" "------"
printf "  EWRAM   %10s %10s %10s  %5s%%\n" \
  "$(fmt $EWRAM)" "$(fmt $EWRAM_FREE)" "$(fmt $EWRAM_CAP)" "$(pct $EWRAM $EWRAM_CAP)"
printf "  IWRAM   %10s %10s %10s  %5s%%\n" \
  "$(fmt $IWRAM_USED)" "$(fmt $IWRAM_FREE)" "$(fmt $IWRAM_CAP)" "$(pct $IWRAM_USED $IWRAM_CAP)"
printf "\n  IWRAM breakdown:  .data=%s  .bss=%s\n" "$(fmt $DATA)" "$(fmt $BSS)"
printf "  ROM image:        %s bytes  (.text=%s  .rodata=%s  .data=%s  .ewram=%s)\n" \
  "$(fmt $ROM_USED)" "$(fmt $TEXT)" "$(fmt $RODATA)" "$(fmt $DATA)" "$(fmt $EWRAM)"

# ---- top consumers ----
# Section ranges in the map (VMA column):
#   EWRAM:        2000000 .. 30000d0
#   IWRAM (.data+.bss): 3000000 .. 3008000
#   ROM (.text+.rodata): 8000000 .. 8200000

# Demangle Rust symbols if rustfilt is around, else leave mangled.
DEMANGLE="cat"
command -v rustfilt >/dev/null 2>&1 && DEMANGLE="rustfilt"

extract_region() {
  # $1 = start VMA (hex, no 0x), $2 = end VMA (hex, no 0x)
  # Emit lines of: "<size_dec> <symbol>" for symbol entries inside that range.
  # A symbol line in the LLD map has exactly 5 fields:
  #   VMA  LMA  SIZE  ALIGN  NAME
  # and its SIZE > 0 and NAME doesn't start with '.' or '='.
  awk -v lo="$1" -v hi="$2" '
    function hex2dec(h,   n,i,c,v) {
      sub(/^0[xX]/, "", h); n=0
      for(i=1;i<=length(h);i++){ c=tolower(substr(h,i,1))
        v=(c>="0"&&c<="9")?c+0:index("abcdef",c)+9; n=n*16+v }
      return n
    }
    BEGIN { LO=hex2dec(lo); HI=hex2dec(hi) }
    {
      vma = hex2dec($1)
      sz  = hex2dec($3)
    }
    NF>=5 && vma>=LO && vma<HI && sz>0 {
      # Only real symbol entries: NAME field ($5..) should not start with "." or "=" or "$"
      name=$5
      for (i=6;i<=NF;i++) name=name" "$i
      if (name ~ /^[.=]/) next
      if (name ~ /^\$[adt]$/) next      # mapping symbols
      if (name ~ /__[a-z]+_(start|end|position|word_|capacity|used|free)/) next
      # skip input-section group headers (paths like .../libfoo.rlib(...):(.section))
      if (name ~ /\.(rlib|o)[\)]?:\(/) next
      if (name ~ /^\//) next
      print sz"\t"name
    }
  ' "$MAP" | sort -rn -k1,1
}

show_region() {
  LABEL="$1"; START="$2"; END="$3"; TOTAL="$4"
  echo
  echo "=== Top consumers in $LABEL ==="
  LINES="$(extract_region "$START" "$END")"
  if [ -z "$LINES" ]; then
    echo "  (no entries)"
    return
  fi
  if [ "$SHOW_ALL" = "1" ]; then
    COUNT=$(echo "$LINES" | wc -l | tr -d ' ')
  else
    COUNT=$TOP
  fi
  printf "  %8s  %5s  %s\n" "bytes" "%"  "symbol"
  printf "  %8s  %5s  %s\n" "--------" "-----" "--------------------------"
  echo "$LINES" | head -n "$COUNT" | while IFS="$(printf '\t')" read -r SZ NAME; do
    DNAME="$(printf "%s" "$NAME" | $DEMANGLE)"
    P=$(awk -v u="$SZ" -v c="$TOTAL" 'BEGIN{ if(c==0){print "0.0"}else{printf "%.1f",(u*100.0)/c} }')
    printf "  %8s  %4s%%  %s\n" "$(fmt $SZ)" "$P" "$DNAME"
  done
  REMAINING=$(echo "$LINES" | wc -l | tr -d ' ')
  if [ "$SHOW_ALL" != "1" ] && [ "$REMAINING" -gt "$COUNT" ]; then
    HIDDEN=$((REMAINING - COUNT))
    echo "  ... $HIDDEN more (use --all or --top N to see more)"
  fi
}

show_region "EWRAM"         "2000000" "3000000" "$EWRAM"
show_region "IWRAM (.data)" "3000000" "30000d0" "$DATA"
show_region "IWRAM (.bss)"  "30000d0" "3008000" "$BSS"

echo
