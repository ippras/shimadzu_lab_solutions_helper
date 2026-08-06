use eframe::emath::Float;

/// App
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    mode: AppMode,
    rows: Vec<Row>,
    text: String,
    dragged_point: Option<(usize, CurveType)>,
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
        self.panels(ui);
    }
}

#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum CurveType {
    PumpA,
    PumpB,
    Flow,
}

impl App {
    fn panels(&mut self, ui: &mut egui::Ui) {
        self.top_panel(ui);
        self.bottom_panel(ui);
        // self.left_panel(ui);
        self.central_panel(ui);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("BottomPanel").show(ui, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::Sides::new().show(
                    ui,
                    |_| {},
                    |ui| {
                        egui::warn_if_debug_build(ui);
                        ui.label(egui::RichText::new(env!("CARGO_PKG_VERSION")).small());
                        ui.separator();
                    },
                );
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| match self.mode {
            AppMode::Table => self.show_table(ui),
            AppMode::Text => self.show_text(ui),
            AppMode::Plot => self.show_plot(ui),
        });
    }

    // Top panel
    fn top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("TopPanel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.heading("Генератор градиента насосов");
                    ui.separator();

                    let mut new_mode = self.mode;

                    if ui
                        .selectable_label(
                            self.mode == AppMode::Table,
                            egui::RichText::new(egui_phosphor::regular::TABLE).heading(),
                        )
                        .on_hover_text("Режим таблицы")
                        .clicked()
                    {
                        new_mode = AppMode::Table;
                    }

                    if ui
                        .selectable_label(
                            self.mode == AppMode::Text,
                            egui::RichText::new(egui_phosphor::regular::TEXT_AA).heading(),
                        )
                        .on_hover_text("Режим текста")
                        .clicked()
                    {
                        new_mode = AppMode::Text;
                    }

                    if ui
                        .selectable_label(
                            self.mode == AppMode::Plot,
                            egui::RichText::new(egui_phosphor::regular::CHART_LINE_UP).heading(),
                        )
                        .on_hover_text("Режим графика")
                        .clicked()
                    {
                        new_mode = AppMode::Plot;
                    }

                    if new_mode != self.mode {
                        self.switch_mode(new_mode);
                    }
                });
            });
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

            let mut previous_time: Option<f64> = None;
            let mut row_to_delete = None;

            egui_dnd::dnd(ui, "dnd_example").show_vec(&mut self.rows, |ui, row, handle, _state| {
                ui.horizontal(|ui| {
                    // Handle (кнопка для перетаскивания)
                    handle.ui(ui, |ui| {
                        ui.add_sized(
                            ui.spacing().interact_size,
                            egui::Label::new(egui_phosphor::regular::DOTS_SIX_VERTICAL),
                        )
                        .on_hover_text(row.id.to_string());
                    });

                    // Кнопка удаления
                    if ui
                        .add_sized(
                            ui.spacing().interact_size,
                            egui::Button::new(egui_phosphor::regular::MINUS),
                        )
                        .clicked()
                    {
                        row_to_delete = Some(row.id);
                    }

                    // Time
                    ui.scope(|ui| {
                        if let Some(time) = previous_time.replace(row.time)
                            && time > row.time
                        {
                            ui.visuals_mut().override_text_color =
                                Some(ui.visuals().error_fg_color);
                        }
                        ui.add_sized(
                            size,
                            egui::DragValue::new(&mut row.time)
                                .speed(0.1)
                                .range(0.0..=1000.0),
                        );
                    });

                    // Pump A
                    let mut a_temp = row.pump_a;
                    if ui
                        .add_sized(
                            size,
                            egui::DragValue::new(&mut a_temp)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        )
                        .changed()
                    {
                        row.pump_a = a_temp;
                        row.pump_b = 100.0 - a_temp;
                    }

                    // Pump B
                    let mut b_temp = row.pump_b;
                    if ui
                        .add_sized(
                            size,
                            egui::DragValue::new(&mut b_temp)
                                .speed(0.1)
                                .range(0.0..=100.0),
                        )
                        .changed()
                    {
                        row.pump_b = b_temp;
                        row.pump_a = 100.0 - b_temp;
                    }

                    // Total Flow
                    ui.scope(|ui| {
                        if row.flow.abs() < f64::EPSILON {
                            ui.visuals_mut().override_text_color = Some(ui.visuals().warn_fg_color);
                        }
                        ui.add_sized(
                            size,
                            egui::DragValue::new(&mut row.flow)
                                .speed(0.01)
                                .range(0.0..=100.0),
                        );
                    });

                    // 5. Description
                    // ui.text_edit_singleline(&mut item.description);
                    ui.add(
                        egui::TextEdit::singleline(&mut row.description)
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
                egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(20),
            );
        });
    }

    fn show_plot(&mut self, ui: &mut egui::Ui) {
        // Создаем копию строк и сортируем по времени
        let mut sorted_rows = self.rows.clone();
        sorted_rows.sort_by_cached_key(|row| row.time.ord());

        // Подготавливаем точки [x, y] для каждой кривой
        let data_a: Vec<[f64; 2]> = sorted_rows
            .iter()
            .map(|row| [row.time, row.pump_a])
            .collect();
        let data_b: Vec<[f64; 2]> = sorted_rows
            .iter()
            .map(|row| [row.time, row.pump_b])
            .collect();
        let data_flow: Vec<[f64; 2]> = sorted_rows.iter().map(|row| [row.time, row.flow]).collect();

        // Задаем цвета
        let color_a = egui::Color32::from_rgb(200, 50, 50);
        let color_b = egui::Color32::from_rgb(50, 150, 250);
        let color_flow = egui::Color32::from_rgb(50, 200, 50);

        // Создаем линии
        let line_a = egui_plot::Line::new("Pump A (%)", data_a.clone())
            .color(color_a)
            .width(2.0);
        let line_b = egui_plot::Line::new("Pump B (%)", data_b.clone())
            .color(color_b)
            .width(2.0);
        let line_flow = egui_plot::Line::new("Total Flow", data_flow.clone())
            .color(color_flow)
            .width(2.0);

        // Создаем точки поверх линий
        let points_a = egui_plot::Points::new("Pump A (%)", data_a)
            .color(color_a)
            .radius(4.0);
        let points_b = egui_plot::Points::new("Pump B (%)", data_b)
            .color(color_b)
            .radius(4.0);
        let points_flow = egui_plot::Points::new("Total Flow", data_flow)
            .color(color_flow)
            .radius(4.0);

        let rows_for_label = sorted_rows.clone();
        // Отрисовываем сам график
        let mut plot = egui_plot::Plot::new("gradient_plot")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Time (min)")
            .y_axis_label("Value")
            .label_formatter(move |hover_pos| match hover_pos {
                egui_plot::HoverPosition::NearDataPoint {
                    plot_name,
                    position,
                    ..
                } if !plot_name.is_empty() => {
                    // Базовый текст подсказки
                    let mut label = format!(
                        "Time (min): {:.1}\n{plot_name}: {:.1}",
                        position.x, position.y
                    );

                    // Ищем индекс текущей точки в нашем массиве по времени (X).
                    // Используем небольшую погрешность (1e-5) для надежного сравнения f64.
                    if let Some(idx) = rows_for_label
                        .iter()
                        .position(|r| (r.time - position.x).abs() < 1e-5)
                    {
                        // --- РАСЧЕТ НАКЛОНА СЛЕВА ---
                        if idx > 0 {
                            let prev = &rows_for_label[idx - 1];
                            // Определяем Y предыдущей точки в зависимости от того, на какую линию навели
                            let prev_y = match *plot_name {
                                "Pump A (%)" => prev.pump_a,
                                "Pump B (%)" => prev.pump_b,
                                "Total Flow" => prev.flow,
                                _ => position.y,
                            };

                            let dx = position.x - prev.time;
                            if dx > 0.0 {
                                let dy = position.y - prev_y;
                                let slope_left = dy / dx;
                                label.push_str(&format!("\nSlope left: {:.2}/min", slope_left));
                            }
                        }

                        // --- РАСЧЕТ НАКЛОНА СПРАВА ---
                        if idx + 1 < rows_for_label.len() {
                            let next = &rows_for_label[idx + 1];
                            // Определяем Y следующей точки
                            let next_y = match *plot_name {
                                "Pump A (%)" => next.pump_a,
                                "Pump B (%)" => next.pump_b,
                                "Total Flow" => next.flow,
                                _ => position.y,
                            };

                            let dx = next.time - position.x;
                            if dx > 0.0 {
                                let dy = next_y - position.y;
                                let slope_right = dy / dx;
                                label.push_str(&format!("\nSlope right: {:.2}/min", slope_right));
                            }
                        }
                    }

                    Some(label)
                }
                _ => None,
            });

        // Отключаем панорамирование графика, если мы в данный момент тянем точку
        if self.dragged_point.is_some() {
            plot = plot.allow_drag(false);
        }

        // Переменные для текущего кадра
        let mut current_plot_pos = None;
        let mut current_closest_idx = None;

        // Отрисовываем график
        let plot_response = plot.show(ui, |plot_ui| {
            plot_ui.line(line_a);
            plot_ui.points(points_a);
            plot_ui.line(line_b);
            plot_ui.points(points_b);
            plot_ui.line(line_flow);
            plot_ui.points(points_flow);

            let response = plot_ui.response();
            let interact_radius = 15.0;

            // --- ОПРЕДЕЛЕНИЕ ТЕКУЩЕЙ ПОЗИЦИИ ---
            // Вычисляем, над чем сейчас находится мышь
            if let Some(pointer_pos) = response.hover_pos() {
                let mut closest_dist = f32::MAX;
                for (index, row) in self.rows.iter().enumerate() {
                    let pos_a =
                        plot_ui.screen_from_plot(egui_plot::PlotPoint::new(row.time, row.pump_a));
                    let pos_b =
                        plot_ui.screen_from_plot(egui_plot::PlotPoint::new(row.time, row.pump_b));
                    let pos_flow =
                        plot_ui.screen_from_plot(egui_plot::PlotPoint::new(row.time, row.flow));

                    let min_dist = pos_a
                        .distance(pointer_pos)
                        .min(pos_b.distance(pointer_pos))
                        .min(pos_flow.distance(pointer_pos));

                    if min_dist < closest_dist && min_dist < interact_radius {
                        closest_dist = min_dist;
                        current_closest_idx = Some(index);
                    }
                }
            }
            if let Some(pos) = plot_ui.pointer_coordinate() {
                current_plot_pos = Some([pos.x, pos.y]);
            }

            // --- А. НАЧАЛО ПЕРЕТАСКИВАНИЯ ---
            if response.drag_started() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let mut closest_dist = f32::MAX;
                    let mut closest_point = None;

                    for (index, row) in self.rows.iter().enumerate() {
                        let pos_a = plot_ui
                            .screen_from_plot(egui_plot::PlotPoint::new(row.time, row.pump_a));
                        if pos_a.distance(pointer_pos) < closest_dist
                            && pos_a.distance(pointer_pos) < interact_radius
                        {
                            closest_dist = pos_a.distance(pointer_pos);
                            closest_point = Some((index, CurveType::PumpA));
                        }

                        let pos_b = plot_ui
                            .screen_from_plot(egui_plot::PlotPoint::new(row.time, row.pump_b));
                        if pos_b.distance(pointer_pos) < closest_dist
                            && pos_b.distance(pointer_pos) < interact_radius
                        {
                            closest_dist = pos_b.distance(pointer_pos);
                            closest_point = Some((index, CurveType::PumpB));
                        }

                        let pos_flow =
                            plot_ui.screen_from_plot(egui_plot::PlotPoint::new(row.time, row.flow));
                        if pos_flow.distance(pointer_pos) < closest_dist
                            && pos_flow.distance(pointer_pos) < interact_radius
                        {
                            closest_dist = pos_flow.distance(pointer_pos);
                            closest_point = Some((index, CurveType::Flow));
                        }
                    }
                    self.dragged_point = closest_point;
                }
            }

            // --- Б. ПРОЦЕСС ПЕРЕТАСКИВАНИЯ ---
            // if response.dragged() {
            //     if let Some((idx, curve_type)) = self.dragged_point {
            //         let delta = plot_ui.pointer_coordinate_drag_delta();
            //         if let Some(row) = self.rows.get_mut(idx) {
            //             row.time = (row.time + delta.x as f64).max(0.0);
            //             match curve_type {
            //                 CurveType::PumpA => {
            //                     row.pump_a = (row.pump_a + delta.y as f64).clamp(0.0, 100.0);
            //                     row.pump_b = 100.0 - row.pump_a;
            //                 }
            //                 CurveType::PumpB => {
            //                     row.pump_b = (row.pump_b + delta.y as f64).clamp(0.0, 100.0);
            //                     row.pump_a = 100.0 - row.pump_b;
            //                 }
            //                 CurveType::Flow => row.flow = (row.flow + delta.y as f64).max(0.0),
            //             }
            //         }
            //     }
            // }
            // --- Б. ПРОЦЕСС ПЕРЕТАСКИВАНИЯ ---
            if response.dragged() {
                if let Some((idx, curve_type)) = self.dragged_point {
                    if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                        let mut target_x = pointer_pos.x;
                        let mut target_y = pointer_pos.y;

                        // Если НЕ зажат Shift, привязываем к точной визуальной сетке egui_plot
                        if !plot_ui.ctx().input(|input| input.modifiers.shift) {
                            // 8.0 - минимальное расстояние между тонкими линиями в пикселях
                            const GRID_PIXEL_SPACING: f64 = 8.0;

                            let rect = plot_ui.response().rect;
                            let bounds = plot_ui.plot_bounds();

                            // Функция, которая в точности повторяет логику next_power из log_grid_spacer
                            let calc_exact_step = |bounds_range: f64, pixels: f64| -> f64 {
                                if pixels <= 0.0 || bounds_range <= 0.0 {
                                    return 1.0;
                                }

                                let base_step_size = bounds_range * (GRID_PIXEL_SPACING / pixels);

                                // Округляем ВВЕРХ до ближайшей степени 10 (..., 0.1, 1.0, 10.0, ...)
                                10.0_f64.powi(base_step_size.log10().ceil() as i32)
                            };

                            // Вычисляем точный шаг для осей X и Y
                            let snap_x = calc_exact_step(bounds.width(), rect.width() as f64);
                            let snap_y = calc_exact_step(bounds.height(), rect.height() as f64);

                            // Округляем координаты до вычисленного шага сетки
                            target_x = (target_x / snap_x).round() * snap_x;
                            target_y = (target_y / snap_y).round() * snap_y;
                        }

                        if let Some(row) = self.rows.get_mut(idx) {
                            row.time = target_x.max(0.0);

                            match curve_type {
                                CurveType::PumpA => {
                                    row.pump_a = target_y.clamp(0.0, 100.0);
                                    row.pump_b = 100.0 - row.pump_a;
                                }
                                CurveType::PumpB => {
                                    row.pump_b = target_y.clamp(0.0, 100.0);
                                    row.pump_a = 100.0 - row.pump_b;
                                }
                                CurveType::Flow => row.flow = target_y.max(0.0),
                            }
                        }
                    }
                }
            }

            // --- В. КОНЕЦ ПЕРЕТАСКИВАНИЯ ---
            if response.drag_stopped() {
                self.dragged_point = None;
                self.rows.sort_by_cached_key(|row| row.time.ord());
            }
        });

        // --- ФИКСАЦИЯ ЦЕЛИ ДЛЯ МЕНЮ ---
        // Сохраняем данные ТОЛЬКО в тот момент, когда нажата правая кнопка мыши.
        // Это гарантирует, что при движении мыши внутри меню цель не собьется.
        if plot_response.response.hovered() && ui.input(|input| input.pointer.secondary_pressed()) {
            ui.data_mut(|data| {
                if let Some(pos) = current_plot_pos {
                    data.insert_temp(egui::Id::new("plot_menu_pos"), pos);
                }
                data.insert_temp(egui::Id::new("plot_menu_idx"), current_closest_idx);
            });
        }

        // --- КОНТЕКСТНОЕ МЕНЮ (ПРАВЫЙ КЛИК) ---
        plot_response.response.context_menu(|ui| {
            // Читаем "замороженные" данные
            let saved_pos =
                ui.data(|data| data.get_temp::<[f64; 2]>(egui::Id::new("plot_menu_pos")));
            let saved_idx = ui
                .data(|data| data.get_temp::<Option<usize>>(egui::Id::new("plot_menu_idx")))
                .flatten();

            if let Some(idx) = saved_idx {
                if ui
                    .button((egui_phosphor::regular::MINUS, "Удалить точку"))
                    .clicked()
                {
                    if self.rows.len() > 2 {
                        self.rows.remove(idx);
                    }
                    ui.close();
                }
            } else {
                if ui
                    .button((egui_phosphor::regular::PLUS, "Добавить точку"))
                    .clicked()
                {
                    if let Some(pos) = saved_pos {
                        let new_time = pos[0].max(0.0);

                        if !self.rows.is_empty() {
                            let mut new_row = self.rows[0].clone();
                            new_row.time = new_time;

                            let mut left = None;
                            let mut right = None;

                            for row in &self.rows {
                                if row.time <= new_time {
                                    left = Some(row);
                                }
                                if row.time >= new_time && right.is_none() {
                                    right = Some(row);
                                }
                            }

                            match (left, right) {
                                (Some(l), Some(r)) if l.time != r.time => {
                                    let t = (new_time - l.time) / (r.time - l.time);
                                    new_row.pump_a = l.pump_a + (r.pump_a - l.pump_a) * t;
                                    new_row.pump_b = l.pump_b + (r.pump_b - l.pump_b) * t;
                                    new_row.flow = l.flow + (r.flow - l.flow) * t;
                                }
                                (Some(l), _) => {
                                    new_row.pump_a = l.pump_a;
                                    new_row.pump_b = l.pump_b;
                                    new_row.flow = l.flow;
                                }
                                (_, Some(r)) => {
                                    new_row.pump_a = r.pump_a;
                                    new_row.pump_b = r.pump_b;
                                    new_row.flow = r.flow;
                                }
                                _ => {}
                            }

                            self.rows.push(new_row);
                            self.rows.sort_by(|a, b| {
                                a.time
                                    .partial_cmp(&b.time)
                                    .unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                    }
                    ui.close();
                }
            }
        });
    }

    fn switch_mode(&mut self, new_mode: AppMode) {
        // Если мы уходим из текстового режима (в таблицу или график), парсим текст
        if self.mode == AppMode::Text && new_mode != AppMode::Text {
            self.parse_text();
        }

        // Если мы заходим в текстовый режим (из таблицы или графика), генерируем текст
        if self.mode != AppMode::Text && new_mode == AppMode::Text {
            self.generate_text();
        }

        self.mode = new_mode;
    }

    fn generate_text(&mut self) {
        self.text.clear();
        for row in &self.rows {
            let t = format!("{}", row.time).replace('.', ",");
            let b = format!("{}", row.pump_b).replace('.', ",");
            let f = format!("{}", row.flow).replace('.', ",");
            let d = &row.description;

            self.text
                .push_str(&format!("{}\tÍàñîñû\tPump B Conc.\t{}\t{}\n", t, b, d));
            self.text
                .push_str(&format!("{}\tÍàñîñû\tTotal Flow\t{}\t\n", t, f));
        }
    }

    fn parse_text(&mut self) {
        let mut new_rows = Vec::new();
        let mut temp_row: Option<Row> = None;

        for line in self.text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
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

            let time_val = time_str.replace(',', ".").parse::<f64>().unwrap_or(0.0);
            let val_f64 = val_str.replace(',', ".").parse::<f64>().unwrap_or(0.0);

            if param == "Pump B Conc." {
                if let Some(pending_row) = temp_row.take() {
                    new_rows.push(pending_row);
                }
                let mut row = Row::default();
                row.time = time_val;
                row.pump_b = val_f64;
                row.pump_a = 100.0 - val_f64;
                row.description = desc;
                temp_row = Some(row);
            } else if param == "Total Flow" {
                if let Some(mut row) = temp_row.take() {
                    if (row.time - time_val).abs() < f64::EPSILON {
                        row.flow = val_f64;
                        new_rows.push(row);
                    } else {
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
        if let Some(row) = temp_row {
            new_rows.push(row);
        }
        self.rows = new_rows;
    }
}

// Структура для хранения данных одной строки.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Row {
    id: uuid::Uuid,
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
            id: uuid::Uuid::new_v4(),
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
            id: uuid::Uuid::new_v4(),
            time: 0.0,
            pump_a: 0.0,   // По умолчанию насос А = 0%
            pump_b: 100.0, // По умолчанию насос B = 100%
            flow: 0.0,     // Поток по умолчанию
            description: String::new(),
        }
    }
}

impl std::hash::Hash for Row {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Режимы работы приложения
#[derive(Clone, Copy, Default, PartialEq, serde::Deserialize, serde::Serialize)]
enum AppMode {
    #[default]
    Table,
    Text,
    Plot,
}
