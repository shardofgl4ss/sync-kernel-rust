// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use crate::{
    mm::buddy, 
    pagetbl::PAGESIZE, 
    panic::kernel_panic,
};

use core::{
    hint::cold_path, 

    ptr::{
        NonNull, 
        null_mut,
    },
};

const DEFAULT_MAX_EMPTY:    usize = 4;
const MAX_SLAB_ORDER:       usize = buddy::Order::ORDER_64K as usize;
const MIN_OBJS_PER_SLAB:    usize = 8;

pub enum SlabError {
    OutOfMemory,
    InternalErr,
    ObjsTooLarge,
}

pub struct SlabCache {
    identifier: &'static str,

    order:      buddy::Order,

    obj_size:   usize,
    alignment:  usize,
    pages:      usize,

    full:       Option<NonNull<Slab>>,
    partial:    Option<NonNull<Slab>>,
    empty:      Option<NonNull<Slab>>,

    objs_per_slab:  usize,
    waste:          usize,
    reserved:       usize,
    metadata:       usize,

    empty_slabs:        u8,
    max_empty_slabs:    u8,
}

struct Slab {
    cache:  NonNull<SlabCache>,

    next:   Option<NonNull<Slab>>,
    prev:   Option<NonNull<Slab>>,

    pool:       *mut u8,
    freelist:   *mut u8,

    free_count: usize,
}



impl SlabCache {
    pub fn free(&mut self, obj: *mut u8) {
        let pv = obj as usize;
        let base = pv & !(self.order.val() - 1);
        let slab = unsafe { &mut *(base as *mut Slab) };

        let was_full = slab.free_count == 0;

        unsafe { *(obj.cast::<*mut u8>()) = slab.freelist; }
        slab.freelist = obj;
        slab.free_count += 1;

        let is_empty = slab.free_count == self.objs_per_slab;

        unsafe {
            if was_full {
                node_relink(&mut self.full, &mut self.partial, NonNull::from_mut(slab));
            } else if is_empty {
                node_relink(&mut self.partial, &mut self.empty, NonNull::from_mut(slab));
                self.empty_slabs += 1;

                if self.empty_slabs > self.max_empty_slabs {
                    let x = self.empty.unwrap_unchecked();
                    node_remove(NonNull::from_mut(self), &mut self.empty, x);
                    self.max_empty_slabs -= 1;
                }
            }
        }
    }

    pub fn alloc(&mut self) -> Option<*mut u8> {
        const AllocArray: [fn(&mut SlabCache) -> Option<*mut u8>; 3] = [ 
            SlabCache::alloc_from_partial, 
            SlabCache::alloc_from_empty, 
            SlabCache::alloc_from_new 
        ];

        let mut obj = None;

        for func in AllocArray {
            obj = func(self);
            if obj.is_some() { break; }
        };

        obj
    }

    pub fn create(&mut self, identifier: &'static str, objsize: usize, alignment: usize, flags: usize) -> Result<(), SlabError> {
        const HDR_SIZE: usize = size_of::<Slab>();

        let mut s_order: usize = 0;
        let mut s_reserved: usize = 0;
        let mut s_count: usize = 0;
        let mut s_waste = usize::MAX;

        for i in 0..MAX_SLAB_ORDER {
            let size = (1_usize << i) * PAGESIZE;
            let pad = (alignment - (HDR_SIZE % alignment)) % alignment;
            let reserved = HDR_SIZE + pad;
            
            if size <= reserved {
                cold_path();
                continue;
            }

            let rem = size - reserved;
            let objs = rem / objsize;

            if objs < MIN_OBJS_PER_SLAB && i < MAX_SLAB_ORDER { continue };
            if objs == 0 { continue };

            let waste = size - (reserved + (objs * objsize));
            
            if waste < s_waste {
                s_order = i;
                s_count = objs;
                s_waste = waste;
                s_reserved = reserved;
            }
        }

        if s_waste == usize::MAX || s_count == 0 {
            cold_path();
            return Err(SlabError::ObjsTooLarge)
        };

        let Ok(ord) = buddy::Order::try_from(s_order) else { return Err(SlabError::InternalErr) };

        self.identifier = identifier;
        self.pages = ord.val() >> 12;
        self.alignment = alignment;
        self.obj_size = objsize;
        self.metadata = flags;

        self.order = ord;
        self.waste = s_waste;
        self.reserved = s_reserved;
        self.objs_per_slab = s_count;
        self.max_empty_slabs = DEFAULT_MAX_EMPTY as u8;

        self.new_slab()?;
        self.empty_slabs += 1;

        Ok(())
    }

