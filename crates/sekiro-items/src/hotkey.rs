use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use anyhow::{Result, anyhow};
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};
use sekiro::sprj::{
    ItemBufferEntry, ItemCategory, ItemId, MapItemMan, SprjTaskGroupIndex, SprjTaskImp,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F6};

static F6_WAS_DOWN: AtomicBool = AtomicBool::new(false);
static PENDING_GRANTS: Mutex<Vec<GrantRequest>> = Mutex::new(Vec::new());

#[derive(Clone, Copy, Debug)]
pub struct GrantRequest {
    pub item_id: u32,
    pub category: u8,
    pub count: u32,
}

impl GrantRequest {
    pub const fn new(item_id: u32, category: u8, count: u32) -> Self {
        Self {
            item_id,
            category,
            count,
        }
    }
}

pub fn start() -> Result<()> {
    std::thread::spawn(|| {
        if wait_for_game().is_ok() {
            let _ = install_frame_task();
        }
    });
    Ok(())
}

fn wait_for_game() -> Result<()> {
    sekiro::util::system::wait_for_system_init(&Program::current(), Duration::MAX)?;
    Ok(())
}

fn install_frame_task() -> Result<()> {
    // Safety: The task closure is frame-bound and only touches game APIs.
    unsafe { SprjTaskImp::instance() }?
        .run_recurring(move |_: &'_ usize| on_frame(), SprjTaskGroupIndex::FrameBegin);
    Ok(())
}

fn on_frame() {
    process_pending_grants();

    // Rising-edge detection so holding F6 does not spam grants.
    let f6_down = unsafe { GetAsyncKeyState(VK_F6.0 as i32) } < 0;
    let was_down = F6_WAS_DOWN.swap(f6_down, Ordering::Relaxed);
    if f6_down && !was_down {
        let _ = grant_item_request(debug_grant_request());
    }
}

pub fn queue_grant_request(request: GrantRequest) -> Result<()> {
    validate_grant_request(request)?;
    lock_pending_grants().push(request);
    Ok(())
}

pub fn queue_grant_requests(requests: &[GrantRequest]) -> Result<()> {
    for request in requests {
        queue_grant_request(*request)?;
    }

    Ok(())
}

pub fn validate_grant_request(request: GrantRequest) -> Result<()> {
    if request.count == 0 {
        return Err(anyhow!("Count must be greater than 0"));
    }

    let category = ItemCategory::try_from(request.category)
        .map_err(|_| anyhow!("Category must be 0 (Weapon), 1 (Protector), 2 (Accessory), or 4 (Goods)"))?;
    ItemId::new(category, request.item_id).map_err(|err| anyhow!(err))?;

    Ok(())
}

fn process_pending_grants() {
    let requests = {
        let mut pending = lock_pending_grants();
        std::mem::take(&mut *pending)
    };

    for request in requests {
        let _ = grant_item_request(request);
    }
}

fn grant_item_request(request: GrantRequest) -> Result<()> {
    validate_grant_request(request)?;

    let item_man = unsafe { MapItemMan::instance() }
        .map_err(|err| anyhow!("MapItemMan is not available: {err}"))?;
    let category = ItemCategory::try_from(request.category).map_err(|err| anyhow!(err))?;
    let item_id = ItemId::new(category, request.item_id).map_err(|err| anyhow!(err))?;

    item_man.grant_item(ItemBufferEntry::new(item_id, request.count));
    Ok(())
}

fn debug_grant_request() -> GrantRequest {
    // TODO: Replace this with a real Sekiro item ID once you pick one.
    GrantRequest::new(660600, 4, 1)
}

fn lock_pending_grants() -> MutexGuard<'static, Vec<GrantRequest>> {
    match PENDING_GRANTS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
