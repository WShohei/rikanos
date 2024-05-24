#![no_std]
#![no_main]

use uefi::prelude::*;
use log::info;

#[entry]
#[allow(dead_code)]
fn efi_main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi::helpers::init(&mut system_table).unwrap();
    info!("Hello, UEFI!");
    system_table.boot_services().stall(10_000_000);

    loop {}

    Status::SUCCESS
}
