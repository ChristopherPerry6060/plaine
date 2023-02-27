#![windows_subsystem = "windows"]
#![allow(dead_code, unused_imports)]

const CHECKDIR: &'static str = ".local/CHECK/";
const STATUSDIR: &'static str = ".local/STATUS/";
const LOCALDIR: &'static str = ".local/";

mod instruct;
use anyhow::{anyhow, bail, Result};
use eframe::{
    egui::{self, CentralPanel, Grid, SidePanel, Ui, Window},
    NativeOptions,
};
use plaine::{
    plan::{
        status::{self, Status},
        Entry, Plan,
    },
    read::GDrivePlan,
    utils::{self, gen_pw, TrunkFileName},
    Branch, Brn, Fnsku,
};
use rfd::FileDialog;
use std::{
    collections::{HashMap, HashSet},
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
        SidePanel::left("branch-panel").show(ctx, |ui| self.show_branch_list(ui));
        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Reset App").clicked() {
                *self = Gui::default();
            };
            self.main_window(ui);
        });
    }
}

#[derive(Default, Debug)]
struct CheckEntry {
    fnsku: String,
    upc: String,
    units: u32,
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    check_memory: Vec<Entry>,
    check_due_memory: Vec<Entry>,
    check_entry_state: CheckEntry,
    in_action: bool,
    config: GuiConfig,
    items: Vec<Entry>,
    branch: Option<String>,
    branch_list: Vec<TrunkFileName>,
    branch_statuses: HashMap<String, Status>,
    unselected: HashSet<String>,
}

impl Gui {
    /// Refresh the following.
    ///
    /// * Branch List.
    /// * Selected Branch.
    /// * Unselected Items.
    /// * Branch statuses
    fn refresh(&mut self) {
        if let Ok(trunks) = dbg!(utils::gather_records(LOCALDIR)) {
            self.branch_list = trunks;
        };
        let root = self.branch.clone().unwrap_or_default();
        self.unselected = HashSet::default();

        self.branch_list
            .clone()
            .into_iter()
            .collect::<HashSet<String>>()
            .iter();

        let dir = std::fs::read_dir(STATUSDIR).expect("Read Statuses");

        let reads = dir.filter_map(|dir| dir.ok());
        let helper = reads.map(|x| (x.path(), x.file_name()));
        let map_helper = helper.filter_map(|(p, name)| {
            let str = std::fs::read_to_string(p).ok()?;
            let status: Status = serde_json::from_str(&str).ok()?;
            let (branch, _) = name.to_str()?.split_once('_')?;
            Some((branch.to_string(), status))
        });
        let mut hm: HashMap<String, Status> = HashMap::default();
        map_helper.for_each(|(bn, status)| {
            hm.insert(bn, status);
        });
        self.branch_statuses = hm;
        self.load_branch(&root);
    }

    /// Shows the branch list in the given UI.
    fn show_branch_list(&mut self, ui: &mut Ui) {
        if ui.button("Refresh").clicked() {
            self.refresh();
        };
        let branchs = self.branch_list.clone();

        // Get the branches from memory
        let uniques = branchs
            .iter()
            .filter_map(|b| b.split_once('_'))
            .map(|(n, _)| n)
            .collect::<HashSet<_>>();

        let mut vex = uniques.clone().into_iter().collect::<Vec<_>>();
        vex.sort();

        let prepped_pairs = uniques
            .into_iter()
            .filter_map(|name| {
                let status = self.branch_statuses.get(name)?;
                Some((name.to_owned(), status.to_string()))
            })
            .collect::<Vec<_>>();

        Grid::new("branch-list").num_columns(5).show(ui, |ui| {
            ui.label("Name");
            ui.label("Status");
            ui.label("Select");
            ui.end_row();
            prepped_pairs.into_iter().for_each(|(name, status)| {
                ui.label(&name);
                ui.label(status);
                if ui.small_button("Select").clicked() {
                    // This is not a great place to do this.
                    self.load_branch(&name);
                };
                ui.end_row();
            });
        });
    }

