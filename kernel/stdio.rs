// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::fmt::{self, Write};

struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let b = s.as_bytes();
        let end = self.pos + b.len();

        if end > self.buf.len() {
            return Err(fmt::Error);
        }

        self.buf[self.pos..end].copy_from_slice(b);
        self.pos = end;
        Ok(())
    }
}

pub fn vsprintf<'a>(buf: &'a mut [u8], args: fmt::Arguments<'_>) -> Result<&'a str, fmt::Error> {
    let mut w = BufWriter { buf, pos: 0 };
    w.write_fmt(args)?;
    let n = w.pos;
    let buf = w.buf;

    core::str::from_utf8(&buf[..n]).map_err(|_| fmt::Error)
}

#[macro_export]
macro_rules! sprintf {
    ($buf:expr, $($arg:tt)*) => {
        vsprintf($buf, format_args!($($arg)*))
    }
}

