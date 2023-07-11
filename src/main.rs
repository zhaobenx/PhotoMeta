#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// This project aims to read all photos in one directory and count the focal length, and then output with a chart
use eframe::egui;
use eframe::egui::plot::{log_grid_spacer, Text};
use egui::plot::{HLine, Line, Plot};
use exif::{In, Reader, Tag};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use walkdir::WalkDir;

extern crate log;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(720.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "焦段统计",
        options,
        Box::new(|cc| Box::new(MainApp::new(cc))),
    )
}

#[derive(Default)]
struct MainApp {
    picked_path: Option<String>,
    statics_line: Option<Vec<[f64; 2]>>,
}

struct _ExifInformation {
    focal_lengths: Vec<[f64; 2]>,
}

impl MainApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        let font = std::fs::read("c:/Windows/Fonts/msyh.ttc").unwrap();

        fonts
            .font_data
            .insert("微软雅黑".to_owned(), egui::FontData::from_owned(font));

        // Put my font first (highest priority) for proportional text:
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "微软雅黑".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("微软雅黑".to_owned());

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);
        Self::default()
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("常用焦段信息统计");
            if ui.button("选择文件夹").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.picked_path = Some(path.display().to_string());
                    let statics = match get_focal_length_statics(self.picked_path.clone().unwrap())
                    {
                        Ok(statics) => statics,
                        Err(e) => {
                            ui.label(format!("读取文件夹失败：{}", e));
                            return;
                        }
                    };

                    let mut statics: Vec<_> = statics
                        .iter()
                        .map(|(focal_length, count)| {
                            [focal_length.parse::<f64>().unwrap(), count.clone() as f64]
                        })
                        .collect();
                    statics.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
                    self.statics_line = Some(statics);
                }
            }
            if let Some(path) = &self.picked_path {
                ui.label(path);
            }

            if let Some(line) = &self.statics_line {
                ui.label(format!(
                    "共{}张照片",
                    line.iter().map(|[_, y]| y).sum::<f64>()
                ));

                ui.horizontal(|ui| {
                    Plot::new("my_plot")
                        .show_axes([true, false])
                        // .x_axis_formatter(|x, _| format!("{}mm!", x))
                        // .clamp_grid(true)
                        .show_y(false)
                        .auto_bounds_y()
                        .auto_bounds_x()
                        .label_formatter(|_, p| {
                            // if line.contains(x)// TODO:::: 解决一下动态字符串显示问题

                            return format!("{:.2}mm\n{}", p.x, p.y);
                            // if line
                            //     .iter()
                            //     .any(|[x, _]| x.round() as i32 == p.x.round() as i32)
                            // {
                            //     return format!("{:.2}mm", p.x);
                            // } else {
                            //     return "".to_string();
                            // }
                        })
                        .x_grid_spacer(log_grid_spacer(10))
                        .view_aspect(2.0)
                        .include_y(0.0)
                        .include_x(0.0)
                        .show(ui, |plot_ui| {
                            plot_ui.hline(HLine::new(0));
                            plot_ui.line(Line::new(line.clone()));
                            let avg =
                                line.iter().fold(0.0, |acc, [_, y]| acc + y) / line.len() as f64;

                            line.iter().for_each(|[x, y]| {
                                if *y > avg {
                                    plot_ui.text(Text::new(
                                        [*x + 1., *y + 1.].into(),
                                        format!("{}mm:{}", x, y),
                                    ))
                                }
                            });
                        });
                    ui.label("hhh")
                });
            }
        });
    }
}

fn get_focal_length_statics(
    directory: String,
) -> Result<HashMap<String, i64>, Box<dyn std::error::Error>> {
    let mut focal_lengths = Vec::new();

    WalkDir::new(Path::new(&directory))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| fs::File::open(entry.path()).ok())
        .filter_map(|file_reader| {
            Reader::new()
                .read_from_container(&mut BufReader::new(file_reader))
                .ok()
        })
        .for_each(|exif| {
            if let Some(focal_length) = exif.get_field(Tag::FocalLength, In::PRIMARY) {
                focal_lengths.push(focal_length.display_value().to_string());
            }
        });

    let mut counts = HashMap::new();
    for focal_length in focal_lengths {
        *counts.entry(focal_length).or_insert(0) += 1;
    }

    println!("Focal length counts:");
    for (focal_length, count) in counts.iter() {
        println!("{}: {}", focal_length, count);
    }
    return Ok(counts);
}

// TODO: 增加相机、镜头的统计功能
// maybe not TODO: 给出不同焦段的示例图片
// TODO: 针对 APSC 计算出等效焦距（似乎有点难）
// TODO: 实现从 github 上自动更新
