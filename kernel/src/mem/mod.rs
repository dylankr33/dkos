use limine::memory_map::EntryType;
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::requests::{HHDM_RESPONSE, MEMORY_MAP_RESPONSE};

/// Physical Memory Address
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PhysAddr(pub usize);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct VirtAddr(pub usize);

impl From<VirtAddr> for PhysAddr {
  fn from(value: VirtAddr) -> Self {
    Self(value.0 - HHDM_RESPONSE.offset() as usize)
  }
}

impl From<PhysAddr> for VirtAddr {
  fn from(value: PhysAddr) -> Self {
    Self(value.0 + HHDM_RESPONSE.offset() as usize)
  }
}

#[global_allocator]
static ALLOC: Talck<spin::Mutex<()>, ClaimOnOom> =
  Talc::new(unsafe { ClaimOnOom::new(Span::empty()) }).lock();

pub fn init_heap() {
  let offset = HHDM_RESPONSE.offset();
  let entry = MEMORY_MAP_RESPONSE
    .entries()
    .iter()
    .filter(|e| e.entry_type == EntryType::USABLE)
    .next()
    .unwrap();
  let base = (entry.base + HHDM_RESPONSE.offset()) as *mut u8;
  let acme = (entry.base + entry.length + HHDM_RESPONSE.offset()) as *mut u8;
  unsafe {
    let _ = ALLOC.lock().claim(Span::new(base, acme));
  }
}
