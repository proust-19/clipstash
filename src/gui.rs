use anyhow::Result;
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::clipboard::ClipboardMonitor;
use crate::storage::ClipboardHistory;

pub fn run_gui(history_path: PathBuf) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ClipStash - Floating Clipboard Manager")
            .with_inner_size([440.0, 600.0])
            .with_min_inner_size([120.0, 48.0])
            .with_always_on_top()
            .with_decorations(true)
            .with_transparent(true)
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "ClipStash",
        native_options,
        Box::new(|_cc| Ok(Box::new(ClipStashApp::new(history_path)))),
    )
    .map_err(|e| anyhow::anyhow!("Failed to start GUI: {}", e))
}

struct ClipStashApp {
    history_path: PathBuf,
    history: ClipboardHistory,
    monitor: ClipboardMonitor,
    search_query: String,
    toast: Option<(String, Instant)>,
    is_expanded: bool,
    always_on_top: bool,
}

impl ClipStashApp {
    fn new(history_path: PathBuf) -> Self {
        let history = ClipboardHistory::load(&history_path).unwrap_or_else(|_| ClipboardHistory::new(100));
        let monitor = ClipboardMonitor::new();

        // Populate initial clipboard state
        if let Ok(Some(content)) = monitor.get_current_content() {
            let mut history = history;
            if history.add_entry(content) {
                let _ = history.save(&history_path);
            }
            return Self {
                history_path,
                history,
                monitor,
                search_query: String::new(),
                toast: None,
                is_expanded: true,
                always_on_top: true,
            };
        }

        Self {
            history_path,
            history,
            monitor,
            search_query: String::new(),
            toast: None,
            is_expanded: true,
            always_on_top: true,
        }
    }

    fn copy_to_clipboard(&mut self, content: &str) {
        if self.monitor.set_content(content).is_ok() {
            let preview = if content.len() > 25 {
                format!("{}...", &content[..25])
            } else {
                content.to_string()
            };
            self.toast = Some((format!("Copied: \"{}\"", preview), Instant::now()));
        }
    }

    fn delete_entry(&mut self, id: u64) {
        if self.history.delete_entry(id) {
            let _ = self.history.save(&self.history_path);
            self.toast = Some(("Entry deleted".to_string(), Instant::now()));
        }
    }

    fn toggle_pin(&mut self, id: u64) {
        if self.history.toggle_pin(id) {
            let _ = self.history.save(&self.history_path);
        }
    }

    fn clear_unpinned(&mut self) {
        self.history.clear();
        let _ = self.history.save(&self.history_path);
        self.toast = Some(("Cleared unpinned entries".to_string(), Instant::now()));
    }

    fn expand(&mut self, ctx: &egui::Context) {
        self.is_expanded = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(440.0, 600.0)));
    }

    fn collapse_to_bubble(&mut self, ctx: &egui::Context) {
        self.is_expanded = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(160.0, 52.0)));
    }
}

