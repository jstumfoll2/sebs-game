//! Ask Windows to run the game at full speed.
//!
//! On battery (and in the "Best power efficiency" power mode) Windows puts apps into
//! "efficiency mode": slow CPU cores, slow clocks. That's great for a web browser but makes
//! the game's animations stutter. Games can opt out, for their own process only.

/// Opt this process out of Windows' efficiency-mode throttling.
#[cfg(windows)]
pub fn full_speed() {
    // FFI, like in input.rs: these live in Windows' kernel32.dll.
    #[repr(C)]
    struct ThrottlingState {
        version: u32,
        control_mask: u32,
        state_mask: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn SetProcessInformation(process: isize, class: i32, info: *const ThrottlingState, size: u32) -> i32;
    }
    const PROCESS_POWER_THROTTLING: i32 = 4;
    // Take control of "execution speed" and "timer resolution" throttling, and turn both off.
    let state = ThrottlingState { version: 1, control_mask: 0x1 | 0x4, state_mask: 0 };
    let size = std::mem::size_of::<ThrottlingState>() as u32;
    // SAFETY: `state` is a valid struct of the size we pass, and lives through the call.
    let ok = unsafe { SetProcessInformation(GetCurrentProcess(), PROCESS_POWER_THROTTLING, &state, size) };
    if ok == 0 {
        crate::log::line("couldn't turn off Windows efficiency mode");
    }
}

#[cfg(not(windows))]
pub fn full_speed() {}
