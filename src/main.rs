#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


use eframe::egui;

extern crate tinyfiledialogs;

use chrono::offset::Utc;
use serde::{Deserialize, Serialize};

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct FolderLockDecryptApp {
    input_dir: String,
    output_dir: String,
    export_dir: String,
    decrypting: bool,
    processed_files: Vec<ProcessedFileInfo>,
    json_export: bool,
    txt_export: bool,
    error_message: Option<String>,
    show_export_success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProcessedFileInfo {
    file_name: String,
    output_path: String,
}

impl Default for FolderLockDecryptApp {
    fn default() -> Self {
        Self {
            input_dir: "Input Directory".to_owned(),
            output_dir: "Output Directory".to_owned(),
            export_dir: "Export Directory".to_owned(),
            decrypting: false,
            processed_files: Vec::new(),
            json_export: false,
            txt_export: false,
            error_message: None,
            show_export_success: false,
        }
    }
}

impl eframe::App for FolderLockDecryptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Folder Lock File Decrypt");
            ui.separator();
            ui.label("This program is intended for use with the encrypted files from within the Android Folder Lock application");
            egui::Grid::new("central_panel_grid")
            .num_columns(2)
            .striped(false)
            .spacing([2.0, 2.0])
            .show(ui, |ui| {

                if self.show_export_success {
                    egui::Window::new("Export Successful")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("The data has been successfully exported.");
                        if ui.button("OK").clicked() {
                            self.show_export_success = false;
                        }
                    });
                }
                ui.end_row();
                ui.label("Input Directory");
                ui.end_row();
                if ui.button("Select Input Directory").clicked() {
                    self.input_dir = file_input();
                }
                ui.text_edit_singleline(&mut self.input_dir);
                ui.end_row();
                ui.separator();
                ui.end_row();

                ui.label("Output Directory");
                ui.end_row();
                if ui.button("Select Output Directory").clicked() {
                    self.output_dir = file_input();
                }
                ui.text_edit_singleline(&mut self.output_dir);

                ui.end_row();
                ui.separator();
                ui.end_row();
                if ui.button("Decrypt").clicked() {
                    if !is_valid_path(&self.input_dir) || !is_valid_path(&self.output_dir) {
                        self.error_message = Some("Invalid input or output directory".into());
                    } else {
                        let entries = match fs::read_dir(&self.input_dir) {
                            Ok(entries) => entries,
                            Err(err) => {
                                eprintln!("Error reading directory: {}", err);
                                self.error_message = Some(format!("Error reading input directory: {}", err));
                                return;
                            }
                        };

                        for entry in entries {
                            let entry = match entry {
                                Ok(entry) => entry,
                                Err(err) => {
                                    eprintln!("Error reading directory entry: {}", err);
                                    continue;
                                }
                            };

                            let file_path = entry.path();
                            let file_path_str = match file_path.to_str() {
                                Some(path_str) => path_str,
                                None => {
                                    eprintln!("Invalid file path");
                                    continue;
                                }
                            };

                            if let Err(err) = process_file(file_path_str, &self.output_dir, &mut self.processed_files) {
                                eprintln!("Error processing file: {}", err);
                            }
                        }
                        self.decrypting = true;
                        self.error_message = None;
                    }
                }

                if let Some(ref error_msg) = self.error_message {
                    ui.colored_label(egui::Color32::RED, error_msg);
                }
            });
        });

        if self.decrypting {
            egui::SidePanel::right("decrypting_log")
            .resizable(true)
            .show_animated(ctx, true, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Decryption Log");
                    ui.separator();
                    egui::Grid::new("right_panel_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([2.0, 2.0])
                    .show(ui, |ui| {

                        if ui.button("Export").clicked() {
                            if !is_valid_path(&self.export_dir) {
                                self.error_message = Some("Invalid export directory. Please select a valid directory before exporting.".into());
                                self.show_export_success = false;
                            } else {
                                let export_folder = &self.export_dir;

                                let mut export_path = PathBuf::from(export_folder);

                                let datetime = Utc::now();
                                let formatted = datetime.format("%d-%m-%Y-%H-%M-%S").to_string();

                                if self.txt_export {
                                    export_path.push(format!("export_{}.txt", formatted));
                                    let text_data: String = self.processed_files
                                    .iter()
                                    .map(|info| format!("Processed: {} -> {}\n", info.file_name, info.output_path))
                                    .collect();

                                    if let Err(e) = fs::write(&export_path, text_data) {
                                        eprintln!("Error exporting TXT: {}", e);
                                        self.error_message = Some("Failed to export to TXT file.".into());
                                    }
                                    export_path.pop();
                                }

                                if self.json_export {
                                    export_path.push(format!("export_{}.json", formatted));
                                    match serde_json::to_string(&self.processed_files) {
                                        Ok(json_data) => {
                                            if let Err(e) = fs::write(&export_path, json_data) {
                                                eprintln!("Error exporting JSON: {}", e);
                                                self.error_message = Some("Failed to export to JSON file.".into());
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Error serializing JSON: {}", e);
                                            self.error_message = Some("Failed to serialize data to JSON.".into());
                                        }
                                    }
                                    export_path.pop();
                                }

                                self.error_message = None;
                                if self.error_message.is_none() {
                                    self.show_export_success = true;
                                }
                            }
                        }

                        if let Some(ref error_msg) = self.error_message {
                            ui.colored_label(egui::Color32::RED, error_msg);
                        }

                        ui.end_row();
                        if ui.button("Select Export Directory").clicked() {
                            match tinyfiledialogs::select_folder_dialog("Select folder", "") {
                                Some(selected_folder) => {
                                    if is_valid_path(&selected_folder) {
                                        self.export_dir = selected_folder;
                                        self.error_message = None;
                                    } else {
                                        self.error_message = Some("Invalid export directory selected".into());
                                    }
                                },
                                None => self.error_message = Some("No export directory selected".into()),
                            }
                        }
                        ui.text_edit_singleline(&mut self.export_dir);

                        if let Some(ref error_msg) = self.error_message {
                            ui.colored_label(egui::Color32::RED, error_msg);
                        }
                        ui.end_row();

                        if ui.radio(self.txt_export, "Text File").clicked() {
                            self.txt_export = true;
                            self.json_export = false;
                        }
                        if ui.radio(self.json_export, "JSON File").clicked() {
                            self.json_export = true;
                            self.txt_export = false;
                        }

                    });
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for info in &self.processed_files {
                            ui.label(format!("Processed: {} -> {}", info.file_name, info.output_path));
                        }
                    });
                });
            }); 
        }
    }
}

