tmux \
  new-session  "wget -vvv http://raspberrypi.local:8080/download/ ; read" \; \
  split-window "wget -vvv http://raspberrypi.local:8080/download/ ; read" \; \
  split-window "wget -vvv http://raspberrypi.local:8080/download/ ; read" \; \
  split-window "wget -vvv http://raspberrypi.local:8080/download/ ; read" \; \
  select-layout even-vertical