    pub fn destroy(&mut self) {
        for head in [&mut self.full, &mut self.partial, &mut self.empty] {
            while let Some(s) = head.take() {
                *head = unsafe { (*s.as_ptr()).next };
                if (buddy::buddy_free(s.as_ptr() as *mut u8, self.order)).is_err() {
                    cold_path();
                    kernel_panic("slabcache corruption!");
                }
            }
        }

        self.obj_size = usize::MAX;
        self.pages = 0;
        self.order = buddy::Order::ORDER_INVALID;
    }


    fn new_slab(&mut self) -> Result<NonNull<Slab>, SlabError> {
        // TODO implement slab.prev
        let slab = buddy::buddy_alloc(self.order) as *mut Slab;

        if slab.is_null() { return Err(SlabError::OutOfMemory) };
        let Some(pslab) = NonNull::new(slab) else { return Err(SlabError::InternalErr) };
        unsafe { slab.write_bytes(0, self.order.val()) };

        let mut pe: *mut Option<NonNull<Slab>> = &mut self.empty;
        let mut pe_prev = null_mut();
        
        unsafe {
            while let Some(cur) = *pe {
                pe_prev = pe;
                pe = &mut (*cur.as_ptr()).next;
            }

            (*slab).next = None;
            *pe = Some(NonNull::new_unchecked(slab));
        }

        let r_slab = unsafe { &mut *slab };

        r_slab.prev = unsafe { if !pe_prev.is_null() { *pe_prev } else { None } };
        r_slab.cache = NonNull::from_mut(self);
        r_slab.pool = unsafe { (slab as *mut u8).byte_add(self.reserved) };
        r_slab.freelist = r_slab.pool;
        r_slab.free_count = self.objs_per_slab;

        let mut curr = r_slab.pool;
        let mut next;

        unsafe { 
            for _ in 0..self.objs_per_slab - 1 {
                next = curr.add(self.obj_size);
                curr.cast::<*mut u8>().write(next);
                curr = next;
            };

            curr.cast::<*mut u8>().write(null_mut());
        }

        Ok(pslab)
    }


    fn alloc_from_partial(&mut self) -> Option<*mut u8> {
        let Some(mut slab) = self.partial else { return None };
        let rslab = unsafe { slab.as_mut() };

        rslab.free_count -= 1;

        if rslab.free_count == 0 {
            unsafe { node_relink(&mut self.partial, &mut self.full, slab); }
        };

        let retalloc = rslab.freelist.cast::<*mut u8>();
        rslab.freelist = unsafe { *retalloc };

        Some(retalloc as *mut u8)
    }

    fn alloc_from_empty(&mut self) -> Option<*mut u8> {
        let Some(mut slab) = self.empty else { return None };
        let rslab = unsafe { slab.as_mut() };

        unsafe { node_relink(&mut self.empty, &mut self.partial, slab); }
        self.empty_slabs -= 1;

        let retalloc = rslab.freelist.cast::<*mut u8>();
        rslab.freelist = unsafe { *retalloc };
        rslab.free_count -= 1;
        
        Some(retalloc as *mut u8)
    }

    fn alloc_from_new(&mut self) -> Option<*mut u8> {
        self.new_slab().ok()?;
        self.alloc_from_empty()
    }
}


unsafe fn node_unlink(head: &mut Option<NonNull<Slab>>, target: NonNull<Slab>) {
    let t = target.as_ptr();

    let next = unsafe { (*t).next };
    let prev = unsafe { (*t).prev };
    
    match prev {
    Some(p) => unsafe { (*p.as_ptr()).next = next },
    None => *head = next,
    }

    if let Some(p) = next { 
        unsafe { (*p.as_ptr()).prev = prev };
    }

    unsafe {
        (*t).next = None;
        (*t).prev = None;
    }
}


unsafe fn node_remove(cache: NonNull<SlabCache>, from: &mut Option<NonNull<Slab>>, target: NonNull<Slab>) {
    unsafe { 
        node_unlink(from, target);
        buddy::buddy_free(target.as_ptr().cast(), (*cache.as_ptr()).order);
    };
}

unsafe fn node_relink(from: &mut Option<NonNull<Slab>>, to: &mut Option<NonNull<Slab>>, target: NonNull<Slab>) {
    unsafe {
        node_unlink(from, target);

        match *to {
        None => *to = Some(target),
        Some(mut cur) => {
            while let Some(p) = (*cur.as_ptr()).next {
                cur = p;
            }

            (*cur.as_ptr()).next = Some(target);
            (*target.as_ptr()).prev = Some(cur);
        }};
    }
}

