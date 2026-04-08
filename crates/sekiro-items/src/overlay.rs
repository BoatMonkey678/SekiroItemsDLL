use std::{sync::LazyLock, time::Duration};

use anyhow::{Result, anyhow};
use fromsoftware_shared::Program;
use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::imgui::{Condition, Context, Io, Ui};
use hudhook::{Hudhook, ImguiRenderLoop, MessageFilter, RenderContext};
use serde::Deserialize;
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_INSERT};

use crate::item_grant::{self, GrantRequest};

static ITEM_CATALOG: LazyLock<std::result::Result<Vec<ItemCatalogEntry>, String>> =
    LazyLock::new(load_item_catalog);

#[derive(Debug, Deserialize)]
struct ItemCatalogEntry {
    category: u8,
    #[serde(default)]
    id: Option<u32>,
    #[serde(default)]
    ids: Vec<u32>,
    in_game_name: String,
}

impl ItemCatalogEntry {
    fn item_ids(&self) -> Vec<u32> {
        let mut item_ids = Vec::with_capacity(self.ids.len() + usize::from(self.id.is_some()));
        if let Some(item_id) = self.id {
            item_ids.push(item_id);
        }
        item_ids.extend(self.ids.iter().copied());
        item_ids
    }

    fn grant_requests(&self, count: u32) -> Result<Vec<GrantRequest>> {
        let item_ids = self.item_ids();
        if item_ids.is_empty() {
            return Err(anyhow!(
                "Item '{}' must define either 'id' or 'ids' in items.json",
                self.in_game_name
            ));
        }

        let requests = item_ids
            .into_iter()
            .map(|item_id| GrantRequest::new(item_id, self.category, count))
            .collect::<Vec<_>>();

        for request in &requests {
            item_grant::validate_grant_request(*request)?;
        }

        Ok(requests)
    }
}

fn load_item_catalog() -> std::result::Result<Vec<ItemCatalogEntry>, String> {
    let items: Vec<ItemCatalogEntry> = serde_json::from_str(include_str!("../items.json"))
        .map_err(|err| format!("Could not parse items.json: {err}"))?;

    if items.is_empty() {
        return Err(String::from("items.json does not contain any items"));
    }

    for item in &items {
        item.grant_requests(1).map_err(|err| {
            format!(
                "Invalid item catalog entry '{}' (category {}): {err}",
                item.in_game_name, item.category
            )
        })?;
    }

    Ok(items)
}

fn item_catalog() -> Result<&'static [ItemCatalogEntry]> {
    match &*ITEM_CATALOG {
        Ok(items) => Ok(items.as_slice()),
        Err(err) => Err(anyhow!(err.clone())),
    }
}

pub fn start(hmodule: HINSTANCE) -> Result<()> {
    std::thread::spawn(move || {
        if wait_for_game().is_ok() {
            let _ = install_overlay(hmodule);
        }
    });

    Ok(())
}

fn wait_for_game() -> Result<()> {
    sekiro::util::system::wait_for_system_init(&Program::current(), Duration::MAX)?;
    Ok(())
}

fn install_overlay(hmodule: HINSTANCE) -> Result<()> {
    Hudhook::builder()
        .with::<ImguiDx11Hooks>(ItemGrantOverlay::default())
        .with_hmodule(hmodule)
        .build()
        .apply()
        .map_err(|status| anyhow!("Could not apply hudhook overlay: {status:?}"))?;
    Ok(())
}

struct ItemGrantOverlay {
    visible: bool,
    insert_was_down: bool,
    focus_item_filter_next_frame: bool,
    selected_item_index: usize,
    item_filter: String,
    count: String,
    status: String,
}

impl Default for ItemGrantOverlay {
    fn default() -> Self {
        Self {
            visible: true,
            insert_was_down: false,
            focus_item_filter_next_frame: true,
            selected_item_index: 0,
            item_filter: String::new(),
            count: String::from("1"),
            status: String::new(),
        }
    }
}

impl ItemGrantOverlay {
    fn poll_toggle(&mut self) {
        let insert_down = unsafe { GetAsyncKeyState(VK_INSERT.0 as i32) } < 0;
        if insert_down && !self.insert_was_down {
            self.visible = !self.visible;
            if self.visible {
                self.focus_item_filter_next_frame = true;
            }
        }
        self.insert_was_down = insert_down;
    }

    fn item_matches_filter(&self, item: &ItemCatalogEntry) -> bool {
        let filter = self.item_filter.trim();
        if filter.is_empty() {
            return true;
        }

        item.in_game_name
            .to_ascii_lowercase()
            .contains(&filter.to_ascii_lowercase())
    }

