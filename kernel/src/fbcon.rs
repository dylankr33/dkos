use core::fmt;
use lazy_static::lazy_static;
use limine::framebuffer::Framebuffer;
use spin::Mutex;
use spleen_font::{FONT_12X24, PSF2Font};

use crate::requests::FRAMEBUFFER_REQUEST;

lazy_static! {
  pub static ref FB_WRITER: Mutex<FramebufferWriter<'static>> = {
    let font = PSF2Font::new(FONT_12X24).unwrap();
    let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().unwrap();
    let framebuffer = framebuffer_response.framebuffers().next().unwrap();
    let writer = Mutex::new(FramebufferWriter::new(framebuffer, font));
    writer
  };
}

/// Print macro.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::fbcon::_print(format_args!($($arg)*)));
}

/// Print line macro.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Print function for macros.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  use core::fmt::Write;
  FB_WRITER.lock().write_fmt(args).unwrap();
}

/// Display trait
pub trait FramebufferDisplay {
  // All this function needs is to be able to write a pixel to the screen
  unsafe fn write_pixel(&self, code: u32, x: u64, y: u64);
}

impl<'a> FramebufferDisplay for Framebuffer<'a> {
  unsafe fn write_pixel(&self, color: u32, x: u64, y: u64) {
    let pixel_offset = y * self.pitch() + x * 4;
    unsafe {
      self
        .addr()
        .add(pixel_offset as usize)
        .cast::<u32>()
        .write(color);
    }
  }
}

/// Struct for writing to the framebuffer. Usually hidden behind a `Mutex`
pub struct FramebufferWriter<'a> {
  pub framebuffer: Framebuffer<'a>,
  pub font: PSF2Font<'a>,
  col_x: usize,
  row_y: usize,
}

impl<'a> FramebufferWriter<'a> {
  pub fn new(framebuffer: Framebuffer<'a>, font: PSF2Font<'a>) -> Self {
    Self {
      framebuffer,
      font,
      col_x: 0,
      row_y: 0,
    }
  }
}

impl<'a> fmt::Write for FramebufferWriter<'a> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    let mut tmp = [0u8, 2];
    for char in s.chars() {
      let bytes = char.encode_utf8(&mut tmp).as_bytes();
      if bytes == &[0x000A_u8][..]
        || self.col_x == self.framebuffer.width() as usize / self.font.width as usize
      {
        self.row_y += 1;
        self.col_x = 0;
      } else {
        let _ = self.write_char(char);
        self.col_x += 1;
      }
    }
    Ok(())
  }

  fn write_char(&mut self, text: char) -> Result<(), core::fmt::Error> {
    let mut tmp = [0u8, 2];
    let bytes = text.encode_utf8(&mut tmp).as_bytes();
    if let Some(glyph) = self.font.glyph_for_utf8(bytes) {
      for (row_y, row) in glyph.enumerate() {
        for (col_x, on) in row.enumerate() {
          unsafe {
            if on {
              self.framebuffer.write_pixel(
                0xFFFFFFFF,
                (self.col_x * self.font.width as usize + col_x) as u64,
                (self.row_y * self.font.header_size as usize + row_y) as u64,
              );
            }
          }
        }
      }
    }

    Ok(())
  }
}
