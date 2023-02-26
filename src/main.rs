#![windows_subsystem = "windows"]

mod instruct;
use anyhow::{bail, Result};
use eframe::{
    egui::{self, CentralPanel, Grid, SidePanel, Ui, Window},
    NativeOptions,
};
use plaine::{
    plan::{Entry, Plan},
    read::GDrivePlan,
    utils::{self, gen_pw, TrunkFileName},
    Brn, RootName,
};
use rfd::FileDialog;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

fn main() {
    let native_options = NativeOptions {
        follow_system_theme: true,
        // default_theme: Theme::Light,
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
    local_dir: PathBuf,
}

impl Default for GuiConfig {
    fn default() -> Self {
        let local_dir = PathBuf::from(".local/");
        Self { local_dir }
    }
}

impl GuiConfig {
    /// Return a relative `Path` to [`Trunk`] storage.
    ///
    /// [Trunk]:(crate::Trunk)
    fn local_dir(&self) -> &std::path::Path {
        &self.local_dir
    }
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    branch_pending_items: Vec<Entry>,
    branch_pending_name: Option<String>,
    config: GuiConfig,
    items: Vec<Entry>,
    root: Option<String>,
    trunk_store: Vec<TrunkFileName>,
    unselected: HashSet<String>,
}

impl Gui {
    /// Swap out all of the items in [`Self`].
    ///
    /// Always prefer to use this rather altering the field directly.
    fn clear_and_push_items(&mut self, items: Vec<Entry>, trunk: Option<RootName>) {
        if trunk.is_some() {
            self.items = items;
            self.root = trunk;
        };
    }

    /// A component displaying all available [`Trunk`]s and [`Branch`]s.
    fn comp_trunks_branches_list(&mut self, ui: &mut Ui) {
        // When a refresh is requested, load up records
        if ui.button("Refresh").clicked() {
            let path = self.config.local_dir();
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
        let items: Vec<String> = self.trunk_store.clone();
        let branch_names = items.iter().filter(|x| {
            let Some((prefix, _suffix)) = x.split_once('_') else {
                return false;
            };
            prefix.eq(trunk)
        });

        let loaded_branches = branch_names
            .flat_map(|filename| {
                let local = self.get_local_path()?.display();
                let path = format!("{local}{filename}");
                dbg!(&path);
                let entry = std::fs::read_to_string(&path).ok()?;
                serde_json::from_str::<Vec<Entry>>(&entry).ok()
            })
            .flatten()
            .collect();

        self.clear_and_push_items(loaded_branches, Some(trunk.to_string()));
    }
    /// Return a default instance.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn main_window(&mut self, ui: &mut Ui) {
        self.google_drive_upload_button(ui);

        if ui.button("Write Check File").clicked() {
            let items = self.items.clone();
            let plan_name = self.root.clone().unwrap_or_default();
            plaine::write::write_check_file(items, plan_name).expect("File to write");
        };

        if ui.button("Write Upload File").clicked() {
            let unselected = self.unselected.clone();
            let selected_items = self.items.remove_fnskus(unselected.into_iter());

            let plan_name = self.root.clone().unwrap_or_default();
            if let Ok(response) = plaine::write::write_upload_txt(selected_items, plan_name) {
                let (branch, items) = response.take();
                self.branch_pending_name = Some(branch);
                self.branch_pending_items = items;
            };
        };

        if ui.button("branch").clicked() {
            self.branch_selected_items(&gen_pw()).unwrap();
        };

        Grid::new("item-grid").striped(true).show(ui, |ui| {
            self.fill_grid(ui);
        });
    }

    /// Adjust the [`Root`] that is currently attached to [`Self`].
    ///
    /// This function is also called by [`branch`]. The two differ in that
    /// branch will adjust the current root downwards, and then adjust
    /// a new branch upwards.
    ///
    /// Neither this function, or branch will adjust the in-memory entries
    /// until the serialization and writing returns without error.
    ///
    /// # Errors
    ///
    /// This function will not force a write to the file system in any way.
    /// If the given `path` cannot be written to, this will return an error.
    ///
    /// Additionally, serialization can fail prior to a write occurring, this
    /// will return an error as well.
    ///
    fn adjust_root(&mut self, adjustments: Vec<Entry>) -> Result<()> {
        // We need to know where to save out files.
        let path = self.config.local_dir();

        match &self.root {
            Some(root) => adjustments.serialize_and_write(root, None, path)?,
            None => bail!("Adjustments failed write"),
        };
        Ok(())
    }

    fn google_drive_upload_button(&mut self, ui: &mut Ui) {
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
        let path = self.config.local_dir();
        let trunk = gen_pw();
        items.serialize_and_write(&trunk, None, path)?;
        self.items.clear();
        self.root = Some(trunk);
        Ok(items)
    }

    /// Create a new branch from the selected items in [`Self`].
    ///
    /// As branching is a zero sum action, calling this method implicitly
    /// contains a negation of the items that are being branched.
    ///
    /// # Errors
    ///
    /// This functions may fail if the local path configurations are not
    /// set correctly. Serialization, and interactions with the file
    /// system may fail for the usual reasons as well.
    ///
    fn branch_selected_items(&mut self, branch: Brn) -> Result<()> {
        let branching_items = self.get_selected_items();
        let negated_items = branching_items.as_negated();
        let trunk = match &self.root {
            Some(ref name) => name,
            None => bail!("Can't branch without a trunk!"),
        };

        if let Some(local) = self.get_local_path() {
            branching_items.serialize_and_write(trunk, Some(branch), local)?;
        }
        self.adjust_root(negated_items)
    }

    fn get_local_path(&self) -> Option<&Path> {
        Some(&self.config.local_dir)
    }

    fn get_selected_items(&self) -> Vec<Entry> {
        let all_fnskus = self
            .items
            .iter()
            .map(|x| x.get_fnsku().to_string())
            .collect::<HashSet<_>>();
        let diff = all_fnskus.difference(&self.unselected).collect::<Vec<_>>();
        self.items
            .clone()
            .into_iter()
            .filter(|x| diff.iter().any(|y| x.get_fnsku() == y.as_str()))
            .collect::<Vec<_>>()
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
