use windows::Win32::{Foundation::HINSTANCE, System::SystemServices::DLL_PROCESS_ATTACH};

mod hook;
mod hotkey;
mod overlay;

/// The entrypoint called when the DLL is first loaded.
///
/// This minimal bootstrap starts one recurring task and installs one hook.
#[unsafe(no_mangle)]
extern "C" fn DllMain(hmodule: HINSTANCE, call_reason: u32) -> bool {
    if call_reason != DLL_PROCESS_ATTACH {
        return true;
    }

    // Safety: Hook installation targets one known game function.
    unsafe {
        hook::install();
    }

    let _ = overlay::start(hmodule);

    // This runs once per game frame using Sekiro's task manager.
    let _ = hotkey::start();

    true
}
