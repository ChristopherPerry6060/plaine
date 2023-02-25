#![windows_subsystem = "windows"]

mod instruct;

use anyhow::{bail, Result};
use eframe::{
    egui::{self, Grid, Ui, Window, CentralPanel},
    NativeOptions, Theme, emath::Align2,
};
use plaine::{
    plan::{Entry, Plan},
    read::GDrivePlan,
    utils::{gen_pw, TrunkFileName},
};
use rfd::FileDialog;
use std::{collections::HashSet, path::PathBuf};

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
    _gd_plan_failed_upload: bool,
    branch_pending_items: Vec<Entry>,
    branch_pending_name: Option<String>,
    confirm_branch_setting: bool,
    items: Vec<Entry>,
    trunk: Option<String>,
    trunk_store: Vec<TrunkFileName>,
    unselected: HashSet<String>,
}

impl Gui {
    /// Return a default instance.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn main_window(&mut self, ui: &mut Ui) {
        self.button_gd_upload(ui);

        if ui.button("Write Check File").clicked() {
            let items = self.items.clone();
            let plan_name = self.trunk.clone().unwrap_or_default();
            plaine::write::write_check_file(items, plan_name).expect("File to write");
        };

        if ui.button("Write Upload File").clicked() {
            let unselected = self.unselected.clone();
            let selected_items = self.items.remove_fnskus(unselected.into_iter());

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
    }

    fn button_gd_upload(&mut self, ui: &mut Ui) {
        if !ui.button("Upload").clicked() {
            return;
        };

        let Some(file_picker) = FileDialog::new().pick_file() else {
            return;
        };

        if let Ok(mut items) = self.try_upload_proc(file_picker) {
            self.items.append(&mut items)
        };
    }

    fn try_upload_proc(&mut self, picked: PathBuf) -> Result<Vec<Entry>> {
        let items = GDrivePlan::proc_from_path(picked)?;
        let trunk = gen_pw();
        items.serialize_to_fs(&trunk, None)?;
        self.items.clear();
        self.trunk = Some(trunk);
        Ok(items)
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
                instruct::instruction_window(ui);
            });
            Window::new(name)
                .vscroll(true)
                .hscroll(true)
                .pivot(Align2::RIGHT_TOP)
                .show(ctx, |ui| {
                    self.main_window(ui);
                });
        });
    }
}
