#![windows_subsystem = "windows"]
use eframe::egui::{self, CentralPanel, Grid, Ui, Window};
use eframe::{NativeOptions, Theme};
use plaine::plan::{Entry, Plan};
use rfd::FileDialog;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let native_options = NativeOptions {
        follow_system_theme: false,
        default_theme: Theme::Light,
        ..Default::default()
    };
    eframe::run_native(
        "Plaine",
        native_options,
        Box::new(|cc| Box::new(Gui::new(cc))),
    ).expect("eframe init to succeed");
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    plan_name: Option<String>,
    selected: HashSet<String>,
    items: Vec<Entry>,
}

impl Gui {
    /// Return a default instance.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Opens a file dialog in a new thread.
    fn show_file_diaglog() -> Option<PathBuf> {
        let file_picker = FileDialog::new().set_title("Select a Plan csv");
        file_picker.pick_file()
    }

    fn instruction_window(&mut self, ui: &mut Ui) {
        ui.separator();
        ui.strong("Prefix GD plan with CLOSED");
        ui.separator();
        ui.label("1: Go to Google Drive Shipping Plan folder.");
        ui.label("2: Open the plan you want to close by clicking on it.");
        ui.label("3: Double click the name of the plan in the top left.");
        ui.label("3a: This should allow you to edit the name of the sheet.");
        ui.label("4: In all capital letters, prefix the name with CLOSED.");
        ui.separator();
        ui.strong("Download Plan as CSV");
        ui.separator();
        ui.label("5: Right under where you just changed the name, click on the File menu.");
        ui.label("6: Navigate the menu until you see the 'Download' option.");
        ui.label("7: Hover over download and select CSV.");
        ui.label("8: Keep note of where our downloaded file was saved to!");
        ui.label("8a: It likely went straight into your downloads folder.");
        ui.label("8b: Anywhere is fine, we will just need it for the next steps.");
        ui.separator();
        ui.strong("Plaine");
        ui.separator();
        ui.label("Moving away from Google Drive.");
        ui.label("Note: If you are already reading this from within Plaine, you can skip right down to 'Check File'.");
        ui.label("9: (More than halfway done!) Navigate in your computer's file browser to 'Dropbox/Gramophone'");
        ui.label("10: Look for a regular folder named 'Plaine'");
        ui.label("11: Open the folder, and double click the Plaine application.");
        ui.separator();
        ui.strong("Check File");
        ui.separator();
        ui.label("1. In the other window click the 'Upload' button.");
        ui.label("2. Use the file dialog to locate the csv file we download earlier.");
        ui.label("3. Once found, double-click / select the csv file.");
        ui.label("4. Plaine will proccess your file and show you a table of the contents");
        ui.separator();
        ui.label("Click on 'Write Check File'");
    }
    fn main_window(&mut self, ui: &mut Ui) {
        let item_copy = self.items.clone();
        let mut sums = item_copy.get_as_sums();
        sums.sort_by_key(|i| i.get_fnsku().to_string());

        let selected = &mut self.selected;

        if ui.button("Upload").clicked() {
            let picked = Gui::show_file_diaglog();
            if let Some(file) = picked {
                self.items = plaine::read::GDrivePlan::proc_from_path(file).unwrap_or_default();
                self.plan_name = Some(plaine::utils::gen_pw());
            };
        };

        Grid::new("buttons").show(ui, |ui| {
            ui.label("msku");
            ui.label("Fnsku");
            ui.label("Units");
            ui.label("Amz Size");
            ui.label("Title");
            ui.end_row();

            for entry in sums {
                // Is the fnsku in the selected set?
                let fnsku_lookup = entry.get_fnsku();
                let checkbox_state = selected.contains(fnsku_lookup);
                let _pre_checkbox_state = checkbox_state;

                // string
                let amz_size = entry.get_amz_size().clone().unwrap_or_default();
                let fnsku = entry.get_fnsku();
                let msku = entry.get_msku().clone().unwrap_or_default();
                let title = entry.get_title().clone().unwrap_or_default();
                let _asin = entry.get_asin().clone().unwrap_or_default();
                let units = entry.get_units().to_string();
                let _total_pounds = entry.get_total_pounds().unwrap_or_default();
                let _id = entry.get_id();
                let _upc = entry.get_upc().clone().unwrap_or_default();

                ui.label(msku);
                ui.label(fnsku);
                ui.label(units);
                ui.label(amz_size);
                ui.label(title);
                ui.end_row();

                if checkbox_state {
                    selected.insert(fnsku_lookup.to_owned());
                } else {
                    selected.remove(fnsku_lookup);
                };
            }
        });
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let name = self
            .plan_name
            .clone()
            .unwrap_or_else(|| String::from("Waiting on Upload :)"));
        CentralPanel::default().show(ctx, |_| {
            Window::new("Instruction").show(ctx, |ui| {
                self.instruction_window(ui);
            });
            Window::new(name).show(ctx, |ui| {
                if ui.button("Write Check File").clicked() {
                    let items = self.items.clone();
                    let plan_name = self.plan_name.clone().unwrap_or_default();
                    plaine::write::write_check_file(items, plan_name)
                        .expect("File to write");
                }
                self.main_window(ui);
            });
        });
    }
}
