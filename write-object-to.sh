#!/bin/bash
MOUNT_POINT=$1

if [ -z "$1" ]; then
    echo "no argument"
    exit 1
fi

mkdir -p "$MOUNT_POINT/EFI/BOOT"
cp ./bootloader/target/x86_64-unknown-uefi/release/bootloader.efi "$MOUNT_POINT/EFI/BOOT/BOOTX64.EFI"
cp ./kernel/kernel.elf "$MOUNT_POINT/kernel.elf"

