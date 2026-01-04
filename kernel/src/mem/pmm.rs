use core::fmt::Debug;

use lazy_static::lazy_static;
use limine::memory_map::EntryType;
use spin::lock_api::Mutex;

use crate::requests::{HHDM_RESPONSE, MEMORY_MAP_RESPONSE};

/// Physical Memory Address
#[repr(transparent)]
pub struct PhysAddr(pub u64);

lazy_static! {
  pub static ref PMM: Mutex<BitmapAllocator> = Mutex::new(BitmapAllocator::new());
}

/// Trait for a Physical Memory Manager
pub trait Pmm {
  /// Allocate one page
  fn alloc(&mut self) -> Option<PhysAddr>;
  /// Deallocate that page.
  /// ```rust
  /// if let Some(page) = PMM.lock().alloc() {
  ///   todo!();
  ///   PMM.dealloc(page);
  ///  };
  /// ```
  fn dealloc(&mut self, pages: PhysAddr) -> Result<(), ()>;
}

/// Struct for a bitmap allocator
pub struct BitmapAllocator {
  base: u64,
  bitmap: [u8; 512],
}

impl BitmapAllocator {
  pub fn new() -> Self {
    let entry = MEMORY_MAP_RESPONSE
      .entries()
      .iter()
      .filter(|e| e.entry_type == EntryType::USABLE && e.length >= 512 * 4096)
      .next()
      .unwrap();

    Self {
      base: entry.base,
      bitmap: [0; 512],
    }
  }
}

impl Pmm for BitmapAllocator {
  fn alloc(&mut self) -> Option<PhysAddr> {
    let index;
    for (i, page) in self.bitmap.into_iter().enumerate() {
      if page == 0x1 {
        continue;
      } else if page == 0x0 {
        index = i;
        self.bitmap[index] = 0x1;
        return Some(PhysAddr(
          self.base + HHDM_RESPONSE.offset() + index as u64 * 4096,
        ));
      }
    }
    None
  }

  fn dealloc(&mut self, page: PhysAddr) -> Result<(), ()> {
    let index = page.0 - self.base / 4096;
    if self.bitmap[index as usize] == 0 {
      return Err(());
    }
    self.bitmap[index as usize] = 0;
    Ok(())
  }
}
