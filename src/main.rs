use device_query::{DeviceQuery, DeviceState, Keycode};
use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const BASE_WIDTH: f32 = 400.0;
const BASE_HEIGHT: f32 = 100.0;
// Max buffer size is used to clear the buffer when it's too long
const MAX_BUFFER_SIZE: usize = 7;
// Debounce duration is used to prevent false keystrokes
//(when the same key is pressed multiple times in a short period of time, it is likely false, and should be filtered)
const DEBOUNCE_DURATION: Duration = Duration::from_millis(50);
// Time in seconds before the text starts to fade
const FADE_START_TIME: f32 = 0.7;
// Duration in seconds for the text to fade out
const FADE_DURATION: f32 = 0.4;

const DELIMITERS: &[&str] = &["Space", "Enter", "Comma", "Period"];

struct KeyMapping {
    name: &'static str,
    symbol: &'static str,
}

struct KeyDisplayApp {
    buffer: Vec<String>,
    key_receiver: Receiver<String>,
    last_press_time: Instant,
    last_key: Option<String>,
    opacity: f32,
}

impl KeyDisplayApp {
    const KEY_MAPPINGS: &'static [KeyMapping] = &[
        // Navigation and control
        KeyMapping {
            name: "Backspace",
            symbol: "<BS>",
        },
        KeyMapping {
            name: "Enter",
            symbol: "<ENTER>",
        },
        KeyMapping {
            name: "Space",
            symbol: " ",
        },
        KeyMapping {
            name: "Tab",
            symbol: "<TAB>",
        },
        // Arrows
        KeyMapping {
            name: "Up",
            symbol: "<UP>",
        },
        KeyMapping {
            name: "Down",
            symbol: "<DN>",
        },
        KeyMapping {
            name: "Left",
            symbol: "<L>",
        },
        KeyMapping {
            name: "Right",
            symbol: "<R>",
        },
        // Special keys
        KeyMapping {
            name: "Escape",
            symbol: "<Esc>",
        },
        KeyMapping {
            name: "Delete",
            symbol: "<Del>",
        },
        KeyMapping {
            name: "Home",
            symbol: "<HOME>",
        },
        KeyMapping {
            name: "End",
            symbol: "<END>",
        },
        // Modifiers
        KeyMapping {
            name: "LShift",
            symbol: "<LShft>",
        },
        KeyMapping {
            name: "RShift",
            symbol: "<RShft>",
        },
        KeyMapping {
            name: "LControl",
            symbol: "<LCtrL>",
        },
        KeyMapping {
            name: "RControl",
            symbol: "<RCtrl>",
        },
        KeyMapping {
            name: "LAlt",
            symbol: "<LAlt>",
        },
        KeyMapping {
            name: "RAlt",
            symbol: "<RAlt>",
        },
        KeyMapping {
            name: "Meta",
            symbol: "<M>",
        },
    ];

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
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            buffer: Vec::new(),
            key_receiver: rx,
            last_press_time: Instant::now(),
            last_key: None,
            opacity: 1.0,
        }
    }

    // this is to check whether the key repeats the last key, which can be a false keystroke
    fn is_valid_keystroke(&self, key: &str) -> bool {
        match &self.last_key {
            Some(last) if last == key => self.last_press_time.elapsed() > DEBOUNCE_DURATION,
            _ => true,
        }
    }

    fn should_clear_buffer(&self, key: &str) -> bool {
        DELIMITERS.iter().any(|&delim| key == delim) || self.buffer.len() >= MAX_BUFFER_SIZE
    }

    fn format_key(&self, key: String) -> String {
        Self::KEY_MAPPINGS
            .iter()
            .find(|mapping| mapping.name == key)
            .map(|mapping| format!(" {}", mapping.symbol)) // add a space before the symbol
            .unwrap_or(key)
    }
}

impl eframe::App for KeyDisplayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(key) = self.key_receiver.try_recv() {
            if self.is_valid_keystroke(&key) {
                let key = self.format_key(key.clone());
                if self.should_clear_buffer(&key) {
                    self.buffer.clear();
                } else {
                    self.buffer.push(key.clone());
                }
                self.last_press_time = Instant::now();
                self.last_key = Some(key);
                self.opacity = 1.0;
            }
        }

        let elapsed = self.last_press_time.elapsed();
        if elapsed > Duration::from_secs_f32(FADE_START_TIME) {
            self.opacity = (1.0 - (elapsed.as_secs_f32() - FADE_DURATION) / 0.4).clamp(0.0, 1.0);
            if self.opacity <= 0.0 {
                self.buffer.clear();
            }
        }

        let screen_rect = frame
            .info()
            .window_info
            .monitor_size
            .unwrap_or(frame.info().window_info.size);

        let (frame_width, frame_height) = (BASE_WIDTH, BASE_HEIGHT);
        let margin = 50.0;

        let frame_pos = egui::pos2(
            (screen_rect.x - frame_width) / 2.0,
            screen_rect.y - frame_height - margin,
        );

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

                    if !self.buffer.is_empty() {
                        let text = egui::RichText::new(self.buffer.join("")).size(24.0).color(
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
        initial_window_size: Some(egui::vec2(BASE_WIDTH, BASE_HEIGHT)),
        ..Default::default()
    };
    eframe::run_native(
        "Key Display Widget",
        options,
        Box::new(|cc| Box::new(KeyDisplayApp::new(cc))),
    )
    .unwrap();
}
