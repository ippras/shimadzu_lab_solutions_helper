use eframe::egui::{self, DragValue, Label, RichText};
use eframe::emath::Float;
use egui_dnd::dnd;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

// Глобальный счетчик для выдачи уникальных ID строкам
static ID: AtomicU64 = AtomicU64::new(0);

/// App
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    rows: Vec<Row>,
    mode: AppMode,
    text_content: String,
}

impl App {
    pub fn new(creation_context: &eframe::CreationContext) -> Self {
        // egui_phosphor
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        creation_context.egui_ctx.set_fonts(fonts);
        // Persistence
        if let Some(storage) = creation_context.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Верхняя панель с кнопками
        egui::Panel::top("TopPanel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Генератор градиента насосов");
                ui.separator();

                let text = if self.mode == AppMode::Table {
                    egui_phosphor::regular::TEXT_AA
                } else {
                    egui_phosphor::regular::TABLE
                };
                let hover_text = if self.mode == AppMode::Table {
                    "Простмотр в режиме текста"
                } else {
                    "Простмотр в режиме таблицы"
                };
                if ui
                    .button(RichText::new(text).heading())
                    .on_hover_text(hover_text)
                    .clicked()
                {
                    self.toggle_mode();
                }
            });
        });

        // Центральная панель
        egui::CentralPanel::default().show(ui, |ui| match self.mode {
            AppMode::Table => self.show_table(ui),
            AppMode::Text => self.show_text(ui),
        });
    }
}

