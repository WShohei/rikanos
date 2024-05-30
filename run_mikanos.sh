source $HOME/osbook/devenv/buildenv.sh
cd ~/workspace/mikanos/kernel
make
cd ~/workspace/rikanos
~/osbook/devenv/run_qemu.sh ~/edk2/Build/MikanLoaderX64/DEBUG_CLANG38/X64/Loader.efi ~/workspace/mikanos/kernel/kernel.elf
