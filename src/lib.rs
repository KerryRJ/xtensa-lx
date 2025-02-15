#![no_std]
#![feature(asm)]

pub mod interrupt;
pub mod mutex;
pub mod timer;

#[macro_use]
mod macros;

/// Move the vector base
#[inline]
pub unsafe fn set_vecbase(base: *const u32) {
    asm!("wsr.vecbase {0}", in(reg) base, options(nostack));
}

/// Get the core stack pointer
#[inline(always)]
pub fn get_stack_pointer() -> *const u32 {
    let x: *const u32;
    unsafe { asm!("mov {0}, sp", out(reg) x, options(nostack)) };
    x
}

/// Set the core stack pointer
///
/// *This is highly unsafe!*
/// It should be used with care at e.g. program start or when building a task scheduler
///
/// `stack` pointer to the non-inclusive end of the stack (must be 16-byte aligned)
#[inline(always)]
pub unsafe fn set_stack_pointer(stack: *mut u32) {
    // FIXME: this function relies on it getting inlined - if it doesn't inline it will try and return from this function using the adress in `a0` which has just been trashed...
    // According to https://nnethercote.github.io/perf-book/inlining.html:
    // "Inline attributes do not guarantee that a function is inlined or not inlined, but in practice, #[inline(always)] will cause inlining in all but the most exceptional cases."
    // Is this good enough? Should we rewrite these as a macro to guarentee inlining?
    
    
    // NOTE: modification of the `sp` & `a0` is not typically allowed inside inline asm!,
    // but because we *need* to modify it we can do so by ommiting it from the clobber
    asm!(
        "movi a0, 0", // trash return register
        "mov sp, {0}", // move stack pointer
        in(reg) stack, options(nostack)
    );
}

/// Get the core current program counter
#[inline(always)]
pub fn get_program_counter() -> *const u32 {
    let x: *const u32;
    unsafe {
        asm!("
            mov {1}, {2}
            call0 1f
            .align 4
            1: 
            mov {0}, {2}
            mov {2}, {1}
            ", out(reg) x, out(reg) _, out(reg) _, options(nostack))
    };
    x
}

/// Get the id of the current core
#[inline]
pub fn get_processor_id() -> u32 {
    let mut x: u32;
    unsafe { asm!("rsr.prid {0}", out(reg) x, options(nostack)) };
    x
}

const XDM_OCD_DCR_SET: u32 = 0x10200C;
const DCR_ENABLEOCD: u32 = 0x01;

/// Returns true if a debugger is attached
#[inline]
pub fn is_debugger_attached() -> bool {
    let mut x: u32;
    unsafe { asm!("rer {0}, {1}", out(reg) x, in(reg) XDM_OCD_DCR_SET, options(nostack)) };
    (x & DCR_ENABLEOCD) != 0
}

/// Insert debug breakpoint
#[inline(always)]
pub fn debug_break() {
    unsafe { asm!("break 1, 15", options(nostack)) };
}
