use ilhook::x64::{CallbackOption, HookFlags, Registers, hook_closure_jmp_back};
use sekiro::sprj::{ItemBuffer, MAP_ITEM_MAN_GRANT_ITEM_VA};

/// Install one proof-of-hook callback on item grants.
///
/// Safety: This modifies process code and must only target the expected address.
pub unsafe fn install() {
    let callback = |reg: *mut Registers| {
        // Keep this callback lightweight. You can inspect or mutate items here.
        let _items = unsafe { &mut *((*reg).rdx as *mut ItemBuffer) };
    };

    std::mem::forget(
        unsafe {
            hook_closure_jmp_back(
                *MAP_ITEM_MAN_GRANT_ITEM_VA as usize,
                callback,
                CallbackOption::None,
                HookFlags::empty(),
            )
        }
        .expect("Hooking MapItemMan::GrantItem failed"),
    );
}
