#!/bin/bash
#
echo "\n✅ Supabase Storage bucket '$BUCKET_NAME' is accessible and working."

# Supabase Storage Performance Test Script
# Performs multiple upload/download operations and reports timing statistics

set -e

BUCKET_NAME="${SUPABASE_BUCKET_NAME:-storage-perf-test}"
NUM_OPS="${NUM_OPS:-20}"
FILE_SIZE_KB="${FILE_SIZE_KB:-4}"
STORAGE_API="$SUPABASE_URL/storage/v1/object"

if [ -z "$SUPABASE_URL" ] || [ -z "$SUPABASE_ANON_KEY" ]; then
    echo "❌ Error: SUPABASE_URL and SUPABASE_ANON_KEY must be set in your environment (.env)"
    exit 1
fi

echo "\n🧪 Supabase Storage Performance Test"
echo "====================================="
echo "Bucket: $BUCKET_NAME"
echo "Operations: $NUM_OPS (upload+download)"
echo "File size: $FILE_SIZE_KB KB"
echo ""

TMP_UPLOAD="/tmp/sb_perf_upload_$$.bin"
TMP_DOWNLOAD="/tmp/sb_perf_download_$$.bin"

# Generate random file of specified size
dd if=/dev/urandom of="$TMP_UPLOAD" bs=1024 count="$FILE_SIZE_KB" status=none

upload_times=()
download_times=()
success_count=0
fail_count=0


# Use gdate if available (for nanosecond precision on macOS), else fallback to date +%s (seconds)

# Use gdate if available (for nanosecond precision on macOS), else fallback to date +%s (seconds)
if command -v gdate >/dev/null 2>&1; then
    _date_cmd=gdate
    get_time() { $_date_cmd +%s%3N; }
else
    _date_cmd=date
    get_time() { $_date_cmd +%s; }
fi

for i in $(seq 1 "$NUM_OPS"); do
    FILE_NAME="perf_test_$(date +%s)_$i.bin"
    # Upload
    start=$(get_time)
    UPLOAD_CODE=$(curl -s -w "%{http_code}" -o /dev/null \
        -X POST \
        -H "apikey: $SUPABASE_ANON_KEY" \
        -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
        -H "Content-Type: application/octet-stream" \
        --data-binary "@$TMP_UPLOAD" \
        "$STORAGE_API/$BUCKET_NAME/$FILE_NAME")
    end=$(get_time)
    elapsed=$((end - start))
    upload_times+=("$elapsed")
    if [ "$UPLOAD_CODE" = "200" ] || [ "$UPLOAD_CODE" = "201" ]; then
        # Download
        start=$(get_time)
        DOWNLOAD_CODE=$(curl -s -w "%{http_code}" -o "$TMP_DOWNLOAD" \
            -H "apikey: $SUPABASE_ANON_KEY" \
            -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
            "$STORAGE_API/$BUCKET_NAME/$FILE_NAME")
        end=$(get_time)
        elapsed=$((end - start))
        download_times+=("$elapsed")
        if [ "$DOWNLOAD_CODE" = "200" ] && cmp -s "$TMP_UPLOAD" "$TMP_DOWNLOAD"; then
            success_count=$((success_count+1))
        else
            echo "   ❌ Download or content mismatch for $FILE_NAME"
            fail_count=$((fail_count+1))
        fi
    else
        echo "   ❌ Upload failed for $FILE_NAME (HTTP $UPLOAD_CODE)"
        fail_count=$((fail_count+1))
    fi
    rm -f "$TMP_DOWNLOAD"
done

rm -f "$TMP_UPLOAD"

# Calculate stats
sum() { local s=0; for v in "$@"; do s=$((s+v)); done; echo $s; }
min() { local m=9999999; for v in "$@"; do [ "$v" -lt "$m" ] && m=$v; done; echo $m; }
max() { local m=0; for v in "$@"; do [ "$v" -gt "$m" ] && m=$v; done; echo $m; }
avg() { local s=$(sum "$@"); local n=${#@}; [ "$n" -gt 0 ] && echo $((s/n)) || echo 0; }

echo "\nResults:"
echo "--------"
echo "Success: $success_count/$NUM_OPS"
echo "Failures: $fail_count"
echo "Upload times (ms): min=$(min "${upload_times[@]}") max=$(max "${upload_times[@]}") avg=$(avg "${upload_times[@]}")"
echo "Download times (ms): min=$(min "${download_times[@]}") max=$(max "${download_times[@]}") avg=$(avg "${download_times[@]}")"

if [ "$fail_count" -eq 0 ]; then
    echo "\n🎉 Supabase Storage performance test PASSED."
else
    echo "\n⚠️  Supabase Storage performance test completed with failures."
fi

echo "\nTip: Set NUM_OPS and FILE_SIZE_KB env vars to control test size."
echo "Example: NUM_OPS=100 FILE_SIZE_KB=16 bash lab/scripts/test_supabase_storage.sh"
