// Copyright 2025 The safe-mmio Authors.
// This project is dual-licensed under Apache 2.0 and MIT terms.
// See LICENSE-APACHE and LICENSE-MIT for details.

use crate::backend::mmio_ops::MmioOps;
use crate::{SharedMmioPointer, UniqueMmioPointer};
use zerocopy::{FromBytes, Immutable, IntoBytes};

macro_rules! asm_read {
    ($ins:literal, $reg:literal, $src:expr) => {{
        let value;
        // SAFETY: Caller guarantees src is valid and aligned.
        unsafe {
            core::arch::asm!(
                concat!($ins, " {value:", $reg, "}, [{ptr}]"),
                value = out(reg) value,
                ptr = in(reg) $src,
            );
        }
        value
    }};
}

macro_rules! asm_write {
    ($ins:literal, $reg:literal, $dst:expr, $value:expr) => {
        // SAFETY: Caller guarantees dst is valid and aligned.
        unsafe {
            core::arch::asm!(
                concat!($ins, " {value:", $reg, "}, [{ptr}]"),
                value = in(reg) $value,
                ptr = in(reg) $dst,
            );
        }
    };
}

/// MmioOps backend using aarch64 inline assembly for MMIO access.
struct Ops;

impl MmioOps for Ops {
    unsafe fn read_u8(src: *const u8) -> u8 {
        asm_read!("ldrb", "w", src)
    }

    unsafe fn read_u16(src: *const u16) -> u16 {
        asm_read!("ldrh", "w", src)
    }

    unsafe fn read_u32(src: *const u32) -> u32 {
        asm_read!("ldr", "w", src)
    }

    unsafe fn read_u64(src: *const u64) -> u64 {
        asm_read!("ldr", "x", src)
    }

    unsafe fn write_u8(dst: *mut u8, value: u8) {
        asm_write!("strb", "w", dst, value);
    }

    unsafe fn write_u16(dst: *mut u16, value: u16) {
        asm_write!("strh", "w", dst, value);
    }

    unsafe fn write_u32(dst: *mut u32, value: u32) {
        asm_write!("str", "w", dst, value);
    }

    unsafe fn write_u64(dst: *mut u64, value: u64) {
        asm_write!("str", "x", dst, value);
    }
}

impl<T: FromBytes + IntoBytes> UniqueMmioPointer<'_, T> {
    /// Performs an MMIO read and returns the value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, reading chunks as large as possible.
    ///
    /// Note that this takes `&mut self` rather than `&self` because an MMIO read may cause
    /// side-effects that change the state of the device.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO read from.
    pub unsafe fn read_unsafe(&mut self) -> T {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe { Ops::read(self.regs) }
    }
}

impl<T: Immutable + IntoBytes> UniqueMmioPointer<'_, T> {
    /// Performs an MMIO write of the given value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, writing chunks as large as possible.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO write to.
    pub unsafe fn write_unsafe(&mut self, value: T) {
        // SAFETY: self.regs is always a valid and unique pointer to MMIO address space.
        unsafe {
            Ops::write(self.regs, value);
        }
    }
}

impl<T: FromBytes + IntoBytes> SharedMmioPointer<'_, T> {
    /// Performs an MMIO read and returns the value.
    ///
    /// If `T` is exactly 1, 2, 4 or 8 bytes long then this will be a single operation. Otherwise
    /// it will be split into several, reading chunks as large as possible.
    ///
    /// # Safety
    ///
    /// This field must be safe to perform an MMIO read from, and doing so must not cause any
    /// side-effects.
    pub unsafe fn read_unsafe(&self) -> T {
        // SAFETY: self.regs is always a valid pointer to MMIO address space.
        unsafe { Ops::read(self.regs) }
    }
}
