use eframe::egui;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Instant, Duration};

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

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(key) = self.key_receiver.try_recv() {
            self.latest_key = Some(key);
            self.last_press_time = Instant::now();
            self.opacity = 1.0;
        }

        let elapsed = self.last_press_time.elapsed();
        if elapsed > Duration::from_millis(500) {
            self.opacity = (1.0 - (elapsed.as_secs_f32() - 0.5) / 0.5).max(0.0).min(1.0);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    if let Some(key) = &self.latest_key {
                        let text = egui::RichText::new(key)
                            .size(24.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 255, (self.opacity * 255.0) as u8));
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
        Box::new(|cc| Box::new(KeyDisplayApp::new(cc)))
    ).unwrap();
}