    /// Update the current branch, and load it from the branch dir.
    fn load_branch(&mut self, brn: Brn) {
        let eq_brn_files = self.branch_list.iter().filter(|x| match x.split_once('_') {
            Some((prefix, _suffix)) => prefix.eq(brn),
            _ => false,
        });

        self.branch = Some(brn.to_string());
        self.items = eq_brn_files
            .flat_map(|filename| {
                let path = format!("{LOCALDIR}{filename}");
                let entry = std::fs::read_to_string(&path).ok()?;
                serde_json::from_str::<Vec<Entry>>(&entry).ok()
            })
            .flatten()
            .collect();
    }

    /// Return a default instance.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn main_window(&mut self, ui: &mut Ui) {
        self.legacy_button(ui);
        self.actions(ui);
    }

    fn actions(&mut self, ui: &mut Ui) {
        let cheat = self.branch.clone();
        let cheat2 = self.branch_statuses.clone();
        let Some(brn) =cheat else {
            return;
        };
        let branch = brn.as_str();
        if let Some(status) = cheat2.get(branch) {
            let text = match status {
                Status::Open => "Add Item",
                Status::Check => "Start Check",
                Status::Confirm => "Confirm Check",
                Status::Measure => "Add Missing Dims",
                Status::BoxContents => "Generate Box Contents",
                Status::CaseLabel => "Generate Label Helper",
                Status::Staged => "Add Shipment Info",
                Status::Shipped => "Ship and Hide",
            };
            if self.in_action {
                self.run_action(brn.to_owned(), status, ui);
            };

            if !self.in_action {
                if ui.button(text).clicked() {
                    self.in_action = true;
                };
                self.show_current_branch_contents(ui);
            }
        };
    }
    fn run_action(&mut self, branch: Branch, status: &Status, ui: &mut Ui) {
        if matches!(status, Status::Open) {
            self.branch_selected_items(&branch);
            Status::mark_for_check(STATUSDIR, &branch).expect("Mark as Checked");
            self.in_action = false;
        };
        if matches!(status, Status::Check) {
            let mut memory = self.check_memory.get_as_sums();
            memory.sort_unstable_by_key(|x| x.get_fnsku().to_string());
            let ce = &mut self.check_entry_state;
            ui.text_edit_singleline(&mut ce.fnsku);
            ui.text_edit_singleline(&mut ce.upc);
            ui.add(egui::DragValue::new(&mut ce.units));
            if ui.button("Add").clicked() {
                let mut entry = Entry::default();
                let upc = ce.upc.clone();
                let fnsku = ce.fnsku.clone();

                ce.fnsku.clear();
                ce.upc.clear();
                entry.set_fnsku(fnsku);

                if upc.is_empty() {
                    entry.set_upc(None);
                } else {
                    entry.set_upc(Some(upc));
                };
                entry.set_units(ce.units as i32);
                self.check_memory.push(entry);
            };

            Grid::new("check-file").show(ui, |ui| {
                ui.label("Scanned Fnskus");
                ui.label("Units Checked");
                ui.end_row();
                memory.into_iter().for_each(|x| {
                    ui.label(x.get_fnsku());
                    ui.label(x.get_units().to_string());
                    ui.end_row();
                });
            });
        };
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

        match &self.branch {
            Some(root) => adjustments.serialize_and_write(root, None, LOCALDIR)?,
            None => bail!("Adjustments failed write"),
        };
        self.refresh();
        Ok(())
    }

    /// Show a file dialog so a google drive sheet can be uploaded.
    fn legacy_button(&mut self, ui: &mut Ui) {
        if !ui.button("Upload").clicked() {
            return;
        };
        let Some(file_picker) = FileDialog::new().pick_file() else {
            return;
        };
        let Ok(items) = GDrivePlan::proc_from_path(file_picker) else {
            return;
        };
        let trunk = gen_pw();
        if let Ok(_) = items.serialize_and_write(&trunk, None, LOCALDIR) {
            self.load_branch(&trunk);
            Status::mark_as_open(STATUSDIR, &trunk).expect("Mark as open.");
        };
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
        let root = match &self.branch {
            Some(ref name) => name,
            None => bail!("Can't branch without a root!"),
        };

        branching_items.serialize_and_write(root, Some(branch), LOCALDIR)?;
        self.adjust_root(negated_items)
    }

    /// Return the currently selected items being shown in the gui.
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
    fn show_current_branch_contents(&mut self, ui: &mut Ui) {
        Grid::new("item-grid").striped(true).show(ui, |ui| {
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
        });
    }
}
