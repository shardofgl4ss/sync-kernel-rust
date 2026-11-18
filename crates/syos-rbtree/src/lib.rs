// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#![no_std]

/*
 * This RB tree implementation depends on
 * the callers getting memory for it.
 * Ie, from the kernel slab in my kernel.
 * I'll probably make a nicer API for the slab
 * in the future, and include an allocator 
 * as a parameter. Until then, just ptrs.
 */

use core::{
    alloc::Layout, ptr::{
        NonNull,
        addr_of_mut,
        null_mut,
        without_provenance_mut,
    },
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Red   = 1,
}

const COLOR_MASK: usize = 1;

// data is user
#[allow(dead_code)]
pub struct Node<T> {
    data: T,

    left:         *mut Node<T>,
    right:        *mut Node<T>,
    /// color is stored in first bit of parent.
    parent_color: *mut Node<T>,
}

pub struct Tree<T> {
    root:   *mut Node<T>,
}

pub enum RBErr {
    InvalidParameters,
}

macro_rules! ptr_deref {
    ($x:expr) => { (*$x.as_ptr()) };
}

/// # Safety
/// self should be a live Node<T> and no aliasing references should exist.
#[rustfmt::skip]
#[allow(clippy::missing_safety_doc)]
pub trait NodeExt: Sized {
    unsafe fn get_root (self) -> Option<Self>;
    unsafe fn get_color(self) -> Color;
    unsafe fn set_color(self, v: Color);

    unsafe fn set_left  (self, v: Option<Self>);
    unsafe fn set_right (self, v: Option<Self>);
    unsafe fn set_parent(self, v: Option<Self>);

    /// # Safety
    /// Only used for normally-guaranteed variants. Unchecked unwraps.
    unsafe fn left(self) -> Self;
    /// # Safety
    /// Only used for normally-guaranteed variants. Unchecked unwraps.
    unsafe fn right(self) -> Self;
    /// # Safety
    /// Only used for normally-guaranteed variants. Unchecked unwraps.
    unsafe fn parent(self) -> Self;
}


#[rustfmt::skip]
#[allow(clippy::missing_safety_doc)]
pub trait NodeOptExt: Sized {
    unsafe fn get_left  (self) -> Self;
    unsafe fn get_right (self) -> Self;
    unsafe fn get_parent(self) -> Self;
}

#[rustfmt::skip]
#[allow(clippy::missing_safety_doc)]
pub trait NodeGetExt: Sized {
    fn get_left  (self) -> Option<Self>;
    fn get_right (self) -> Option<Self>;
    fn get_parent(self) -> Option<Self>;
}

#[rustfmt::skip]
#[allow(clippy::missing_safety_doc)]
pub trait ColorCheck: Sized {
    unsafe fn is_black(&self) -> bool;
    unsafe fn is_red(&self)   -> bool;
}

#[allow(clippy::missing_safety_doc)]
pub trait NodeTraverse: Sized {
    unsafe fn next(self) -> Option<Self>;
    unsafe fn prev(self) -> Option<Self>;
}


type RBNode<T>    = NonNull<Node<T>>;
type RBNodeOpt<T> = Option<NonNull<Node<T>>>;


impl<T> NodeOptExt for RBNodeOpt<T> {
    unsafe fn get_left(self) -> Self {
        self?.get_left()
    }
    unsafe fn get_right(self) -> Self {
        self?.get_right()
    }
    unsafe fn get_parent(self) -> Self {
        self?.get_parent()
    }
}

impl<T> ColorCheck for RBNodeOpt<T> {
    unsafe fn is_black(&self) -> bool {
        unsafe {
            match *self {
            Some(s) => s.is_black(),
            None => true,
            }
        }
    }

    unsafe fn is_red(&self) -> bool {
        unsafe {
            match *self {
            Some(s) => s.is_red(),
            None => false,
            }
        }
    }
}

