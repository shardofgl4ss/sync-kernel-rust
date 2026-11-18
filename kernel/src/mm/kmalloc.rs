// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Sync Shard

use super::*;

fn kmalloc_init() {}

pub fn init() {
	vmm::init();
	vmalloc::init();
	kmalloc_init();
}
