# LD_LIBRARY_PATH=/media/mmcblk0p2/gfx-sysroot/usr/lib kmscube --atomic --count 300 2>&1 | tee /tmp/kmscube.log
LD_LIBRARY_PATH=/media/mmcblk0p2/gfx-sysroot/usr/lib RUST_BACKTRACE=1 ./ambient_display 2>&1 | tee /tmp/ambient_display.log 