impl<T> NodeGetExt for RBNode<T> {
    fn get_left(self) -> Option<Self> {
        unsafe { NonNull::new(ptr_deref!(self).left) }
    }
    fn get_right(self) -> Option<Self> {
        unsafe { NonNull::new(ptr_deref!(self).right) }
    }
    fn get_parent(self) -> Option<Self> {
        unsafe { NonNull::new(ptr_deref!(self).parent_color.map_addr(|a| a & !COLOR_MASK)) }
    }
}

impl<T> ColorCheck for RBNode<T> {
    unsafe fn is_black(&self) -> bool {
        unsafe { self.get_color() == Color::Black }
    }

    unsafe fn is_red(&self) -> bool {
        unsafe { self.get_color() == Color::Red }
    }
}

impl<T> NodeExt for RBNode<T> {
    unsafe fn get_root(self) -> RBNodeOpt<T> {
        let mut root = Some(self);

        while let Some(r) = root && let Some(parent) = r.get_parent() {
            root = Some(parent);
        }

        root
    }

    unsafe fn get_color(self) -> Color {
        const RED: usize = Color::Red as usize;

        unsafe {
            let color = ptr_deref!(self).parent_color.addr() & COLOR_MASK;

            match color {
            RED => Color::Red,
            _ => Color::Black,
            }
        }
    }

    unsafe fn set_left(self, v: Option<Self>) {
        unsafe { ptr_deref!(self).left = v.map_or(null_mut(), NonNull::as_ptr) };
    }
    unsafe fn set_right(self, v: Option<Self>) {
        unsafe { ptr_deref!(self).right = v.map_or(null_mut(), NonNull::as_ptr) };
    }
    unsafe fn set_parent(self, v: Option<Self>) {
        unsafe {
            let color = ptr_deref!(self).parent_color.addr() & COLOR_MASK;
            let pc = addr_of_mut!(ptr_deref!(self).parent_color);

            match v {
            Some(rnode) => (*pc) = rnode.as_ptr().map_addr(|a| a | color),
            None        => (*pc) = without_provenance_mut::<Node<T>>(color),
            }
        }
    }
    unsafe fn set_color(self, v: Color) {
        unsafe {
            let pc = addr_of_mut!(ptr_deref!(self).parent_color);
            *pc = (*pc).map_addr(|a| (a & !COLOR_MASK) | (v as usize & COLOR_MASK));
        }
    }

    unsafe fn left(self) -> Self {
        unsafe {
            debug_assert!(self.get_left().is_some());
            self.get_left().unwrap_unchecked()
        }
    }

    unsafe fn right(self) -> Self {
        unsafe {
            debug_assert!(self.get_right().is_some());
            self.get_right().unwrap_unchecked()
        }
    }

    unsafe fn parent(self) -> Self {
        unsafe {
            debug_assert!(self.get_parent().is_some());
            self.get_parent().unwrap_unchecked()
        }
    }

}

impl<T> NodeTraverse for *mut Node<T> {
    unsafe fn next(self) -> Option<Self> {
        unsafe { Some(NonNull::new(self)?.next()?.as_ptr()) }
    }

    unsafe fn prev(self) -> Option<Self> {
        unsafe { Some(NonNull::new(self)?.prev()?.as_ptr()) }
    }
}

impl<T> NodeTraverse for RBNode<T> {
    unsafe fn next(self) -> Option<Self> {
        let mut node = self;
        if let Some(r) = node.get_right() {
            node = r;
            while let Some(x) = node.get_left() { node = x; };
            return Some(node);
        }

        let mut parent = node.get_parent();
        
        while let Some(p) = parent && let Some(pr) = p.get_right() && node == pr {
            node = p;
            parent = p.get_parent();
        }

        parent
    }

    unsafe fn prev(self) -> Option<Self> {
        let mut node = self;
        if let Some(l) = node.get_left() {
            node = l;
            while let Some(x) = node.get_right() { node = x; };
            return Some(node);
        }

        let mut parent = node.get_parent();
        
        while let Some(p) = parent && let Some(pr) = p.get_left() && node == pr {
            node = p;
            parent = p.get_parent();
        }

        parent
    }
}

impl<T> Tree<T> {
    pub const fn new() -> Self {
        Self {
            root: null_mut()
        }
    }

