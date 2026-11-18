// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

#![no_std]

// hehe sync

use core::{
	cell::UnsafeCell,
	marker::PhantomData,
	mem::MaybeUninit,
	ops::{
		Deref,
		DerefMut,
	},
	sync::atomic::{
		AtomicBool,
		AtomicI8,
		Ordering::{
			Acquire,
			Relaxed,
			Release,
		},
	},
};

pub struct KMutex<T> {
	lock: AtomicBool,
	data: UnsafeCell<T>,
}

pub struct KGuard<'a, T> {
	mutex: &'a KMutex<T>,
	_p: PhantomData<*const ()>,
}

pub struct KOnce<T> {
	state: AtomicI8,
	data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Sync for KMutex<T> {}
unsafe impl<T: Sync> Sync for KGuard<'_, T> {}
unsafe impl<T: Send + Sync> Sync for KOnce<T> {}

impl<'a, T> Deref for KGuard<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		unsafe { &*self.mutex.data.get() }
	}
}

impl<'a, T> DerefMut for KGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *self.mutex.data.get() }
	}
}

impl<'a, T> Drop for KGuard<'a, T> {
	fn drop(&mut self) {
		self.mutex.lock.store(false, Release);
	}
}

impl<'a, T> KGuard<'a, T> {
	/// Just a method for an explicit drop().
	pub fn unlock(this: Self) {
		drop(this)
	}
}

impl<T> KMutex<T> {
	pub const fn new(data: T) -> Self {
		Self {
			lock: AtomicBool::new(false),
			data: UnsafeCell::new(data),
		}
	}

	pub fn try_lock(&self) -> Option<KGuard<'_, T>> {
		if self.lock.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
			Some(KGuard {
				mutex: self,
				_p: PhantomData,
			})
		} else {
			None
		}
	}

	/// # Warning
	/// This function spinloops. Deadlocks may occur from bad logic or IRQ.
	/// If you want to make sure you get the lock, make sure you don't deadlock.
	/// Otherwise, if you just want to try to get it once, use try_lock().
	pub fn lock(&self) -> KGuard<'_, T> {
		loop {
			if let Some(g) = self.try_lock() {
				return g;
			}
			core::hint::spin_loop();
		}
	}

	/// dont call this function unless you REALLY have to.
	/// Call unlock, or let the guard drop itself.
	/// # Safety
	/// No KGuard for this mutex should be alive,
	/// No other context should hold a lock.
	pub unsafe fn force_unlock(&self) {
		self.lock.store(false, Release);
	}
}


impl<T> KOnce<T> {
	const UNINIT: i8 = 0;
	const INITIALIZING: i8 = -1;
	const INIT: i8 = 1;

	pub const fn new() -> Self {
		Self {
			state: AtomicI8::new(Self::UNINIT),
			data: UnsafeCell::new(MaybeUninit::uninit()),
		}
	}

	pub fn get(&self) -> Option<&T> {
		if self.state.load(Acquire) == Self::INIT {
			unsafe { Some((*self.data.get()).assume_init_ref()) }
		} else {
			None
		}
	}

	pub fn set(&self, data: T) -> Result<(), T> {
		match self.state.compare_exchange(Self::UNINIT, Self::INITIALIZING, Acquire, Relaxed) {
		Ok(_) => {
			unsafe { (*self.data.get()).write(data) };
			self.state.store(Self::INIT, Release);

			Ok(())
		},
		Err(_) => Err(data),
		}
	}
}

impl<T> Default for KOnce<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Drop for KOnce<T> {
	fn drop(&mut self) {
		if *self.state.get_mut() == Self::INIT {
			unsafe {
				self.data.get_mut().assume_init_drop();
			}
		}
	}
}
