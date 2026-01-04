#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_default)]

use core::arch::asm;
use limine::{
  memory_map::{Entry, EntryType},
  paging::Mode,
};
use uart_16550::SerialPort;

use crate::{
  mem::pmm::{PMM, Pmm},
  requests::{BASE_REVISION, HHDM_RESPONSE, MEMORY_MAP_RESPONSE, PAGING_MODE_REQUEST},
};

mod arch;
mod fbcon;
mod mem;
mod requests;

const SERIAL_IO_PORT: u16 = 0x3F8;

/// Debugging function that tests the #DE CPU exception
#[allow(unused)]
fn divide_by_zero() {
  unsafe { asm!("mov dx, 0; div dx") }
}

/// Debugger breakpoint that can be broken out of in GDB with `set $pc += 2`
#[allow(unused)]
fn breakpoint() {
  unsafe { asm!("2: jmp 2b") }
}

/// Halt & catch fire
fn hcf() -> ! {
  loop {
    unsafe {
      #[cfg(target_arch = "x86_64")]
      asm!("hlt");
    }
  }
}

/// This is the kernel entrypoint
#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
  assert!(BASE_REVISION.is_supported());
  assert!(PAGING_MODE_REQUEST.get_response().unwrap().mode() == Mode::MIN);

  let mut serial_port = unsafe { SerialPort::new(SERIAL_IO_PORT) };
  serial_port.init();
  println!("dkos 0.1.0");
  #[cfg(target_arch = "x86_64")]
  {
    arch::x86_64::gdt::init_gdt();
    arch::x86_64::idt::init_idt();
  }
  for entry in MEMORY_MAP_RESPONSE.entries() {
    println!("0x{:x}", entry.base + HHDM_RESPONSE.offset());
  }
  hcf();
}

/// Custom panic handler
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
  println!("{}", info);
  hcf();
}
