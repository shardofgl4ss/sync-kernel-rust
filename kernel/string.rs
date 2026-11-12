use core::slice;
use core::str;

pub trait PtrToStr {
	/// # Safety
	/// to_str() should always have a valid UTF-8 or ASCII string passed.
	unsafe fn to_str(self) -> Result<&'static str, str::Utf8Error>;
	/// # Safety
	/// to_str() should always have a valid UTF-8 or ASCII string passed.
	/// failure to do so for unchecked() will result in a panic
	unsafe fn to_str_unchecked(self) -> &'static str;
}

impl PtrToStr for *const u8 {
	unsafe fn to_str(self) -> Result<&'static str, str::Utf8Error> {
		let mut len = 0;
		unsafe {
			while *self.add(len) != 0 {
				len += 1;
			}
			let byts = slice::from_raw_parts(self, len);
			str::from_utf8(byts)
		}
	}
	unsafe fn to_str_unchecked(self) -> &'static str {
		let mut len = 0;
		unsafe {
			while *self.add(len) != 0 {
				len += 1;
			}
			let byts = slice::from_raw_parts(self, len);
			str::from_utf8_unchecked(byts)
		}
	}
}