    fn filtered_item_indices(&self, items: &[ItemCatalogEntry]) -> Vec<usize> {
        items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| self.item_matches_filter(item).then_some(index))
            .collect()
    }

    fn parse_requests(&self) -> Result<Vec<GrantRequest>> {
        let items = item_catalog()?;
        if !items.iter().any(|item| self.item_matches_filter(item)) {
            return Err(anyhow!("No items match the current search"));
        }

        let count = self
            .count
            .trim()
            .parse::<u32>()
            .map_err(|_| anyhow!("Count must be an unsigned integer"))?;
        let item = items
            .get(self.selected_item_index)
            .ok_or_else(|| anyhow!("Selected item is invalid"))?;
        if !self.item_matches_filter(item) {
            return Err(anyhow!("Selected item does not match the current search"));
        }

        item.grant_requests(count)
    }

    fn render_window(&mut self, ui: &Ui) {
        let items = item_catalog().ok();
        let catalog_error = item_catalog().err().map(|err| err.to_string());
        if let Some(items) = items
            && self.selected_item_index >= items.len()
        {
            self.selected_item_index = 0;
        }

        ui.window("Sekiro Item Grant")
            .position([30.0, 30.0], Condition::FirstUseEver)
            .size([420.0, 260.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Insert toggles the overlay.");
                ui.separator();

                if let Some(items) = items {
                    let filtered_indices = self.filtered_item_indices(items);
                    if let Some(first_match) = filtered_indices.first().copied()
                        && !filtered_indices.contains(&self.selected_item_index)
                    {
                        self.selected_item_index = first_match;
                    }

                    if self.focus_item_filter_next_frame {
                        ui.set_keyboard_focus_here();
                        self.focus_item_filter_next_frame = false;
                    }
                    ui.input_text("Search", &mut self.item_filter).build();

                    let preview_value = if filtered_indices.is_empty() {
                        String::from("No matching items")
                    } else {
                        items[self.selected_item_index].in_game_name.clone()
                    };

                    if let Some(_combo) = ui.begin_combo("Item", &preview_value) {
                        for index in filtered_indices {
                            let item = &items[index];
                            let selected = index == self.selected_item_index;
                            if ui
                                .selectable_config(&item.in_game_name)
                                .selected(selected)
                                .build()
                            {
                                self.selected_item_index = index;
                            }
                            if selected {
                                ui.set_item_default_focus();
                            }
                        }
                    }

                    if let Some(item) = items.get(self.selected_item_index) {
                        ui.text(format!(
                            "Selected: {}{}",
                            item.in_game_name,
                                String::new()
                        ));
                    }
                } else if let Some(err) = &catalog_error {
                    ui.text_wrapped(err);
                }

                ui.input_text("Count", &mut self.count).build();

                let requests = self.parse_requests();
                let validation_error = requests.as_ref().err().map(|err| err.to_string());
                {
                    let _enabled = ui.begin_enabled(requests.is_ok());
                    if ui.button("Grant Item") {
                        self.status = match requests.as_ref() {
                            Ok(requests) => match item_grant::queue_grant_requests(requests) {
                                Ok(()) => format!(
                                    "Queued item grant",
                                ),
                                Err(err) => format!("Could not queue grant: {err}"),
                            },
                            Err(err) => err.to_string(),
                        };
                    }
                }

                ui.separator();
                if let Some(err) = validation_error {
                    ui.text_wrapped(format!("Input error: {err}"));
                } else if !self.status.is_empty() {
                    ui.text_wrapped(&self.status);
                }
            });
    }
}

impl ImguiRenderLoop for ItemGrantOverlay {
    fn initialize<'a>(&'a mut self, ctx: &mut Context, _render_context: &'a mut dyn RenderContext) {
        ctx.io_mut().font_global_scale = 1.2;
    }

    fn before_render<'a>(
        &'a mut self,
        ctx: &mut Context,
        _render_context: &'a mut dyn RenderContext,
    ) {
        ctx.io_mut().mouse_draw_cursor = self.visible;
        self.poll_toggle();
    }

    fn render(&mut self, ui: &mut Ui) {
        if self.visible {
            self.render_window(ui);
        }
    }

    fn message_filter(&self, _io: &Io) -> MessageFilter {
        if self.visible {
            MessageFilter::InputAll
        } else {
            MessageFilter::empty()
        }
    }
}