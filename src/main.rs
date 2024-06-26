use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

struct KeyDisplayApp {
    latest_key: Option<String>,
    key_receiver: Receiver<String>,
    last_press_time: Instant,
    opacity: f32,
}

impl KeyDisplayApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_keys = Vec::new();

            loop {
                let keys: Vec<Keycode> = device_state.get_keys();
                if keys != last_keys {
                    if let Some(key) = keys.last() {
                        tx.send(format!("{:?}", key)).unwrap();
                    }
                    last_keys = keys;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });

        Self {
            latest_key: None,
            key_receiver: rx,
            last_press_time: Instant::now(),
            opacity: 1.0,
        }
    }
}

impl eframe::App for KeyDisplayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(key) = self.key_receiver.try_recv() {
            self.latest_key = Some(key);
            self.last_press_time = Instant::now();
            self.opacity = 1.0;
        }

        let elapsed = self.last_press_time.elapsed();
        if elapsed > Duration::from_secs_f32(0.3) {
            self.opacity = (1.0 - (elapsed.as_secs_f32() - 0.3) / 0.4).clamp(0.0, 1.0);
        }

        // Get the screen size
        let screen_rect = frame
            .info()
            .window_info
            .monitor_size
            .unwrap_or(frame.info().window_info.size);

        // Set the frame size
        let frame_width = 200.0;
        let frame_height = 100.0;
        let margin = 50.0;

        // Calculate the frame position (bottom center)
        let frame_pos = egui::pos2(
            (screen_rect.x - frame_width) / 2.0,
            screen_rect.y - frame_height - margin,
        );

        // Set the new window position and size
        frame.set_window_pos(frame_pos);
        frame.set_window_size(egui::vec2(frame_width, frame_height));

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let rect_color =
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, (self.opacity * 100.0) as u8);
                ui.painter().rect_filled(rect, 10.0, rect_color);

                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    // Use golden ratio for vertical positioning
                    ui.add_space(frame_height * (1.0 - 0.618));

                    if let Some(key) = &self.latest_key {
                        let text = egui::RichText::new(key).size(24.0).color(
                            egui::Color32::from_rgba_unmultiplied(
                                255,
                                255,
                                255,
                                (self.opacity * 255.0) as u8,
                            ),
                        );
                        ui.label(text);
                    }
                });
            });

        ctx.request_repaint();
    }
}

fn main() {
    let options = eframe::NativeOptions {
        always_on_top: true,
        transparent: true,
        decorated: false,
        initial_window_size: Some(egui::vec2(200.0, 100.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Key Display Widget",
        options,
        Box::new(|cc| Box::new(KeyDisplayApp::new(cc))),
    )
    .unwrap();
}
