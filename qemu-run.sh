#!/bin/bash
if [ -z "$1" ];  then
   IMG_NAME=rikan.img
else
   IMG_NAME="$1"
fi

DEVENV_DIR=~/osbook/devenv
mkdir -p OVMFs/
cp $DEVENV_DIR/OVMF_CODE.fd ./OVMFs/
cp $DEVENV_DIR/OVMF_VARS.fd ./OVMFs/
qemu-system-x86_64 \
    -monitor stdio \
    -drive if=pflash,format=raw,readonly,file=./OVMFs/OVMF_CODE.fd \
    -drive if=pflash,format=raw,file=./OVMFs/OVMF_VARS.fd \
    -drive if=ide,index=0,media=disk,format=raw,file=$IMG_NAME \
    -device nec-usb-xhci,id=xhci \
    -device usb-mouse -device usb-kbd
