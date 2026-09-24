#!/usr/bin/env bash
set -eu

duration_seconds=3600
interval_seconds=5
output_dir="suica-benchmark-$(date -u +%Y%m%dT%H%M%SZ)"
health_url="http://127.0.0.1:3030/health/live"
core_pid=""
chromium_pid=""

usage() {
  printf '%s\n' \
    'Usage: benchmark-pi.sh [options]' \
    '  --duration-seconds N   total collection time (default: 3600)' \
    '  --interval-seconds N   seconds between samples (default: 5)' \
    '  --output-dir PATH      output directory' \
    '  --health-url URL       Core health endpoint' \
    '  --core-pid PID         fixed Suica Core PID' \
    '  --chromium-pid PID     fixed Chromium PID'
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --duration-seconds) duration_seconds=${2:?missing duration}; shift 2 ;;
    --interval-seconds) interval_seconds=${2:?missing interval}; shift 2 ;;
    --output-dir) output_dir=${2:?missing output directory}; shift 2 ;;
    --health-url) health_url=${2:?missing health URL}; shift 2 ;;
    --core-pid) core_pid=${2:?missing Core PID}; shift 2 ;;
    --chromium-pid) chromium_pid=${2:?missing Chromium PID}; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) printf 'Unknown option: %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

case "$duration_seconds:$interval_seconds" in
  *[!0-9:]*|0:*|*:0) printf 'Duration and interval must be positive integers.\n' >&2; exit 2 ;;
esac

mkdir -p "$output_dir"
samples_file="$output_dir/samples.csv"
metadata_file="$output_dir/metadata.txt"
summary_file="$output_dir/summary.json"

{
  printf 'started_at_utc=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  printf 'hostname=%s\n' "$(hostname 2>/dev/null || printf unknown)"
  printf 'kernel=%s\n' "$(uname -srmo 2>/dev/null || printf unknown)"
  if [ -r /proc/device-tree/model ]; then
    printf 'model='; tr -d '\000' < /proc/device-tree/model; printf '\n'
  else
    printf 'model=unavailable\n'
  fi
  if [ -r /etc/os-release ]; then
    sed 's/^/os_/' /etc/os-release
  fi
  command -v chromium >/dev/null 2>&1 && chromium --version 2>/dev/null || true
  command -v uxplay >/dev/null 2>&1 && uxplay -h 2>&1 | sed -n '1p' || true
} > "$metadata_file"

printf '%s\n' 'timestamp_utc,cpu_temp_c,load_1m,mem_available_kb,core_pid,core_rss_kb,chromium_pid,chromium_rss_kb,health_status,health_ms' > "$samples_file"

find_core_pid() {
  if [ -n "$core_pid" ]; then printf '%s' "$core_pid"; return; fi
  pgrep -xo suica-core 2>/dev/null || true
}

find_chromium_pid() {
  if [ -n "$chromium_pid" ]; then printf '%s' "$chromium_pid"; return; fi
  pgrep -fo 'chromium.*--user-data-dir' 2>/dev/null || pgrep -xo chromium 2>/dev/null || true
}

read_rss_kb() {
  pid=$1
  if [ -n "$pid" ] && [ -r "/proc/$pid/status" ]; then
    awk '$1 == "VmRSS:" { print $2; found=1 } END { if (!found) print "" }' "/proc/$pid/status"
  fi
}

read_temperature() {
  if [ -r /sys/class/thermal/thermal_zone0/temp ]; then
    awk '{ printf "%.1f", $1 / 1000 }' /sys/class/thermal/thermal_zone0/temp
  fi
}

start_epoch=$(date +%s)
end_epoch=$((start_epoch + duration_seconds))
sample_count=0

while [ "$(date +%s)" -le "$end_epoch" ]; do
  timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  temperature=$(read_temperature)
  load_1m=$(awk '{ print $1 }' /proc/loadavg 2>/dev/null || true)
  mem_available=$(awk '$1 == "MemAvailable:" { print $2 }' /proc/meminfo 2>/dev/null || true)
  current_core_pid=$(find_core_pid)
  current_chromium_pid=$(find_chromium_pid)
  core_rss=$(read_rss_kb "$current_core_pid")
  chromium_rss=$(read_rss_kb "$current_chromium_pid")
  health_status=""
  health_ms=""
  if command -v curl >/dev/null 2>&1; then
    health_result=$(curl --silent --show-error --output /dev/null --max-time 3 --write-out '%{http_code},%{time_total}' "$health_url" 2>/dev/null || true)
    health_status=${health_result%%,*}
    health_seconds=${health_result#*,}
    if [ "$health_result" != "$health_seconds" ] && [ -n "$health_seconds" ]; then
      health_ms=$(awk -v seconds="$health_seconds" 'BEGIN { printf "%.1f", seconds * 1000 }')
    fi
  fi
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$timestamp" "$temperature" "$load_1m" "$mem_available" \
    "$current_core_pid" "$core_rss" "$current_chromium_pid" "$chromium_rss" \
    "$health_status" "$health_ms" >> "$samples_file"
  sample_count=$((sample_count + 1))
  [ "$(date +%s)" -ge "$end_epoch" ] && break
  sleep "$interval_seconds"
done

finished_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
cat > "$summary_file" <<EOF
{
  "finishedAtUtc": "$finished_at",
  "durationSeconds": $duration_seconds,
  "intervalSeconds": $interval_seconds,
  "sampleCount": $sample_count
}
EOF

printf 'Wrote %s samples to %s\n' "$sample_count" "$output_dir"