impl App {
    fn show_table(&mut self, ui: &mut egui::Ui) {
        let size = [ui.spacing().combo_width, ui.spacing().interact_size.y];
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Заголовки таблицы (используем фиксированную ширину для имитации колонок)
            ui.horizontal(|ui| {
                ui.add_sized(ui.spacing().interact_size, egui::Label::new("")); // Под handle
                ui.add_sized(ui.spacing().interact_size, egui::Label::new("")); // Под кнопку удаления

                ui.add_sized(
                    size,
                    egui::Label::new(egui::RichText::new("Time (min)").strong()),
                );
                ui.add_sized(
                    size,
                    egui::Label::new(egui::RichText::new("Pump A (%)").strong()),
                );
                ui.add_sized(
                    size,
                    egui::Label::new(egui::RichText::new("Pump B (%)").strong()),
                );
                ui.add_sized(
                    size,
                    egui::Label::new(egui::RichText::new("Total Flow").strong()),
                );
                ui.add(egui::Label::new(
                    egui::RichText::new("Description").strong(),
                ));
            });

            ui.add_space(4.0);

            let mut row_to_delete = None;

            dnd(ui, "dnd_example").show_vec(&mut self.rows, |ui, item, handle, _state| {
                ui.horizontal(|ui| {
                    // Handle (кнопка для перетаскивания)
                    handle.ui(ui, |ui| {
                        ui.add_sized(
                            ui.spacing().interact_size,
                            Label::new(egui_phosphor::regular::DOTS_SIX_VERTICAL),
                        );
                    });

                    // 6. Кнопка удаления
                    if ui
                        .add_sized(
                            ui.spacing().interact_size,
                            egui::Button::new(egui_phosphor::regular::MINUS),
                        )
                        .clicked()
                    {
                        row_to_delete = Some(item.id);
                    }

                    // 1. Time
                    ui.add_sized(
                        size,
                        DragValue::new(&mut item.time)
                            .speed(0.1)
                            .range(0.0..=1000.0),
                    );

                    // 2. Pump A
                    let mut a_temp = item.pump_a;
                    if ui
                        .add_sized(
                            size,
                            DragValue::new(&mut a_temp).speed(0.1).range(0.0..=100.0),
                        )
                        .changed()
                    {
                        item.pump_a = a_temp;
                        item.pump_b = 100.0 - a_temp;
                    }

                    // 3. Pump B
                    let mut b_temp = item.pump_b;
                    if ui
                        .add_sized(
                            size,
                            DragValue::new(&mut b_temp).speed(0.1).range(0.0..=100.0),
                        )
                        .changed()
                    {
                        item.pump_b = b_temp;
                        item.pump_a = 100.0 - b_temp;
                    }

                    // 4. Total Flow
                    ui.add_sized(
                        size,
                        DragValue::new(&mut item.flow)
                            .speed(0.01)
                            .range(0.0..=100.0),
                    );

                    // 5. Description
                    // ui.text_edit_singleline(&mut item.description);
                    ui.add(
                        egui::TextEdit::singleline(&mut item.description)
                            .desired_width(f32::INFINITY),
                    );
                });
            });

            // Удаляем строку по ID, если была нажата кнопка
            if let Some(id) = row_to_delete {
                self.rows.retain(|row| row.id != id);
            }

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        ui.spacing().interact_size,
                        egui::Button::new(egui_phosphor::regular::SORT_ASCENDING),
                    )
                    .clicked()
                {
                    self.rows.sort_by_cached_key(|row| row.time.ord());
                }
                if ui
                    .add_sized(
                        ui.spacing().interact_size,
                        egui::Button::new(egui_phosphor::regular::PLUS),
                    )
                    .clicked()
                {
                    self.rows.push(Row::default());
                }
            });
        });
    }

    fn show_text(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.text_content)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20),
            );
        });
    }

    fn toggle_mode(&mut self) {
        if self.mode == AppMode::Table {
            // Переход: Таблица -> Текст
            self.text_content.clear();
            for row in &self.rows {
                // Форматируем f64 в строку и меняем точку на запятую
                let t = format!("{}", row.time).replace('.', ",");
                let b = format!("{}", row.pump_b).replace('.', ",");
                let f = format!("{}", row.flow).replace('.', ",");
                let d = &row.description;

                self.text_content
                    .push_str(&format!("{}\tÍàñîñû\tPump B Conc.\t{}\t{}\n", t, b, d));
                self.text_content
                    .push_str(&format!("{}\tÍàñîñû\tTotal Flow\t{}\t\n", t, f));
            }
            self.mode = AppMode::Text;
        } else {
            // Переход: Текст -> Таблица (Парсинг)
            let mut new_rows = Vec::new();
            let mut temp_row: Option<Row> = None;

            for (index, line) in self.text_content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 4 {
                    // error!("index={index} line={line}");
                    continue;
                }

                let time_str = parts[0].trim();
                let param = parts[2].trim();
                let val_str = parts[3].trim();
                let desc = if parts.len() > 4 {
                    parts[4].trim().to_string()
                } else {
                    "".to_string()
                };

                // Парсим строки обратно в f64 (меняя запятую на точку)
                let time_val = time_str.replace(',', ".").parse::<f64>().unwrap_or(0.0);
                let val_f64 = val_str.replace(',', ".").parse::<f64>().unwrap_or(0.0);

                if param == "Pump B Conc." {
                    let mut row = Row::default();
                    row.time = time_val;
                    row.pump_b = val_f64;
                    row.pump_a = 100.0 - val_f64;
                    row.description = desc;
                    temp_row = Some(row);
                } else if param == "Total Flow" {
                    if let Some(mut row) = temp_row.take() {
                        // Сравниваем f64 с учетом погрешности (epsilon)
                        if (row.time - time_val).abs() < f64::EPSILON {
                            row.flow = val_f64;
                            new_rows.push(row);
                        } else {
                            // Если время не совпало, сохраняем старую и создаем новую
                            new_rows.push(row);
                            let mut new_row = Row::default();
                            new_row.time = time_val;
                            new_row.flow = val_f64;
                            new_rows.push(new_row);
                        }
                    } else {
                        let mut new_row = Row::default();
                        new_row.time = time_val;
                        new_row.flow = val_f64;
                        new_rows.push(new_row);
                    }
                }
            }
            // Если осталась непарная строка Pump B
            if let Some(row) = temp_row {
                new_rows.push(row);
            }

            self.rows = new_rows;
            // if self.rows.is_empty() {
            //     self.rows.push(Row::default());
            // }

            self.mode = AppMode::Table;
        }
    }
}

// Структура для хранения данных одной строки.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Row {
    id: u64,
    time: f64,
    pump_a: f64,
    pump_b: f64,
    flow: f64,
    description: String,
}

// Реализуем Clone вручную, чтобы при клонировании строка получала новый уникальный ID
impl Clone for Row {
    fn clone(&self) -> Self {
        Self {
            id: ID.fetch_add(1, Ordering::Relaxed),
            time: self.time,
            pump_a: self.pump_a,
            pump_b: self.pump_b,
            flow: self.flow,
            description: self.description.clone(),
        }
    }
}

impl Default for Row {
    fn default() -> Self {
        Self {
            id: ID.fetch_add(1, Ordering::Relaxed),
            time: 0.0,
            pump_a: 0.0,   // По умолчанию насос А = 0%
            pump_b: 100.0, // По умолчанию насос B = 100%
            flow: 1.0,     // Поток по умолчанию
            description: String::new(),
        }
    }
}

impl Hash for Row {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Режимы работы приложения
#[derive(Clone, Copy, Default, PartialEq, serde::Deserialize, serde::Serialize)]
enum AppMode {
    #[default]
    Table,
    Text,
}
