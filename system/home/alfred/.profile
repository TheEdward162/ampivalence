# LD_LIBRARY_PATH=/media/mmcblk0p2/gfx-sysroot/usr/lib kmscube --atomic --count 300 2>&1 | tee /tmp/kmscube.log
# LD_LIBRARY_PATH=/media/mmcblk0p2/gfx-sysroot/usr/lib RUST_BACKTRACE=1 ./ambient_display 2>&1 | tee /tmp/ambient_display.log 

GFX_SYSROOT='/media/mmcblk0p2/gfx-sysroot'
#LD_LIBRARY_PATH="${GFX_SYSROOT}/usr/lib" LIBGL_DRIVERS_PATH="${GFX_SYSROOT}/usr/lib/xorg/modules/dri" kmscube --atomic --count 500 2>&1 | tee /tmp/kmscube.log
# LD_LIBRARY_PATH="${GFX_SYSROOT}/usr/lib" LIBGL_DRIVERS_PATH="${GFX_SYSROOT}/usr/lib/xorg/modules/dri" RUST_BACKTRACE=1 ./test_kmscube 2>&1 | tee /tmp/kmscube.log
LD_LIBRARY_PATH="${GFX_SYSROOT}/usr/lib" LIBGL_DRIVERS_PATH="${GFX_SYSROOT}/usr/lib/xorg/modules/dri" RUST_BACKTRACE=1 /tmp/test_kmscube 2>&1 | tee /tmp/kmscube.log
