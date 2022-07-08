#!/bin/sh

SYSROOT='/media/mmcblk0p2/gfx-sysroot'

# quit if sysroot already exists to avoid mistakes
mkdir $SYSROOT || exit 1

# create the sysrooot and copy repo keys from parent 
mkdir -p $SYSROOT/etc/apk
cp -r /etc/apk/keys $SYSROOT/etc/apk/keys

# pull mesa dependencies into the sysroot - this is about 160MB of libraries (including llvm)
apk add --initdb --repositories-file /etc/apk/repositories --root $SYSROOT mesa-gbm mesa-egl mesa-dri-gallium mesa-dev

# for /usr/lib/xorg/modules/dri/vc4_dri.so which MESA-LOADER cannot find
# ln -sf $SYSROOT/usr/lib/xorg /usr/lib/xorg USE LIBGL_DRIVERS_PATH="${GFX_SYSROOT}/usr/lib/xorg/modules/dri" instead

# then launch app with LD_LIBRARY_PATH=/media/mmcblk0p2/gfx-sysroot/usr/lib <app>
