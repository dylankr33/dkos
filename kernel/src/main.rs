#![no_std]
#![no_main]


use core::arch::asm;
use limine::paging::Mode;
use uart_16550::SerialPort;

use crate::{
  requests::{BASE_REVISION, PAGING_MODE_REQUEST},
};

mod arch;
mod fbcon;
mod requests;
mod util;

const SERIAL_IO_PORT: u16 = 0x3F8;

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
  hcf();
}

/// Custom panic handler
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
  println!("{}", info);
  hcf();
}
