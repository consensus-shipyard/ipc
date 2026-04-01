#!/usr/bin/env bash
set -euo pipefail

CLI=./target/release/ipc-cli
GW=http://136.115.12.207:8080

pass() { echo "  ✅ PASS"; }
fail() { echo "  ❌ FAIL: $1"; exit 1; }

# ── Setup: ensure credit + bucket exist ──────────────────────────────────────
echo "=== 0. Setup ==="

echo "  Buying 0.1 FIL of storage credit..."
$CLI storage client credit buy 0.1 || fail "credit buy"

echo "  Checking for existing buckets..."
BUCKET=$($CLI storage client bucket list 2>/dev/null \
  | grep -oE 't0[0-9]+' | head -1 || true)

if [ -z "$BUCKET" ]; then
  echo "  No bucket found — creating one..."
  $CLI storage client bucket create || fail "bucket create"
  sleep 5
  BUCKET=$($CLI storage client bucket list 2>/dev/null \
    | grep -oE 't0[0-9]+' | head -1 || true)
  [ -n "$BUCKET" ] || fail "could not find bucket after creation"
fi
echo "  Using bucket: $BUCKET"
pass

# ── Cleanup old test data ────────────────────────────────────────────────────
echo "=== 1. Cleanup ==="
$CLI storage client rm -r --force ipc://$BUCKET/t/ 2>&1 || true
echo "  (pending blobs may prevent full cleanup — that's OK)"

rm -rf /tmp/ipc-test /tmp/ipc-dl
mkdir -p /tmp/ipc-test/subdir
echo "hello ipc storage"   > /tmp/ipc-test/file1.txt
echo "second file"         > /tmp/ipc-test/file2.txt
echo "nested file"         > /tmp/ipc-test/subdir/nested.txt
pass

# ── Phase 1: Upload & Read ───────────────────────────────────────────────────
echo "=== 2. credit info ==="
$CLI storage client credit info || fail "credit info"
pass

echo "=== 3. bucket list ==="
$CLI storage client bucket list || fail "bucket list"
pass

echo "=== 4. upload file1.txt ==="
$CLI storage client cp --overwrite /tmp/ipc-test/file1.txt ipc://$BUCKET/t/file1.txt --gateway $GW \
  || fail "upload file1"
pass

echo "=== 5. upload file2.txt ==="
$CLI storage client cp --overwrite /tmp/ipc-test/file2.txt ipc://$BUCKET/t/file2.txt --gateway $GW \
  || fail "upload file2"
pass

echo "=== 6. upload directory recursively ==="
$CLI storage client cp -r --overwrite /tmp/ipc-test/subdir ipc://$BUCKET/t/subdir --gateway $GW \
  || fail "upload directory"
pass

echo "=== 7. list all ==="
$CLI storage client ls ipc://$BUCKET/ || fail "ls all"
pass

echo "=== 8. list prefix t/ ==="
$CLI storage client ls ipc://$BUCKET/t/ || fail "ls prefix"
pass

echo "=== 9. stat ==="
$CLI storage client stat ipc://$BUCKET/t/file1.txt || fail "stat"
pass

echo "=== 10. cat ==="
OUTPUT=$($CLI storage client cat ipc://$BUCKET/t/file1.txt --gateway $GW)
echo "$OUTPUT"
[ "$OUTPUT" = "hello ipc storage" ] || fail "cat content mismatch"
pass

echo "=== 11. download single file ==="
mkdir -p /tmp/ipc-dl
$CLI storage client cp ipc://$BUCKET/t/file2.txt /tmp/ipc-dl/file2.txt --gateway $GW \
  || fail "download single"
CONTENT=$(cat /tmp/ipc-dl/file2.txt)
echo "$CONTENT"
[ "$CONTENT" = "second file" ] || fail "download content mismatch"
pass

echo "=== 12. download directory recursively ==="
$CLI storage client cp -r ipc://$BUCKET/t/subdir /tmp/ipc-dl/subdir --gateway $GW \
  || fail "download directory"
[ -f /tmp/ipc-dl/subdir/nested.txt ] || fail "nested.txt not downloaded"
pass

# ── Phase 2: Mutations (need blob finalization) ──────────────────────────────
echo ""
echo "── Waiting 90s for blob finalization before testing mv/rm... ──"
sleep 90

echo "=== 13. move file2 -> file2-renamed ==="
$CLI storage client mv ipc://$BUCKET/t/file2.txt ipc://$BUCKET/t/file2-renamed.txt \
  --gateway $GW || fail "mv"
pass

echo "=== 14. verify rename ==="
$CLI storage client ls ipc://$BUCKET/t/
pass

echo "=== 15. rm file2-renamed ==="
$CLI storage client rm --force ipc://$BUCKET/t/file2-renamed.txt || fail "rm single"
pass

echo "=== 16. rm file1 ==="
$CLI storage client rm --force ipc://$BUCKET/t/file1.txt || fail "rm file1"
pass

echo "=== 17. rm -r t/subdir ==="
$CLI storage client rm -r --force ipc://$BUCKET/t/subdir || fail "rm recursive"
pass

echo "=== 18. final list ==="
$CLI storage client ls ipc://$BUCKET/
pass

echo ""
echo "════════════════════════════════════════════════"
echo "  All 18 steps passed!"
echo "════════════════════════════════════════════════"

rm -rf /tmp/ipc-test /tmp/ipc-dl