    fn rotate_left(&mut self, x: RBNode<T>) {
        unsafe {
            let Some(y) = x.get_right() else { return };
            x.set_right(Some(y));

            if let Some(y) = y.get_left() {
                y.set_parent(Some(x));
            }

            y.set_parent(x.get_parent());

            match x.get_parent() {
            None => self.root = y.as_ptr(),
            Some(p) => if p.get_left() == Some(x) {
                p.set_left(Some(y));
            } else {
                p.set_right(Some(y));
            },
            }

            y.set_left(Some(x));
            x.set_parent(Some(y));
        }
    }

    fn rotate_right(&mut self, x: RBNode<T>) {
        unsafe {
            let Some(y) = x.get_left() else { return };
            x.set_left(y.get_right());

            if let Some(y) = y.get_right() {
                y.set_parent(Some(x));
            }

            y.set_parent(x.get_parent());

            match x.get_parent() {
            None => self.root = y.as_ptr(),
            Some(p) => if p.get_right() == Some(x) {
                p.set_right(Some(y));
            } else {
                p.set_left(Some(y));
            },
            }

            y.set_right(Some(x));
            x.set_parent(Some(y));
        }
    }

    fn fix_insert(&mut self, mut x: RBNode<T>) {
        unsafe {
            while x.as_ptr() != self.root && x.parent().is_red() {
                let mut parent = x.parent();
                let mut grand = parent.parent();

                let unc_right: bool = parent == grand.left();
                let big_unc = if unc_right { grand.get_right() } else { grand.get_left() };

                if let Some(big_unc) = big_unc && big_unc.is_red() {
                    parent.set_color(Color::Black);
                    big_unc.set_color(Color::Black);

                    grand.set_color(Color::Red);
                    x = grand;

                    continue;
                }

                let inner = if unc_right { parent.right() } else { parent.left() };

                if x == inner {
                    x = parent;

                    if unc_right {
                        self.rotate_left(x);
                    } else {
                        self.rotate_right(x);
                    }

                    parent = x.parent();
                    grand = parent.parent();
                }

                parent.set_color(Color::Black);
                grand.set_color(Color::Red);

                if unc_right {
                    self.rotate_right(grand);
                } else {
                    self.rotate_left(grand);
                }
            }

            NonNull::new_unchecked(self.root).set_color(Color::Black);
        }
    }

    fn fix_delete(&mut self, mut x: RBNodeOpt<T>, mut parent: RBNodeOpt<T>) {
        unsafe {
            while let Some(p) = parent && x.is_black() {
                let rotate_left: bool = p.get_left() == x;

                let mut w = if rotate_left { p.right() } else { p.left() };
                let inner = |w: RBNode<T>| if rotate_left { w.get_left()  } else { w.get_right() };
                let outer = |w: RBNode<T>| if rotate_left { w.get_right() } else { w.get_left()  };

                if w.is_red() {
                    w.set_color(Color::Black);
                    p.set_color(Color::Red);

                    if rotate_left { self.rotate_left(p);  w = p.right(); } 
                    else           { self.rotate_right(p); w = p.left();  }
                }

                if outer(w).is_black() && inner(w).is_black() {
                    w.set_color(Color::Red);
                    x = Some(p);
                    parent = p.get_parent();
                } else {
                    if outer(w).is_black() {
                        inner(w).unwrap_unchecked().set_color(Color::Black);
                        w.set_color(Color::Red);

                        if rotate_left { self.rotate_right(w); w = p.right(); } 
                        else           { self.rotate_left(w);  w = p.left();  }
                    }

                    w.set_color(p.get_color());
                    p.set_color(Color::Black);

                    if let Some(o) = outer(w) { o.set_color(Color::Black); }

                    if rotate_left {
                        self.rotate_left(p);
                    } else {
                        self.rotate_right(p);
                    }

                    x = NonNull::new(self.root);
                    break;
                }
            }

            if let Some(n) = x {
                n.set_color(Color::Black);
            }
        }
    }

