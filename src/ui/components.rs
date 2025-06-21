use eframe::egui;
use uuid::Uuid;
use crate::notes::NoteGroup;

/// Утилиты для создания переиспользуемых UI компонентов
pub struct UiComponents;

impl UiComponents {
    /// Создает стандартную кнопку с иконкой
    pub fn icon_button(
        ui: &mut egui::Ui,
        icon: &str,
        tooltip: &str,
        bg_color: egui::Color32,
        border_color: egui::Color32,
    ) -> egui::Response {
        let button = egui::Button::new(icon)
            .min_size(egui::Vec2::new(24.0, 24.0))
            .fill(bg_color)
            .stroke(egui::Stroke::new(1.0, border_color));
        
        ui.add(button).on_hover_text(tooltip)
    }
    
    /// Создает стандартную рамку для текстовых полей
    pub fn text_field_frame() -> egui::Frame {
        egui::Frame::new()
            .stroke(egui::Stroke::new(0.7, egui::Color32::GRAY))
            .corner_radius(6)
            .inner_margin(egui::Margin::same(6))
    }
    
    /// Создает однострочное текстовое поле
    pub fn single_line_text_edit(
        ui: &mut egui::Ui,
        text: &mut String,
        hint: &str,
        width: f32,
        height: f32,
    ) -> egui::Response {
        let text_edit = egui::TextEdit::singleline(text)
            .hint_text(hint)
            .frame(false);
        
        #[cfg(target_os = "linux")]
        {
            let text_edit = text_edit.clip_text(false);
            ui.add_sized([width, height], text_edit)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            ui.add_sized([width, height], text_edit)
        }
    }
    
    /// Создает многострочное текстовое поле с автоматической высотой
    pub fn multiline_text_edit(
        ui: &mut egui::Ui,
        text: &mut String,
        hint: &str,
    ) -> egui::Response {
        Self::text_field_frame().show(ui, |ui| {
            Self::multiline_text_edit_internal(ui, text, hint)
        })
        .inner
    }
    
    fn multiline_text_edit_internal(
        ui: &mut egui::Ui,
        text: &mut String,
        hint: &str,
    ) -> egui::Response {
        let line_count = text.lines().count().max(1) + 2;
        let max_lines = 15;
        
        if line_count <= max_lines {
            let text_edit = egui::TextEdit::multiline(text)
                .hint_text(hint)
                .desired_rows(line_count)
                .desired_width(ui.available_width())
                .frame(false);
            
            #[cfg(target_os = "linux")]
            {
                ui.add(text_edit.clip_text(false).code_editor())
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                ui.add(text_edit)
            }
        } else {
            let scroll_height = max_lines as f32 * 18.0;
            
            egui::ScrollArea::vertical()
                .max_height(scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(text)
                        .hint_text(hint)
                        .desired_width(ui.available_width())
                        .frame(false);
                    
                    #[cfg(target_os = "linux")]
                    {
                        ui.add(text_edit.clip_text(false).code_editor())
                    }
                    
                    #[cfg(not(target_os = "linux"))]
                    {
                        ui.add(text_edit)
                    }
                })
                .inner
        }
    }
    
    /// Создает стандартный селектор групп
    pub fn group_selector(
        ui: &mut egui::Ui,
        current_group_id: Option<Uuid>,
        groups: &[NoteGroup],
        id_salt: &str,
        on_change: impl FnOnce(Option<Uuid>),
        max_groups_shown: usize,
    ) {
        let mut new_group_id: Option<Option<Uuid>> = None;
        
        let selected_text = groups
            .iter()
            .find(|g| Some(g.id) == current_group_id)
            .map(|g| g.name.as_str())
            .unwrap_or("Без группы");
        
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        // Опция "Без группы"
                        if ui.add_sized(
                            [ui.available_width(), 24.0],
                            egui::SelectableLabel::new(current_group_id.is_none(), "Без группы"),
                        )
                        .clicked()
                        {
                            new_group_id = Some(None);
                        }
                        
                        // Показываем группы с ограничением
                        let groups_to_show: Vec<_> = groups.iter().take(max_groups_shown).collect();
                        
                        for group in groups_to_show {
                            let is_selected = current_group_id == Some(group.id);
                            let group_text = if group.level > 0 {
                                format!("{}{}", "  ".repeat(group.level as usize), group.name)
                            } else {
                                group.name.clone()
                            };
                            
                            if ui.add_sized(
                                [ui.available_width(), 24.0],
                                egui::SelectableLabel::new(is_selected, group_text),
                            )
                            .clicked()
                            {
                                new_group_id = Some(Some(group.id));
                            }
                        }
                        
                        // Показываем сообщение о лимите
                        if groups.len() > max_groups_shown {
                            ui.separator();
                            ui.label(format!("Показано {} из {} групп", max_groups_shown, groups.len()));
                        }
                    });
            });
        
        // Применяем изменения
        if let Some(gid) = new_group_id {
            on_change(gid);
        }
    }
} 