#!/usr/bin/env sh
# Kill the dashboard refresh loop, clear the screen, re-enable screensaver, and restart the Kindle UI so the user can use the device normally.

# Use stderr so this is visible even if stdout is lost when we kill start.sh below.
echo "calling stop.sh" >&2
DIR="$(dirname "$0")"
PID_FILE="${DIR}/.pid"

# Kill the refresh loop if we started it (script.sh writes this PID when it starts the loop)
if [ -f "$PID_FILE" ]; then
  pid=$(cat "$PID_FILE")
  kill "$pid" 2>/dev/null || true
  rm -f "$PID_FILE"
fi

# In case a refresh is mid-run, stop any script.sh that's still executing
pkill -f "bin/start.sh" 2>/dev/null || true

# Re-enable the screensaver so the lock button works normally again
lipc-set-prop com.lab126.powerd preventScreenSaver 0

# Restart the Kindle UI so the home screen and menus work again (no-op if we never stopped them)
/sbin/start framework   2>/dev/null || true
/sbin/start lab126_gui  2>/dev/null || true
