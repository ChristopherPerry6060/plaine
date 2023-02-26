#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod instruct;

use anyhow::{bail, Result};
use eframe::{
    egui::{self, CentralPanel, Grid, TopBottomPanel, Ui, Window},
    emath::Align2,
    NativeOptions, Theme,
};
use plaine::{
    plan::{Entry, Plan},
    read::GDrivePlan,
    utils::{self, gen_pw, TrunkFileName},
};
use rfd::FileDialog;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        SidePanel::left("branch-panel").show(ctx, |ui| self.comp_trunks_branches_list(ui));
        Window::new("Instruction").show(ctx, |ui| {
            instruct::instruction_window(ui);
        });
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Reset App").clicked() {
                *self = Gui::default();
            };
            self.main_window(ui);
        });
    }
}

/// The configurations available to [`Gui`].
///
/// # Fields
/// * `relative_trunk_path`: "\[Binary Root\]/.local/"
///
/// [`Gui`]:(crate::Gui)
#[derive(Debug, Clone)]
struct GuiConfig {
    relative_trunk_path: PathBuf,
}

impl Default for GuiConfig {
    fn default() -> Self {
        let relative_trunk_path = PathBuf::from(".local/");
        Self {
            relative_trunk_path,
        }
    }
}

impl GuiConfig {
    /// Return a relative `Path` to [`Trunk`] storage.
    ///
    /// [Trunk]:(crate::Trunk)
    fn relative_trunk_path(&self) -> &std::path::Path {
        &self.relative_trunk_path
    }
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    _gd_plan_failed_upload: bool,
    branch_pending_items: Vec<Entry>,
    branch_pending_name: Option<String>,
    config: GuiConfig,
    confirm_branch_setting: bool,
    error_stack: Vec<String>,
    items: Vec<Entry>,
    trunk: Option<String>,
    trunk_store: Vec<TrunkFileName>,
    unselected: HashSet<String>,
}

impl Gui {
    /// Swap out all of the items in [`Self`].
    ///
    /// Always prefer to use this rather altering the field directly.
    /// Using this pattern will allow to containerize certain updates that
    /// need to take place on branch/trunk changes.
    fn clear_and_push_items(&mut self, items: Vec<Entry>, trunk: Option<Trunk>) {
        if trunk.is_some() {
            self.items = items;
            self.trunk = trunk;
        };
    }

    /// A component displaying all available [`Trunk`]s and [`Branch`]s.
    fn comp_trunks_branches_list(&mut self, ui: &mut Ui) {
        // When a refresh is requested, load up records
        if ui.button("Refresh").clicked() {
            let path = self.config.relative_trunk_path();
            if let Ok(trunks) = utils::gather_records(path) {
                self.trunk_store = trunks;
            };
        };

        Grid::new("comp_trunks_branches_list_grid")
            .num_columns(5)
            .show(ui, |ui| {
                ui.label("Name");
                ui.label("Status");
                ui.label("Select");
                ui.label("Note");
                ui.label("Actions");
                ui.end_row();
                let trunk_store = self.trunk_store.clone();
                let mut splits = trunk_store
                    .iter()
                    .filter_map(|trunk_name| trunk_name.split_once('_'));
                let selection = splits.find_map(|split| {
                    let (name, _id) = split;
                    ui.label(name);
                    ui.label(name);
                    if ui.small_button("Select").clicked() {
                        // Only the trunk name is needed
                        // since we will need to summate all records
                        // within that trunk.
                        return Some(name);
                    };
                    ui.label("Notes");
                    ui.label("Menus");
                    ui.end_row();
                    None
                });
                if let Some(item) = selection {
                    self.try_load_trunk(item);
                };
            });
    }

    fn try_load_trunk(&mut self, trunk: &str) {
        let path = self.config.relative_trunk_path();
        let cloner = self.trunk_store.clone();
        let items: Vec<Entry> = cloner
            .into_iter()
            .filter(|in_tks| in_tks.contains(trunk))
            .flat_map(|to_pull| {
                let mut fullpath = PathBuf::from(path);
                fullpath.push(to_pull);
                let Ok(str) = std::fs::read_to_string(&fullpath) else {
                    return None;
                };
                serde_json::from_str::<Vec<Entry>>(&str).ok()
            })
            .flatten()
            .collect();

        let cloned_trunk = trunk.to_string();
        self.clear_and_push_items(items, Some(cloned_trunk));
    }
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
