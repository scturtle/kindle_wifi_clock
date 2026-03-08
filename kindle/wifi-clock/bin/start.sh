#!/usr/bin/env sh

# ignore HUP since kual will exit after pressing start, and that might kill our long running script
trap '' HUP

DIR="$(dirname "$0")"
PID_FILE="${DIR}/.pid"
URL="https://time.scturtle.me"

refresh_screen() {
  curl -k "$URL/image" -o "$DIR/screen.png"
  eips -c
  eips -c
  eips -g "$DIR/screen.png" -x 0 -y 10 -w du0
  # Draw battery at top (eips can't print %, so we strip it from gasgauge-info -c)
  eips 0 0 "                                                $(gasgauge-info -c 2>/dev/null | sed 's/%//g' || echo '?')"
}

# Keep the screen on (no screensaver) while the dashboard is running
lipc-set-prop com.lab126.powerd preventScreenSaver 1

# ignore term since stopping the framework/gui will send a TERM signal to our script since kual is probably related to the GUI
trap '' TERM
# Stop the Kindle UI so only our image + date/battery are visible (cleaner full-screen dashboard).
/sbin/stop framework
/sbin/stop lab126_gui
sleep 2
trap - TERM

# Refresh loop in background: fetch and display synchronized with the server's minute.
(
  while true; do
    refresh_screen
    
    # 1. Get the current second from the URL (silent mode, ignore cert errors)
    SEC=$(curl -sk "$URL/second")

    # Validate that SEC is a number; fallback to 0 if it's not
    if ! [ "$SEC" -eq "$SEC" ] 2>/dev/null; then
      SEC=0
    fi

    # 2. Calculate sleep time (60 - sec)
    SLEEP_TIME=$((60 - SEC))

    # Ensure SLEEP_TIME is sensible (between 1 and 60)
    if [ "$SLEEP_TIME" -le 0 ]; then
      SLEEP_TIME=60
    fi

    sleep "$SLEEP_TIME"
  done
) &

echo $! > "$PID_FILE"

# if home button is pressed, run the stop script
script -q -c "evtest /dev/input/event2 2>&1" /dev/null | grep -m 1 -q "code 102 (Home), value 1" && "$DIR/stop.sh"
exit 0
