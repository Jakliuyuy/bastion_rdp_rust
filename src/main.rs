mod crypto;
mod api;
mod config;

use eframe::egui;
use egui::FontDefinitions;
use std::sync::Arc;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native("堡垒机", options, Box::new(|cc| {
        setup_fonts(&cc.egui_ctx);
        Ok(Box::new(BastionApp::new()))
    }))
}

fn setup_fonts(ctx: &egui::Context) {
    if let Ok(data) = std::fs::read("C:/Windows/Fonts/msyh.ttc") {
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
    msg: String,
    cfg: config::Config,
}

enum AppState {
    MainMenu,
    Config,
    Connecting,
}

impl BastionApp {
    fn new() -> Self {
        let cfg = config::load();
        let state = if cfg.user.is_empty() || cfg.password.is_empty() {
            AppState::Config
        } else {
            AppState::MainMenu
        };
        Self { state, msg: String::new(), cfg }
    }
}

impl eframe::App for BastionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                AppState::MainMenu => self.show_main_menu(ui, ctx),
                AppState::Config => self.show_config(ui),
                AppState::Connecting => self.show_connecting(ui, ctx),
            }
        });
    }
}

impl BastionApp {
    fn show_main_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.heading("堡垒机一键连接");
            ui.add_space(30.0);

            if ui.add_sized([260.0, 40.0], egui::Button::new("使用上次账号密码连接"))
                .on_hover_text("直接连接")
                .clicked()
            {
                self.state = AppState::Connecting;
                let cfg = self.cfg.clone();
                let ctx_clone = ctx.clone();
                std::thread::spawn(move || {
                    let _result = connect_and_launch(&cfg);
                    ctx_clone.request_repaint();
                    // We need a way to communicate back. For simplicity, we store result.
                    // This is a simplified version.
                });
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
                        self.state = AppState::Connecting;
                        let cfg = self.cfg.clone();
                        std::thread::spawn(move || {
                            connect_and_launch(&cfg);
                        });
                    }
                }
                if ui.add_sized([120.0, 35.0], egui::Button::new("取消")).clicked() {
                    self.state = AppState::MainMenu;
                }
            });
        });
    }

    fn show_connecting(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label("正在连接...");
            ui.add_space(20.0);
            ui.spinner();
        });
    }
}

fn connect_and_launch(cfg: &config::Config) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        match api::login_and_connect(cfg).await {
            Ok(url) => {
                std::process::Command::new("cmd")
                    .args(["/c", "start", "", &url])
                    .spawn()
                    .ok();
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    });
}
