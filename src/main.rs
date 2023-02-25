#![windows_subsystem = "windows"]
use anyhow::{bail, Result};
use eframe::{
    egui::{self, CentralPanel, Grid, Ui, Window},
    NativeOptions, Theme,
};

use plaine::{
    plan::{Entry, Plan},
    read::GDrivePlan,
    utils::gen_pw,
    Trunk,
};
use rfd::FileDialog;
use std::{collections::HashSet, path::PathBuf};

/// [`Entry`](s) and their [`Trunk`], as response from [`Gui::gd_intake`].
#[derive(Default, Debug)]
struct GdIntake {
    trunk: plaine::Trunk,
    items: Vec<Entry>,
}

impl GdIntake {
    /// Create a new [`Self`] from a [`Trunk`] and [`Entry`].
    fn new(trunk: Trunk, items: Vec<Entry>) -> Self {
        Self { trunk, items }
    }

    /// Take ownership of [`Self`].
    fn take(self) -> (Trunk, Vec<Entry>) {
        (self.trunk, self.items)
    }
}

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
    )
    .expect("eframe init to succeed");
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    trunk: Option<String>,
    unselected: HashSet<String>,
    items: Vec<Entry>,
    branch_pending_name: Option<String>,
    branch_pending_items: Vec<Entry>,
    confirm_branch_setting: bool,
    gd_plan_failed_upload: bool,
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
        //
        ui.separator();
        ui.strong("Prefix GD plan with CLOSED");
        ui.separator();
        //
        ui.label("1: Go to Google Drive Shipping Plan folder.");
        ui.label("2: Open the plan you want to close by clicking on it.");
        ui.label("3: Double click the name of the plan in the top left.");
        ui.label("3a: This should allow you to edit the name of the sheet.");
        ui.label("4: In all capital letters, prefix the name with CLOSED.");
        //
        ui.separator();
        ui.strong("Download Plan as CSV");
        ui.separator();
        //
        ui.label("5: Right under where you just changed the name, click on the File menu.");
        ui.label("6: Navigate the menu until you see the 'Download' option.");
        ui.label("7: Hover over download and select CSV.");
        ui.label("8: Keep note of where our downloaded file was saved to!");
        ui.label("8a: It likely went straight into your downloads folder.");
        ui.label("8b: Anywhere is fine, we will just need it for the next steps.");
        //
        ui.separator();
        ui.strong("Check File");
        ui.separator();
        //
        ui.label("1. In the other window click the 'Upload' button.");
        ui.label("2. Use the file dialog to locate the csv file we download earlier.");
        ui.label("3. Once found, double-click / select the csv file.");
        ui.label("4. Plaine will proccess your file and show you a table of the contents");
        ui.separator();
        ui.label("Click on 'Write Check File'");
    }
    fn main_window(&mut self, ui: &mut Ui) {
        let unselected = self.unselected.clone();
        let upload_button = ui.button("Upload");

        let picked = match upload_button.clicked() {
            true => Gui::show_file_diaglog(),
            false => None,
        };

        if ui.button("Write Check File").clicked() {
            let items = self.items.clone();
            let plan_name = self.trunk.clone().unwrap_or_default();
            plaine::write::write_check_file(items, plan_name).expect("File to write");
        };

        if ui.button("Write Upload File").clicked() {
            let selected_items = self.items.filter_fnskus(unselected.into_iter());

            let plan_name = self.trunk.clone().unwrap_or_default();
            match plaine::write::write_upload_txt(selected_items, plan_name) {
                Ok(response) => {
                    let (branch, items) = response.take();
                    self.branch_pending_name = Some(branch);
                    self.branch_pending_items = items;
                }
                Err(err) => {
                    dbg!(format!("Uh oh, {err}."));
                }
            };
        };

        if let Some(branch) = &self.branch_pending_name {
            let button_text = format!("Set the branch: {branch}?");
            if ui.button(button_text).clicked() {
                self.confirm_branch_setting = true
            };
            if self.confirm_branch_setting && ui.button("Are you sure?").clicked() {
                // TODO: This should show a warning about the error.
                if let Ok(mut negatives) = self.try_write_branch(Some(branch)) {
                    self.items.append(&mut negatives)
                };
            };
        };

        Grid::new("buttons").striped(true).show(ui, |ui| {
            self.fill_grid(ui);
        });
        let Some(file) = picked else {
            return;
        };

        let Ok(intake) = Gui::gd_intake(file) else {
            self.gd_intake_failed();
            return;
        };

        self.items.clear();
        let (trunk, mut items) = intake.take();
        self.trunk = Some(trunk);
        self.items.append(&mut items);
    }

    fn gd_intake_failed(&mut self) {
        self.gd_plan_failed_upload = true;
    }
    fn gd_intake(file: PathBuf) -> Result<GdIntake> {
        let plan = GDrivePlan::proc_from_path(file)?;
        let trunk = gen_pw();
        plan.serialize_to_fs(&trunk, None)?;
        Ok(GdIntake::new(trunk, plan))
    }

    fn try_write_branch(&self, branch: Option<&str>) -> Result<Vec<Entry>> {
        let trunk = match &self.trunk {
            Some(ref name) => name,
            None => bail!("Can't branch without a trunk!"),
        };

        let set = self.branch_pending_items.clone();
        let negatives = set.as_negated();
        let _ = set.serialize_to_fs(trunk, branch)?;
        Ok(negatives)
    }

    /// Fill the Ui with a grid, displaying sums of the passed entries.
    fn fill_grid(&mut self, ui: &mut Ui) {
        let mut sums = self.items.clone().get_as_sums();
        sums.sort_by_key(|i| i.get_fnsku().to_string());
        let un_set = &mut self.unselected;

        ui.label("");
        ui.label("mSku");
        ui.label("Upc*");
        ui.label("Fnsku");
        ui.label("Units");
        ui.label("Amz Size");
        ui.label("Condition");
        ui.label("Title");
        ui.end_row();

        sums.into_iter()
            .filter(|entry| *entry.get_units() > 0)
            .for_each(|entry| {

                let fnsku = entry.get_fnsku();
                // When the fnsku is NOT in the map, display check.
                let mut check = !un_set.contains(fnsku);
                ui.checkbox(&mut check, "");

                ui.label(entry.str_msku());
                ui.label(entry.str_upc());
                ui.label(entry.str_fnsku());
                ui.label(entry.get_units().to_string());
                ui.label(entry.str_amz_size());
                ui.label(entry.str_condition());
                ui.label(entry.str_title());
                ui.end_row();

                // When the fnsku is NOT checked, insert.
                // When check, remove.
                if !check {
                    un_set.insert(fnsku.to_owned());
                } else {
                    un_set.remove(fnsku);
                };
            });
    }
}


impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let name = self
            .trunk
            .clone()
            .unwrap_or_else(|| String::from("Waiting on Upload :)"));
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Reset App").clicked() {
                *self = Gui::default();
            };
            Window::new("Instruction").show(ctx, |ui| {
                self.instruction_window(ui);
            });
            Window::new(name).show(ctx, |ui| {
                self.main_window(ui);
            });
        });
    }
}
