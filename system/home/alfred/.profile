export XDG_RUNTIME_DIR="/tmp/${USER}-runtime"

rm -r "$XDG_RUNTIME_DIR" 2>/dev/null
mkdir -p "$XDG_RUNTIME_DIR"

LIBSEAT_BACKEND=seatd WLR_LIBINPUT_NO_DEVICES=0 sway
#-d >/tmp/sway.log 2>&1