fn is_valid_path(path: &str) -> bool {
    !path.trim().is_empty() && std::path::Path::new(path).exists()
}

fn process_file(
    input_file_path: &str,
    output_directory: &str,
    processed_files: &mut Vec<ProcessedFileInfo>,
) -> io::Result<()> {
    let mut input_file = File::open(input_file_path)?;

    let mut buffer = [0u8; 111];
    input_file.read_exact(&mut buffer)?;

    buffer.reverse();

    let mut rest_of_file = Vec::new();
    input_file.read_to_end(&mut rest_of_file)?;

    let file_name = match std::path::Path::new(input_file_path).file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            eprintln!("Invalid file path: {}", input_file_path);
            return Ok(());
        }
    };

    let new_file_name = file_name.replace("#", ".");

    let output_file_path = format!("{}/{}", output_directory, new_file_name);
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&output_file_path)?;

    output_file.write_all(&buffer)?;
    output_file.write_all(&rest_of_file)?;

    processed_files.push(ProcessedFileInfo {
        file_name: input_file_path.to_string(),
        output_path: output_file_path.clone(),
    });

    Ok(())
}

fn file_input() -> String {
    let folder: String;
    match tinyfiledialogs::select_folder_dialog("Select folder", "") {
        Some(result) => folder = result,
        None => folder = "null".to_string(),
    }
    return folder;
}

fn main() -> Result<(), eframe::Error> {
    FolderLockDecryptApp::default();

    let options = eframe::NativeOptions {
        initial_window_size: None,
        ..Default::default()
    };

    eframe::run_native(
        "Folder Lock Decrypt",
        options,
        Box::new(|_cc| Box::new(FolderLockDecryptApp::default())),
    )
}
