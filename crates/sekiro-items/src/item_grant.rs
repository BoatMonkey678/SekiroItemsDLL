use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use anyhow::{Result, anyhow};
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};
use sekiro::sprj::{
    ItemBufferEntry, ItemCategory, ItemId, MapItemMan, SprjTaskGroupIndex, SprjTaskImp,
};

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
    // Another's Memory: Shura - grant Protector items when granting Good 9800 (Another's Memory Shura)
    if request.item_id == 9800 {
        item_man.grant_item(ItemBufferEntry::new(item_id, request.count));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 300000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 301000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 331000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 302000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 303000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 304000)?, 1));
    }
    // Another's Memory: Ashina
    else if request.item_id == 9810 {
        item_man.grant_item(ItemBufferEntry::new(item_id, request.count));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 400000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 401000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 431000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 402000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 403000)?, 1));
    }
    // Another's Memory: Tengu
    else if request.item_id == 9820 {
        item_man.grant_item(ItemBufferEntry::new(item_id, request.count));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 500000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 501000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 531000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 502000)?, 1));
        item_man.grant_item(ItemBufferEntry::new(ItemId::new(ItemCategory::Protector, 503000)?, 1));
    }
    else
    {
        item_man.grant_item(ItemBufferEntry::new(item_id, request.count));
    }
    Ok(())
}

fn lock_pending_grants() -> MutexGuard<'static, Vec<GrantRequest>> {
    match PENDING_GRANTS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
