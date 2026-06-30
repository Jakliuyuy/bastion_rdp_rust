#![windows_subsystem = "windows"]

mod crypto;
mod api;
mod config;

use eframe::egui;
use egui::FontDefinitions;
use std::sync::{Arc, Mutex};
use std::fs;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 300.0])
            .with_resizable(false)
            .with_centered(),
        ..Default::default()
    };
    eframe::run_native("堡垒机", options, Box::new(|cc| {
        setup_fonts(&cc.egui_ctx);
        Ok(Box::new(BastionApp::new()))
    }))
}

fn setup_fonts(ctx: &egui::Context) {
    if let Ok(data) = fs::read("C:/Windows/Fonts/msyh.ttc") {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert("msyh".into(), Arc::new(egui::FontData::from_owned(data)));
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.insert(0, "msyh".into());
        }
        ctx.set_fonts(fonts);
    }
}

struct BastionApp {
    state: AppState,
    result: Arc<Mutex<Option<Result<String, String>>>>,
    msg: String,
    cfg: config::Config,
}

enum AppState {
    MainMenu,
    Config,
    Connecting,
    Done(Result<String, String>),
}

impl BastionApp {
    fn new() -> Self {
        let cfg = config::load();
        let state = if cfg.user.is_empty() || cfg.password.is_empty() {
            AppState::Config
        } else {
            AppState::MainMenu
        };
        Self { state, result: Arc::new(Mutex::new(None)), msg: String::new(), cfg }
    }
}

impl eframe::App for BastionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if background task completed
        if let AppState::Connecting = self.state {
            if let Ok(mut r) = self.result.lock() {
                if let Some(res) = r.take() {
                    self.state = AppState::Done(res);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                AppState::MainMenu => self.show_main_menu(ui, ctx),
                AppState::Config => self.show_config(ui),
                AppState::Connecting => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label("正在连接...");
                        ui.add_space(20.0);
                        ui.spinner();
                    });
                    ctx.request_repaint();
                }
                AppState::Done(Ok(_)) => {
                    let mut back = false;
                    ui.vertical_centered(|ui| {
                        ui.add_space(60.0);
                        ui.heading("连接成功");
                        ui.add_space(10.0);
                        ui.label("远程桌面已启动");
                        ui.add_space(20.0);
                        if ui.add_sized([150.0, 35.0], egui::Button::new("返回")).clicked() {
                            back = true;
                        }
                    });
                    if back { self.state = AppState::MainMenu; }
                }
                AppState::Done(Err(e)) => {
                    let mut back = false;
                    let err = e.clone();
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.heading("连接失败");
                        ui.add_space(10.0);
                        ui.label(&err);
                        ui.add_space(20.0);
                        if ui.add_sized([150.0, 35.0], egui::Button::new("返回")).clicked() {
                            back = true;
                        }
                    });
                    if back { self.state = AppState::MainMenu; }
                }
            }
        });
    }
}

impl BastionApp {
    fn show_main_menu(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading("堡垒机一键连接");
            ui.add_space(30.0);

            if ui.add_sized([260.0, 40.0], egui::Button::new("使用上次账号密码连接"))
                .on_hover_text("直接连接")
                .clicked()
            {
                self.start_connect();
            }

            ui.add_space(10.0);
            if ui.add_sized([260.0, 40.0], egui::Button::new("修改配置")).clicked() {
                self.state = AppState::Config;
            }
        });
    }

    fn show_config(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(15.0);
            ui.heading("配置");
            ui.add_space(15.0);

            egui::Grid::new("config_grid")
                .striped(true)
                .min_col_width(100.0)
                .show(ui, |ui| {
                    ui.label("堡垒机用户名:");
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.user).desired_width(200.0));
                    ui.end_row();
                    ui.label("堡垒机密码:");
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.password).password(true).desired_width(200.0));
                    ui.end_row();
                    ui.label("服务器密码:");
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.server_pwd).password(true).desired_width(200.0));
                    ui.end_row();
                    ui.label("服务器账号:");
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.server_user).desired_width(200.0));
                    ui.end_row();
                    ui.label("服务器IP:");
                    ui.add(egui::TextEdit::singleline(&mut self.cfg.server_ip).desired_width(200.0));
                    ui.end_row();
                });

            ui.add_space(15.0);
            ui.horizontal(|ui| {
                ui.add_space(60.0);
                if ui.add_sized([120.0, 35.0], egui::Button::new("保存并连接")).clicked() {
                    if !self.cfg.user.is_empty() && !self.cfg.password.is_empty() {
                        config::save(&self.cfg);
                        self.start_connect();
                    }
                }
                if ui.add_sized([120.0, 35.0], egui::Button::new("取消")).clicked() {
                    self.state = AppState::MainMenu;
                }
            });
        });
    }

    fn start_connect(&mut self) {
        self.state = AppState::Connecting;
        let cfg = self.cfg.clone();
        let result = self.result.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let res: Result<String, String> = rt.block_on(async { api::login_and_connect(&cfg).await });
            if let Ok(url) = &res {
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", url])
                    .spawn().ok();
            }
            let mut r = result.lock().unwrap();
            *r = Some(res);
        });
    }
}
