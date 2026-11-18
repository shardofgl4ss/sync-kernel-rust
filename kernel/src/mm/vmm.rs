// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use syos_rbtree::{
    Tree as RBTree,
    Node as RBNode,
};
use syos_sync::{
    KOnce,
    KMutex,
    KGuard,
};

use core::ptr::{
    addr_of,
    null_mut,
    NonNull,
};

use crate::{
    include::types::*,
    mm::slab::SlabCache as KCache,
    pagetbl::PageFrame,
};

unsafe extern "C" {
    static KERNEL_VMMMAP: u8;
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct VMPtr<P>(*mut P);

impl<P> VMPtr<P> {
    pub const fn new(v: *mut P) -> Self {
        Self(v)
    }

    pub const INVALID: Self = Self(usize::MAX as *mut P);
    pub fn is_invalid(&self) -> bool { self.0.addr() == usize::MAX }
    pub fn as_usize(&self) -> usize { self.0 as usize }
}

pub enum VMMError {
    CacheFailure,
    AtomicInitFailure,
}

type Result<T> = core::result::Result<T, VMMError>;
type VMNode = NonNull<RBNode<VMA>>;
type VMTree = RBTree<VMA>;

#[repr(u32)]
pub enum VMFlags {
	READ          = 1 << 0,
	WRITE         = 1 << 1,
	EXEC          = 1 << 2,
	USER          = 1 << 3,
	ANON          = 1 << 4,
	IO            = 1 << 5,
	NOCACHE       = 1 << 6,
	WRITE_THROUGH = 1 << 7,
	SHARED        = 1 << 8,
	GROWSDOWN     = 1 << 9,
}

pub struct VMA {
    vm_start: VirtAddr,
    vm_end:   VirtAddr,
    pages:    *mut *mut PageFrame,
    vm_flags: VMFlags,
}

unsafe impl Send for VMA {}

pub struct VMARegion {
    r_start: VirtAddr,
    r_end:   VirtAddr,
    r_tree:  KMutex<VMTree>,
}


const KERNEL_VMMAP_LEN: usize = (1824_usize * 1024_usize * 1024_usize * 1024_usize) * 64_usize;
const VMA_IDENTIFIER: &str = "kernelvma_struct";

static VMMAP_CACHE: KMutex<KCache> = KMutex::new(KCache::new(VMA_IDENTIFIER));
static VMMAP_REGION: KOnce<VMARegion> = KOnce::new();


fn vma_alloc() -> *mut VMNode {
    let mut c = VMMAP_CACHE.lock();
    let a = c.alloc();
    if let Some(p) = a {
        p.cast::<VMNode>()
    } else {
        null_mut()
    }
}
fn vma_free(n: VMNode)  {
    let mut c = VMMAP_CACHE.lock();
    c.free(n.as_ptr().cast());
}
fn vma_new(n: VMNode, data: VMA) {
    unsafe { RBNode::init(n.as_ptr(), data) };
}


pub fn init() -> Result<()> {
    let mut cache = VMMAP_CACHE.lock();
    let layout = RBNode::<VMA>::layout();
    cache.create(VMA_IDENTIFIER, layout.size(), layout.align(), 0).map_err(|_| VMMError::CacheFailure)?;

    let vmmap_addr = addr_of!(KERNEL_VMMMAP) as usize;
    VMMAP_REGION.set(VMARegion {
        r_start: VirtAddr(vmmap_addr),
        r_end: VirtAddr(vmmap_addr + KERNEL_VMMAP_LEN),
        r_tree: KMutex::new(RBTree::new())
    }).map_err(|_| VMMError::AtomicInitFailure)?;

    Ok(())
}