impl eframe::App for ClipStashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Periodically monitor clipboard changes
        ctx.request_repaint_after(Duration::from_millis(500));

        // Check if external clipboard changed
        if self.monitor.has_changed() {
            if let Some(content) = self.monitor.last_content() {
                if self.history.add_entry(content.to_string()) {
                    let _ = self.history.save(&self.history_path);
                }
            }
        }

        // Configure custom dark theme visuals
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(230, 235, 245));
        ctx.set_style(style);

        // Render Collapsed Floating Desktop Bubble ("Chat Head Widget") Mode
        if !self.is_expanded {
            egui::CentralPanel::default()
                .frame(egui::Frame::none().inner_margin(4.0).fill(egui::Color32::from_rgb(18, 22, 32)))
                .show(ctx, |ui| {
                    ui.centered_and_justified(|ui| {
                        let total = self.history.list().len();
                        let bubble_text = format!("📋 ClipStash ({})", total);

                        let bubble_frame = egui::Frame::none()
                            .fill(egui::Color32::from_rgb(30, 41, 59))
                            .stroke(egui::Stroke::new(1.5_f32, egui::Color32::from_rgb(96, 165, 250)))
                            .rounding(egui::Rounding::same(20.0))
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0));

                        let resp = bubble_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&bubble_text)
                                        .strong()
                                        .color(egui::Color32::from_rgb(96, 165, 250)),
                                );
                            });
                        }).response;

                        if resp.clicked() || ui.button("✨ Expand").clicked() {
                            self.expand(ctx);
                        }

                        resp.on_hover_text("Click to expand ClipStash clipboard history");
                    });
                });
            return;
        }

        // Render Expanded Full Window Mode
        egui::TopBottomPanel::top("header_panel")
            .frame(egui::Frame::none().inner_margin(12.0).fill(egui::Color32::from_rgb(18, 22, 32)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("📋 ClipStash").strong().color(egui::Color32::from_rgb(96, 165, 250)));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Collapse to Bubble Button
                        if ui.button(egui::RichText::new("➖ Bubble").small()).clicked() {
                            self.collapse_to_bubble(ctx);
                        }

                        // Always-on-top toggle
                        let top_icon = if self.always_on_top { "📌 Pinned" } else { "📍 Float" };
                        if ui.button(egui::RichText::new(top_icon).small()).clicked() {
                            self.always_on_top = !self.always_on_top;
                            let level = if self.always_on_top {
                                egui::WindowLevel::AlwaysOnTop
                            } else {
                                egui::WindowLevel::Normal
                            };
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                        }

                        let total = self.history.list().len();
                        ui.label(egui::RichText::new(format!("{} items", total)).small().color(egui::Color32::from_rgb(148, 163, 184)));
                    });
                });

                ui.add_space(8.0);

                // Search Bar
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search history...")
                            .desired_width(ui.available_width() - 35.0),
                    );
                    if !self.search_query.is_empty() {
                        if ui.button("❌").clicked() {
                            self.search_query.clear();
                            search_response.request_focus();
                        }
                    }
                });

                // Toast Notification Banner
                if let Some((msg, created_at)) = &self.toast {
                    if created_at.elapsed() < Duration::from_secs(2) {
                        ui.add_space(6.0);
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(30, 58, 138))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(egui::RichText::new(msg).strong().color(egui::Color32::WHITE));
                                });
                            });
                    } else {
                        self.toast = None;
                    }
                }
            });

        egui::TopBottomPanel::bottom("footer_panel")
            .frame(egui::Frame::none().inner_margin(8.0).fill(egui::Color32::from_rgb(18, 22, 32)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("💡 Click item to copy")
                            .small()
                            .color(egui::Color32::from_rgb(148, 163, 184)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("Clear History").small().color(egui::Color32::from_rgb(248, 113, 113))).clicked() {
                            self.clear_unpinned();
                        }
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(8.0).fill(egui::Color32::from_rgb(26, 32, 46)))
            .show(ctx, |ui| {
                let query = self.search_query.trim().to_lowercase();
                let entries: Vec<_> = self.history
                    .list()
                    .iter()
                    .rev()
                    .filter(|e| query.is_empty() || e.content.to_lowercase().contains(&query))
                    .cloned()
                    .collect();

                if entries.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(if query.is_empty() { "No clipboard entries yet.\nCopy something to get started!" } else { "No matching entries found." })
                                .color(egui::Color32::from_rgb(148, 163, 184)),
                        );
                    });
                    return;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in entries {
                        let id = entry.id;
                        let content = entry.content.clone();
                        let pinned = entry.pinned;

                        let card_bg = if pinned {
                            egui::Color32::from_rgb(45, 42, 30)
                        } else {
                            egui::Color32::from_rgb(33, 41, 58)
                        };

                        let card_stroke = if pinned {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(234, 179, 8))
                        } else {
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(45, 55, 75))
                        };

                        egui::Frame::none()
                            .fill(card_bg)
                            .stroke(card_stroke)
                            .rounding(egui::Rounding::same(8.0))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Pin Button
                                    let pin_icon = if pinned { "📌" } else { "📍" };
                                    let pin_btn = ui.button(pin_icon);
                                    if pin_btn.clicked() {
                                        self.toggle_pin(id);
                                    }
                                    if pin_btn.hovered() {
                                        pin_btn.on_hover_text(if pinned { "Unpin item" } else { "Pin item" });
                                    }

                                    // Content Text (Clickable to copy)
                                    let available_w = ui.available_width() - 70.0;
                                    let text_label = ui.add_sized(
                                        [available_w, 0.0],
                                        egui::SelectableLabel::new(
                                            false,
                                            egui::RichText::new(&content).color(egui::Color32::from_rgb(241, 245, 249)),
                                        ),
                                    );

                                    if text_label.clicked() {
                                        self.copy_to_clipboard(&content);
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Delete Button
                                        let del_btn = ui.button(
                                            egui::RichText::new("🗑")
                                                .color(egui::Color32::from_rgb(248, 113, 113)),
                                        );
                                        if del_btn.clicked() {
                                            self.delete_entry(id);
                                        }
                                        if del_btn.hovered() {
                                            del_btn.on_hover_text("Delete entry");
                                        }

                                        // Copy Button
                                        let copy_btn = ui.button("📋");
                                        if copy_btn.clicked() {
                                            self.copy_to_clipboard(&content);
                                        }
                                        if copy_btn.hovered() {
                                            copy_btn.on_hover_text("Copy to clipboard");
                                        }
                                    });
                                });
                            });

                        ui.add_space(6.0);
                    }
                });
            });
    }
}
