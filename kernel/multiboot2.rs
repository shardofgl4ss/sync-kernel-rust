#![allow(clippy::upper_case_acronyms)]

use core::mem::size_of;
use core::ffi::{c_char};

pub const MULTIBOOT_SEARCH: u32 = 32768;
pub const MULTIBOOT_HEADER_ALIGN: u32 = 8;
pub const MULTIBOOT2_HEADER_MAGIC: u32 = 3897708758;
pub const MULTIBOOT2_BOOTLOADER_MAGIC: u32 = 920085129;
pub const MULTIBOOT_MOD_ALIGN: u32 = 4096;
pub const MULTIBOOT_INFO_ALIGN: u32 = 8;
pub const MULTIBOOT_TAG_ALIGN: u32 = 8;

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TagBaseType {
    END              = 0,
    CMDLINE          = 1,
    BOOT_LOADER_NAME = 2,
    MODULE           = 3,
    BASIC_MEMINFO    = 4,
    BOOTDEV          = 5,
    MMAP             = 6,
    VBE              = 7,
    FRAMEBUFFER      = 8,
    ELF_SECTIONS     = 9,
    APM              = 10,
    EFI32            = 11,
    EFI64            = 12,
    SMBIOS           = 13,
    ACPI_OLD         = 14,
    ACPI_NEW         = 15,
    NETWORK          = 16,
    EFI_MMAP         = 17,
    EFI_BS           = 18,
    EFI32_IH         = 19,
    EFI64_IH         = 20,
    LOAD_BASE_ADDR   = 21,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    END                 = 0,
    INFORMATION_REQUEST = 1,
    ADDRESS             = 2,
    ENTRY_ADDRESS       = 3,
    CONSOLE_FLAGS       = 4,
    FRAMEBUFFER         = 5,
    MODULE_ALIGN        = 6,
    EFI_BS              = 7,
    ENTRY_ADDRESS_EFI32 = 8,
    ENTRY_ADDRESS_EFI64 = 9,
    RELOCATABLE         = 10,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureType {
    I386   = 0,
    MIPS32 = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LoadPreferenceType {
    NONE = 0,
    LOW  = 1,
    HIGH = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsoleFlagsType {
    CONSOLE_REQUIRED   = 1,
    EGA_TEXT_SUPPORTED = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    AVAILABLE        = 1,
    RESERVED         = 2,
    ACPI_RECLAIMABLE = 3,
    NVS              = 4,
    BADRAM           = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FramebufferType {
    INDEXED  = 0,
    RGB      = 1,
    EGA_TEXT = 2,
}


pub trait TailTag {
    type Elem;
    const PFX: usize;
}


pub unsafe fn tail_of<'a, X: TailTag>(tag: *const Tag) -> &'a [X::Elem] {
    unsafe {
        let size = (*tag).size as usize;
        let len = size.saturating_sub(X::PFX) / core::mem::size_of::<X::Elem>();
        let data = (tag as *const u8).add(X::PFX) as *const X::Elem;
        core::slice::from_raw_parts(data, len)
    }
}

pub fn trim_null(s: &[u8]) -> &[u8] {
    match s.split_last() { 
        Some((0, rest)) => rest,
        _ => s
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Header {
    pub magic: u32,
    pub arch: u32,
    pub length: u32,
    pub checksum: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HeaderTag {
    pub r#type: u16,
    pub flags: u16,
    pub size: u32,
}

#[repr(C)]
pub struct InfoRequestHdrTag {
    pub tag: HeaderTag,
}

impl TailTag for InfoRequestHdrTag {
    type Elem = u32;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct AddressHdrTag {
    pub tag: HeaderTag,
    pub header_addr: u32,
    pub load_addr: u32,
    pub load_end_addr: u32,
    pub bss_end_addr: u32,
}

#[repr(C)]
pub struct EntryHdrTag {
    pub tag: HeaderTag,
    pub entry_addr: u32,
}

#[repr(C)]
pub struct ConsoleFlagsHdrTag {
    pub tag: HeaderTag,
    pub flags: u32,
}

#[repr(C)]
pub struct FramebufferHdrTag {
    pub tag: HeaderTag,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C)]
pub struct ModuleAlignHdrTag {
    pub tag: HeaderTag,
}

#[repr(C)]
pub struct RelocatableHdrTag {
    pub tag: HeaderTag,
    pub min_addr: u32,
    pub max_addr: u32,
    pub align: u32,
    pub preference: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MmapEntryTag {
    pub addr: u64,
    pub len: u64,
    pub r#type: u32,
    pub zero: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Tag {
    pub r#type: u32,
    pub size: u32,
}

#[repr(C)]
pub struct StringTag {
    pub tag: Tag,
}

impl TailTag for StringTag {
    type Elem = c_char;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct ModuleTag {
    pub tag: Tag,
    pub mod_start: u32,
    pub mod_end: u32,
}

impl TailTag for ModuleTag {
    type Elem = c_char;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct BasicMeminfoTag {
    pub tag: Tag,
    pub mem_lower: u32,
    pub mem_upper: u32,
}

#[repr(C)]
pub struct BootdevTag {
    pub tag: Tag,
    pub biosdev: u32,
    pub slice: u32,
    pub part: u32,
}

#[repr(C)]
pub struct MmapTag {
    pub tag: Tag,
    pub entry_size: u32,
    pub entry_version: u32,
}

impl TailTag for MmapTag {
    type Elem = MmapEntryTag;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct VBEInfoBlock {
    pub external_specification: [u8; 512],
}

#[repr(C)]
pub struct VBEModeInfoBlock {
    pub external_specification: [u8; 256],
}

#[repr(C)]
pub struct VBETag {
    pub tag: Tag,

    pub mode: u16,
    pub interface_seg: u16,
    pub interface_off: u16,
    pub interface_len: u16,
    pub control_info: VBEInfoBlock,
    pub mode_info: VBEModeInfoBlock,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RgbInfo {
    pub red_pos: u8, pub red_size: u8,
    pub green_pos: u8, pub green_size: u8,
    pub blue_pos: u8, pub blue_size: u8,
}

#[repr(C)]
// palette entries follow at the tail
pub struct PaletteInfo {
    pub colors: u16,
}

#[repr(C)]
pub union FbColorInfo {
    pub rgb: RgbInfo,
    pub palette: core::mem::ManuallyDrop<PaletteInfo>,
}

#[repr(C)]
pub struct FramebufferTag {
    pub tag: Tag,

    pub addr: u64,
    pub pitch: u32,
    pub width: u32,
    pub height: u32,
    pub bpp: u8,
    pub r#type: u8,
    pub reserved: u16,
    pub colors: FbColorInfo,
}

#[repr(C)]
pub struct ELFSectionsTag {
    pub tag: Tag,
    pub num: u32,
    pub entsize: u32,
    pub shndx: u32,
}

impl TailTag for ELFSectionsTag {
    type Elem = c_char;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct APMTag {
    pub tag: Tag,
    pub version: u16,
    pub cseg: u16,
    pub offset: u32,
    pub cseg_16: u16,
    pub dseg: u16,
    pub flags: u16,
    pub cseg_len: u16,
    pub cseg_16_len: u16,
    pub dseg_len: u16,
}

#[repr(C)]
pub struct EFI32Tag {
    pub tag: Tag,
    pub pointer: u32,
}

#[repr(C)]
pub struct EFI64Tag {
    pub tag: Tag,
    pub pointer: u64,
}

#[repr(C)]
pub struct SMBIOSTag {
    pub tag: Tag,
    pub major: u8,
    pub minor: u8,
    pub reserved: [u8; 6],
}

impl TailTag for SMBIOSTag {
    type Elem = u8;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct OldACPITag {
    pub tag: Tag,
}

#[repr(C)]
pub struct NewACPITag {
    pub tag: Tag,

}

impl TailTag for NewACPITag {
    type Elem = u8;
    const PFX: usize = size_of::<Self>();
}
impl TailTag for OldACPITag {
    type Elem = u8;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct NetworkTag {
    pub tag: Tag,
}

impl TailTag for NetworkTag {
    type Elem = u8;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct EFIMmapTag {
    pub tag: Tag,
    pub descr_size: u32,
    pub descr_vers: u32,
}

impl TailTag for EFIMmapTag {
    type Elem = u8;
    const PFX: usize = size_of::<Self>();
}

#[repr(C)]
pub struct EFI32ihTag {
    pub tag: Tag,
    pub pointer: u32,
}

#[repr(C)]
pub struct EFI64ihTag {
    pub tag: Tag,
    pub pointer: u64,
}

#[repr(C)]
pub struct LoadBaseAddrTag {
    pub tag: Tag,
    pub load_base_addr: u32,
}

const _: () = assert!(size_of::<Tag>() == 8);
const _: () = assert!(size_of::<MmapEntryTag>() == 24);
const _: () = assert!(size_of::<MmapTag>() == 16);
const _: () = assert!(size_of::<BasicMeminfoTag>() == 16);
const _: () = assert!(size_of::<BootdevTag>() == 20);
const _: () = assert!(size_of::<APMTag>() == 28);
const _: () = assert!(size_of::<EFI32Tag>() == 12);
const _: () = assert!(size_of::<EFI64Tag>() == 16);
const _: () = assert!(size_of::<EFIMmapTag>() == 16);
const _: () = assert!(size_of::<VBETag>() == 8 + 8 + 512 + 256);
const _: () = assert!(size_of::<FramebufferTag>() == 40);

#[repr(C)]
pub struct InfoHeader {
    pub total: u32,
    pub reserved: u32,
}


pub fn find_tag(info: *const InfoHeader, r#type: TagBaseType) -> Option<*const Tag> {
    unsafe {
        let base = info as usize;
        let end = base + (*info).total as usize;
        let mut p = base + size_of::<InfoHeader>();

        while p < end {
            let tag = p as *const Tag;
            let t = (*tag).r#type;
            let size = (*tag).size as usize;

            if size < size_of::<Tag>() || size > end - p {
                break;
            }

            if t == r#type as u32 {
                return Some(tag);
            }

            if (*tag).r#type == TagType::END as u32 {
                break;
            }

            p += (size + 7) & !7;
        }
    };

    None
}

