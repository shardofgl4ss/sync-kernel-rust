// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use core::ptr::write_volatile;
use synos_common::Global;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ColorCode(u8);
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum color {
    BLACK = 0,
    BLUE = 1,
    GREEN = 2,
    CYAN = 3,
    RED = 4,
    MAGENTA = 5,
    BROWN = 6,
    LIGHT_GREY = 7,
    DARK_GREY = 8,
    LIGHT_BLUE = 9,
    LIGHT_GREEN = 10,
    LIGHT_CYAN = 11,
    LIGHT_RED = 12,
    LIGHT_MAGENTA = 13,
    LIGHT_BROWN = 14,
    WHITE = 15,
}

impl ColorCode {
    pub const fn new(foreground: color, background: color) -> Self {
        Self((foreground as u8) | ((background as u8) << 4))
    }
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum vga_mode {
    STDOUT = 1,
    STDERR = 2, // at the moment, STDERR just makes output red lol
}

const VGA_INIT_COLOR: ColorCode = ColorCode::new(color::LIGHT_CYAN, color::BLACK);

const VGA_TABSTOP: usize = 8;

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
// const VGA_CELLS: usize = VGA_WIDTH * VGA_HEIGHT;
// const VGA_MAX_STRING_SIZE: usize = VGA_CELLS;


#[derive(Copy, Clone)]
#[repr(C, packed)]
struct VgaCell {
    ch: u8,
    color: u8,
}

#[repr(transparent)]
struct VgaBuffer {
    cells: [[VgaCell; VGA_WIDTH]; VGA_HEIGHT],
}

struct VgaCon {
    cx: usize,
    cy: usize,
    color: ColorCode,
    buffer: *mut VgaBuffer,
}

impl VgaCon {
    pub const fn new() -> Self {
        Self {
            cx: 0,
            cy: 0,
            color: VGA_INIT_COLOR,
            buffer: 0xb8000 as *mut VgaBuffer,
        }
    }
    pub fn cur_color(&self) -> ColorCode {
        self.color
    }
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

	/// # Safety
    /// this is called only when y >= VGA_HEIGHT
    /// undefined behavior if y < VGA_HEIGHT
    fn scroll(&mut self) {
        for y in 1..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
	            unsafe {
		            (*self.buffer).cells[y - 1][x] = (*self.buffer).cells[y][x];
	            }
            }
        }
        let empty: VgaCell = VgaCell {
            color: self.color.as_u8(),
            ch: b' ',
        };
        for x in 0..VGA_WIDTH {
	        unsafe {
		        (*self.buffer).cells[VGA_HEIGHT - 1][x] = empty;
	        }
        }
        self.cx = 0;
        self.cy -= 1;
    }
    fn inc_buffer(&mut self) {
        self.cx += 1;

        if self.cx >= VGA_WIDTH {
            self.scroll();
            self.cx = 0;
        }
    }
    pub fn putchar(&mut self, c: u8) {
        match c {
            b'\n' => self.newline(),
            b'\r' => self.carriage_ret(),
            b'\t' => self.tab(),
            0x20..=0x7e => self.write_cell(c),
            _ => {}
        }
    }
    fn write_cell(&mut self, c: u8) {
        unsafe {
            write_volatile(
	            &mut (*self.buffer).cells[self.cy][self.cx],
	            VgaCell {
                    ch: c,
                    color: self.color.as_u8(),
                },
            );
        }
        self.inc_buffer();
    }
    fn newline(&mut self) {
        self.cy += 1;
        if self.cy >= VGA_HEIGHT {
            self.scroll();
        }
        self.cx = 0;
    }
    fn carriage_ret(&mut self) {
        self.cx = 0;
    }
    fn tab(&mut self) {
        let distance = VGA_TABSTOP - (self.cx % VGA_TABSTOP);

        for _ in 0..distance {
            self.write_cell(b' ');
        }
    }
    pub fn clear(&mut self) {
        let empty: VgaCell = VgaCell {
            color: self.color.as_u8(),
            ch: b' ',
        };
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
	            unsafe {
		            (*self.buffer).cells[y][x] = empty;
	            }
            }
        }
    }
}

static VGA_CON: Global<VgaCon> = Global::new(VgaCon::new());

pub fn init() {
    unsafe {
        VGA_CON.get().clear();
    }
}

pub fn puts(s: &str) {
    unsafe {
        let con = VGA_CON.get();

        for c in s.chars() {
            con.putchar(c as u8);
        }
    }
}

pub fn set_color(fg: color, bg: color) {
    unsafe {
        VGA_CON.get().set_color(ColorCode::new(fg, bg));
    }
}

pub fn putchar(c: u8) {
    unsafe {
        VGA_CON.get().putchar(c);
    }
}

pub fn clear() {
    unsafe {
        VGA_CON.get().clear();
    }
}

pub fn get_color() -> ColorCode {
    unsafe { VGA_CON.get().cur_color() }
}
