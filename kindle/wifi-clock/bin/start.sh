#!/usr/bin/env sh

# ignore HUP since kual will exit after pressing start, and that might kill our long running script
trap '' HUP

DIR="$(dirname "$0")"
PID_FILE="${DIR}/.pid"
URL="https://time.scturtle.me"

refresh_screen() {
  BAT=$(gasgauge-info -c 2>/dev/null | sed 's/%//g' || echo '?')
  if curl -k --connect-timeout 5 -m 15 "$URL/image" -o "$DIR/screen.png"; then
    eips -c
    eips -c
    eips -g "$DIR/screen.png" -x 0 -y 10 -w du0
    eips 0 0 "                                               ${BAT}"
  else
    eips 0 0 "Sync Failed                                    ${BAT}"
    if [ "$(lipc-get-prop com.lab126.wifid cmState)" != "CONNECTED" ]; then
      lipc-set-prop com.lab126.cmd wirelessEnable 0
      sleep 30
      lipc-set-prop com.lab126.cmd wirelessEnable 1
      sleep 30
    fi
  fi
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
    
    # Get the current second from the URL (silent mode, ignore cert errors)
    SEC=$(curl -sk "$URL/second" --connect-timeout 5 --max-time 15)

    # Validate that SEC is a number; fallback to 0 if it's not
    case "$SEC" in
      [0-9] | [0-5][0-9])
        ;;
      *)
        SEC=0
        ;;
    esac

    sleep "$((60 - SEC))"
  done
) &

echo $! > "$PID_FILE"

# if home button is pressed, run the stop script
script -q -c "evtest /dev/input/event2 2>&1" /dev/null | grep -m 1 -q "code 102 (Home), value 1" && "$DIR/stop.sh"
exit 0
