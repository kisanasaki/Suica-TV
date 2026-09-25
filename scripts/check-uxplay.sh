#!/usr/bin/env bash
# UxPlay運用に必要なbinary、GStreamer plugin、sink、mDNS、portを診断する。
# 環境は変更せず、必須項目が欠けている場合だけ非zeroで終了する。

set -u

failures=0
warnings=0

ok() { printf 'OK   %s\n' "$1"; }
warn() { printf 'WARN %s\n' "$1"; warnings=$((warnings + 1)); }
fail() { printf 'FAIL %s\n' "$1"; failures=$((failures + 1)); }

if command -v uxplay >/dev/null 2>&1; then
  ok "UxPlay binary: $(command -v uxplay)"
  uxplay -h 2>&1 | sed -n '1p'
else
  fail 'UxPlay is not installed (check: apt-cache policy uxplay)'
fi

if command -v gst-inspect-1.0 >/dev/null 2>&1; then
  ok "GStreamer: $(gst-inspect-1.0 --version 2>/dev/null | sed -n '1p')"
  for plugin in appsrc h264parse avdec_h264 audioconvert audioresample; do
    if gst-inspect-1.0 "$plugin" >/dev/null 2>&1; then
      ok "GStreamer plugin: $plugin"
    else
      fail "Missing GStreamer plugin: $plugin"
    fi
  done
  video_sink_found=0
  for sink in waylandsink glimagesink kmssink autovideosink; do
    if gst-inspect-1.0 "$sink" >/dev/null 2>&1; then
      ok "Video sink candidate: $sink"
      video_sink_found=1
    fi
  done
  [ "$video_sink_found" -eq 1 ] || fail 'No supported video sink candidate found'
  audio_sink_found=0
  for sink in pipewiresink pulsesink alsasink autoaudiosink; do
    if gst-inspect-1.0 "$sink" >/dev/null 2>&1; then
      ok "Audio sink candidate: $sink"
      audio_sink_found=1
    fi
  done
  [ "$audio_sink_found" -eq 1 ] || fail 'No supported audio sink candidate found'
else
  fail 'gst-inspect-1.0 is not installed'
fi

if command -v systemctl >/dev/null 2>&1 && systemctl is-active --quiet avahi-daemon 2>/dev/null; then
  ok 'avahi-daemon is active for mDNS discovery'
else
  warn 'avahi-daemon is not confirmed active; AirPlay discovery may fail'
fi

if command -v ss >/dev/null 2>&1; then
  if ss -lntu 2>/dev/null | awk '{print $5}' | grep -Eq ':(35000|35001|35002)$'; then
    warn 'One or more proposed UxPlay ports (35000-35002) are already in use'
  else
    ok 'Proposed UxPlay ports 35000-35002 appear available'
  fi
else
  warn 'ss is unavailable; port availability was not checked'
fi

printf '\nResult: %s failure(s), %s warning(s)\n' "$failures" "$warnings"
[ "$failures" -eq 0 ]