    unsafe fn parent_link(&mut self, node: RBNode<T>) -> *mut *mut Node<T> {
        unsafe {
            match node.get_parent() {
            Some(p) => if p.get_left() == Some(node) {
                addr_of_mut!(ptr_deref!(p).left)
            } else {
                addr_of_mut!(ptr_deref!(p).right)
            },
            None => addr_of_mut!(self.root),
            }
        }
    }

    /// # Safety
    /// This function expects valid pointers from the caller.
    /// Invalid pointers will result in:
    /// - a null pointer dereference,
    /// - RB tree corruption.
    /// - Computer explosions.
    ///
    /// Assumptions: 
    /// - link is the target slot in parent,
    /// - or parent is null_mut() if link is pointing to root.
    pub unsafe fn insert(&mut self, node: *mut Node<T>, parent: *mut Node<T>, link: *mut *mut Node<T>) -> Result<(), RBErr> {
        debug_assert!(parent.is_null() == (link == addr_of_mut!(self.root)));
        unsafe {
            let Some(n) = NonNull::new(node) else { return Err(RBErr::InvalidParameters) };

            addr_of_mut!((*node).left).write(null_mut());
            addr_of_mut!((*node).right).write(null_mut());
            n.set_parent(NonNull::new(parent));
            n.set_color(Color::Red);

            *link = node;

            self.fix_insert(n);
        }
        Ok(())
    }

    /// # Safety
    /// This function expects valid pointers from the caller.
    /// Invalid pointers will result in 
    /// - a null pointer dereference,
    /// - RB tree corruption.
    /// - Computer explosions.
    pub unsafe fn remove(&mut self, node: *mut Node<T>) -> RBNodeOpt<T> {
        let node = NonNull::new(node)?;

        unsafe {
            let parent = node.get_parent();

            match (node.get_left(), node.get_right()) {
            (None, None) => {
                *self.parent_link(node) = null_mut();
                if node.is_black() { self.fix_delete(None, parent); }
            },
            (Some(n), None) | (None, Some(n)) => {
                debug_assert!(node.is_black() && n.is_red());

                *self.parent_link(node) = n.as_ptr();
                n.set_parent(parent);
                n.set_color(Color::Black);
            },
            (Some(_), Some(_)) => {
                let mut x = node.right();

                while x.get_left().is_some() {
                    x = x.left();
                }

                self.remove(x.as_ptr());
                *self.parent_link(node) = x.as_ptr();

                x.set_left(node.get_left());
                if let Some(lx) = x.get_left() { lx.set_parent(Some(x)); }

                x.set_right(node.get_right());
                if let Some(rx) = x.get_right() { rx.set_parent(Some(x)); }

                x.set_color(node.get_color());
                x.set_parent(node.get_parent());
            },
            }

            let p = node.as_ptr();
            addr_of_mut!((*p).left).write(null_mut());
            addr_of_mut!((*p).right).write(null_mut());
            addr_of_mut!((*p).parent_color).write(null_mut());
        }

        Some(node)
    }

    pub fn first(&self) -> Option<*mut Node<T>> {
        let mut node = NonNull::new(self.root)?;
        while let Some(x) = node.get_left() { node = x; }
        Some(node.as_ptr())
    }

    pub fn last(&self) -> Option<*mut Node<T>> {
        let mut node = NonNull::new(self.root)?;
        while let Some(x) = node.get_right() { node = x; }
        Some(node.as_ptr())
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Node<T> {
    pub const fn layout() -> Layout { Layout::new::<Node<T>>() }

    /// # Safety
    /// The pointer should be valid and aligned to Node<T>'s layout,
    /// do not use on a live node in the tree.
    pub unsafe fn init(p: *mut Node<T>, data: T) -> RBNode<T> {
        unsafe {
            addr_of_mut!((*p).data).write(data);
            addr_of_mut!((*p).left).write(null_mut());
            addr_of_mut!((*p).right).write(null_mut());
            addr_of_mut!((*p).parent_color).write(null_mut());
            NonNull::new_unchecked(p)
        }
    }
}

