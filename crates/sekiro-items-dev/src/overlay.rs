use std::time::Duration;

use anyhow::{Result, anyhow};
use fromsoftware_shared::Program;
use hudhook::hooks::dx11::ImguiDx11Hooks;
use hudhook::imgui::{Condition, Context, Io, Ui};
use hudhook::{Hudhook, ImguiRenderLoop, MessageFilter, RenderContext};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_INSERT};

use crate::item_grant::{self, GrantRequest};

const ITEM_CATEGORIES: [ItemCategoryOption; 3] = [
    ItemCategoryOption::new("Weapon", 0),
    ItemCategoryOption::new("Goods", 4),
    ItemCategoryOption::new("Protector", 1),
];

#[derive(Clone, Copy)]
struct ItemCategoryOption {
    label: &'static str,
    value: u8,
}

impl ItemCategoryOption {
    const fn new(label: &'static str, value: u8) -> Self {
        Self { label, value }
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
    focus_item_id_next_frame: bool,
    item_id: String,
    selected_category_index: usize,
    count: String,
    status: String,
}

impl Default for ItemGrantOverlay {
    fn default() -> Self {
        Self {
            visible: true,
            insert_was_down: false,
            focus_item_id_next_frame: true,
            item_id: String::new(),
            selected_category_index: 0,
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
                self.focus_item_id_next_frame = true;
            }
        }
        self.insert_was_down = insert_down;
    }

    fn selected_category(&self) -> Result<ItemCategoryOption> {
        ITEM_CATEGORIES
            .get(self.selected_category_index)
            .copied()
            .ok_or_else(|| anyhow!("Selected category is invalid"))
    }

    fn parse_requests(&self) -> Result<Vec<GrantRequest>> {
        let item_id = self
            .item_id
            .trim()
            .parse::<u32>()
            .map_err(|_| anyhow!("Item ID must be an unsigned integer"))?;

        let count = self
            .count
            .trim()
            .parse::<u32>()
            .map_err(|_| anyhow!("Count must be an unsigned integer"))?;

        let category = self.selected_category()?;
        let request = GrantRequest::new(item_id, category.value, count);
        item_grant::validate_grant_request(request)?;

        Ok(vec![request])
    }

    fn render_window(&mut self, ui: &Ui) {
        let selected_category = self.selected_category().ok();

        ui.window("Sekiro Item Grant")
            .position([30.0, 30.0], Condition::FirstUseEver)
            .size([360.0, 220.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Insert toggles the overlay.");
                ui.separator();

                if self.focus_item_id_next_frame {
                    ui.set_keyboard_focus_here();
                    self.focus_item_id_next_frame = false;
                }

                ui.input_text("Item ID", &mut self.item_id).build();

                let category_preview = selected_category
                    .map(|category| category.label)
                    .unwrap_or("Select category");
                if let Some(_combo) = ui.begin_combo("Category", category_preview) {
                    for (index, category) in ITEM_CATEGORIES.iter().enumerate() {
                        let selected = index == self.selected_category_index;
                        if ui
                            .selectable_config(category.label)
                            .selected(selected)
                            .build()
                        {
                            self.selected_category_index = index;
                        }
                        if selected {
                            ui.set_item_default_focus();
                        }
                    }
                }

                ui.input_text("Count", &mut self.count).build();

                let requests = self.parse_requests();
                let validation_error = requests.as_ref().err().map(|err| err.to_string());
                {
                    let _enabled = ui.begin_enabled(requests.is_ok());
                    if ui.button("Grant Item") {
                        self.status = match requests.as_ref() {
                            Ok(requests) => match item_grant::queue_grant_requests(requests) {
                                Ok(()) => {
                                    let request = requests[0];
                                    let category = self
                                        .selected_category()
                                        .map(|value| value.label)
                                        .unwrap_or("Unknown");
                                    format!(
                                        "Queued {category} item {} x{}",
                                        request.item_id, request.count
                                    )
                                }
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