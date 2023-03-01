#![windows_subsystem = "windows"]

const CHECKDIR: &str = ".local/CHECK/";
const STATUSDIR: &str = ".local/STATUS/";
const LOCALDIR: &str = ".local/";
const BOXCONTENTS: &str = "FlatBoxContents/";
const CONFIRMATION: &str = "Confirmations/";

const AMZ_STA_LINK: &str = "https://sellercentral.amazon.com/fba/sendtoamazon/";
use anyhow::{anyhow, bail, Result};
use eframe::{
    egui::{self, CentralPanel, Grid, ScrollArea, SidePanel, Ui},
    NativeOptions,
};
use plaine::{
    plan::{status::Status, Entry, Plan},
    read::GDrivePlan,
    utils::{self, gen_pw, TrunkFileName},
    write, Branch, Brn,
};
use rfd::FileDialog;
use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
};

fn main() {
    let native_options = NativeOptions {
        follow_system_theme: false,
        default_theme: eframe::Theme::Light,
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
            ui.label("v0.0.02");
            ui.hyperlink_to("Amz Upload Files Go Here!", AMZ_STA_LINK);
            if ui.button("Reset App").clicked() {
                *self = Gui::default();
            };
            self.main_window(ui);
        });
    }
}

impl TryFrom<CheckEntry> for Vec<Entry> {
    type Error = anyhow::Error;

    fn try_from(value: CheckEntry) -> std::result::Result<Self, Self::Error> {
        let fnsku = value.fnsku;
        let upc = value.upc;
        let units_per_case = value.units_per_case;
        let total_cases = value.cases;
        let mut entry = Entry::default();
        if fnsku.is_empty() {
            bail!("Fnsku cannot be empty.");
        };
        if units_per_case.eq(&0) {
            bail!("Units per case cannot be zero.");
        };
        if total_cases.eq(&0) {
            bail!("Total cases cannot be zero.");
        };

        let cleaned_units: i32 = units_per_case.try_into()?;
        let cleaned_upc = (!upc.is_empty()).then_some(upc);
        entry.set_upc(cleaned_upc);
        entry.set_fnsku(fnsku);
        entry.set_units(cleaned_units);
        let plan = (0..total_cases).map(|_| entry.clone()).collect();
        Ok(plan)
    }
}
#[derive(Default, Debug, Clone)]
struct CheckEntry {
    fnsku: String,
    upc: String,
    units_per_case: u32,
    cases: u32,
}

/// The Plaine application data.
#[derive(Default, Debug)]
pub struct Gui {
    moved_branch_name: Option<Branch>,
    check_memory: Vec<Entry>,
    check_entry_state: CheckEntry,
    check_entry_error: Option<anyhow::Error>,
    in_check: bool,
    items: Vec<Entry>,
    current_branch: Option<String>,
    branch_list: Vec<TrunkFileName>,
    branch_statuses: HashMap<String, Status>,
    unselected: HashSet<String>,
    error_stack: Vec<anyhow::Error>,
}

impl Gui {
    /// Refresh the following.
    ///
    /// * Branch List.
    /// * Selected Branch.
    /// * Unselected Items.
    /// * Branch statuses
    fn refresh(&mut self) {
        if let Ok(trunks) = utils::gather_records(LOCALDIR) {
            self.branch_list = trunks;
        };

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
            let str = read_to_string(p).ok()?;
            let status: Status = serde_json::from_str(&str).ok()?;
            let (branch, _) = name.to_str()?.split_once('_')?;

            Some((branch.to_string(), status))
        });
        let mut hm: HashMap<String, Status> = HashMap::default();
        let mut sorted = map_helper.collect::<Vec<_>>();

        sorted.sort();
        sorted.into_iter().for_each(|(bn, status)| {
            hm.insert(bn, status);
        });
        if let Some(ref branch) = &self.current_branch.clone() {
            self.switch_to_branch(branch);
        };
        self.branch_statuses = hm;
        self.check_memory = Vec::default();
        self.check_entry_state = CheckEntry::default();
        self.in_check = false;
        self.moved_branch_name = None;
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
        let istatus = self.branch_statuses.clone();
        let prepped_pairs = vex
            .into_iter()
            .map(|name| {
                let status = match istatus.get(name) {
                    Some(x) => x.to_owned(),
                    None => Status::default(),
                };
                (name.to_string(), status.to_string())
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
                    self.refresh();
                    self.switch_to_branch(&name);
                };
                ui.end_row();
            });
        });
    }

    /// Update the current branch, and load it from the branch dir.
    fn switch_to_branch(&mut self, brn: Brn) {
        // Remove client from action state whenever switching

        self.in_check = false;
        let current_branch_list = &mut self.branch_list;
        let incoming_branch = current_branch_list.iter().filter(|x| {
            let Some((json_file_prefix, _)) = x.split_once('_') else {
                return false;
            };
            json_file_prefix.eq(brn)
        });

        // Set the current branch in self.
        self.current_branch = Some(brn.to_string());
        let file_names = incoming_branch.map(|filename| format!("{LOCALDIR}{filename}"));

        let deser_entries = file_names
            .filter_map(|path| {
                let entry = read_to_string(path).ok()?;
                serde_json::from_str::<Vec<Entry>>(&entry).ok()
            })
            .flatten()
            .collect();

        self.items = deser_entries;
    }

    /// Return a default instance.
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn main_window(&mut self, ui: &mut Ui) {
        let flate = "Gen Flat Box Contents";
        if self.current_branch.is_some() && ui.button(flate).clicked() {
            if let Err(err) = self.make_box_contents_fil() {
                self.error_stack.push(err);
            };
        };

        if ui.button("Gen Confirmation File").clicked() {
            if let Err(err) = self.output_confirmation_file() {
                self.error_stack.push(err);
            };
        };
        if ui.button("Upload Google Drive Plan").clicked() {
            match self.legacy_button() {
                Err(err) => self.error_stack.push(err),
                Ok(branch) => {
                    self.switch_to_branch(&branch);
                    self.refresh();
                }
            };
        };
        self.actions(ui);
    }

    fn actions(&mut self, ui: &mut Ui) -> Option<()> {
        let current_branch = &self.current_branch.clone()?;
        let status_map = self.branch_statuses.clone();
        let current_status = status_map.get(current_branch)?;

        ui.separator();
        ui.label("You are on:");
        ui.label(current_branch);
        ui.separator();

        if let Some(recent_moved_branch) = &self.moved_branch_name {
            ui.separator();
            ui.label("You recently moved items to:");
            ui.label(recent_moved_branch);
            ui.separator();
        }

        if matches!(current_status, Status::Open) {
            let make_upload_file = ui.button("Make Upload File");
            let remove_unselected = ui.button("Move unselected to new branch");
            let mark_for_checking = ui.button("Mark CURRENT branch for checking");

            if make_upload_file.clicked() {
                let items = self.items.clone();
                write::write_upload_txt(items, current_branch).expect("Upload File Write");
            };
            if remove_unselected.clicked() {
                let new_branch_name = self.new_branch_from_unselected().ok()?;
                self.moved_branch_name = Some(new_branch_name);
            };
            if mark_for_checking.clicked() {
                Status::mark(&Status::Check, STATUSDIR, current_branch).ok()?;
                self.refresh()
            };
            self.show_current_branch_contents(ui);
        };
        // CLOSED SHIPMENT
        if matches!(current_status, Status::Check) && !self.in_check {
            if ui.button("Start Check").clicked() {
                self.prep_check().ok()?;
            };
        };
        if self.in_check {
            self.run_check(current_branch.to_owned(), ui).ok()?;
        };
        Some(())
    }

    fn output_confirmation_file(&mut self) -> Result<()> {
        let our_branch = self
            .current_branch
            .as_ref()
            .ok_or(anyhow!("Not on a branch"))?;

        let check_dir = std::fs::read_dir(CHECKDIR)?;
        let reads = check_dir.filter_map(|dir| dir.ok());

        let check_dir_files = reads.map(|x| (x.path(), x.file_name()));

        let our_checks = check_dir_files.filter_map(|(p, name)| {
            let str = read_to_string(p).ok()?;
            let check_entries: Vec<Entry> = serde_json::from_str(&str).ok()?;
            let (branch, _) = name.to_str()?.split_once('_')?;
            our_branch.eq(branch).then_some(check_entries)
        });

        our_checks.for_each(|entries| {
            self.check_memory.extend_from_slice(&entries);
        });
        let mut check = self.check_memory.clone();
        let negated_expectation = self.items.as_negated();
        check.extend_from_slice(&negated_expectation);
        let x = serde_json::to_string(&check)?;
        let path = format!("{CONFIRMATION}{our_branch}-Confirmation.json");
        Ok(std::fs::write(path, x)?)
    }
    /// Pulls the local check entries into memory
    fn prep_check(&mut self) -> Result<()> {
        let our_branch = self
            .current_branch
            .as_ref()
            .ok_or(anyhow!("Not on a branch"))?;

        let check_dir = std::fs::read_dir(CHECKDIR)?;
        let reads = check_dir.filter_map(|dir| dir.ok());

        let check_dir_files = reads.map(|x| (x.path(), x.file_name()));

        let our_checks = check_dir_files.filter_map(|(p, name)| {
            let str = read_to_string(p).ok()?;
            let check_entries: Vec<Entry> = serde_json::from_str(&str).ok()?;
            let (branch, _) = name.to_str()?.split_once('_')?;
            our_branch.eq(branch).then_some(check_entries)
        });

        our_checks.for_each(|entries| {
            self.check_memory.extend_from_slice(&entries);
        });
        self.in_check = true;
        Ok(())
    }

    /// Runs the check in the central panel.
    fn run_check(&mut self, branch: Branch, ui: &mut Ui) -> Result<()> {
        let mut memory = self.check_memory.get_as_sums();
        let input = &mut self.check_entry_state;

        // Short by fnsku so everything stays still in the ui.
        memory.sort_unstable_by_key(|x| x.get_fnsku().to_string());

        Grid::new("checking").num_columns(2).show(ui, |ui| {
            ui.label("Enter Fnsku");
            ui.text_edit_singleline(&mut input.fnsku);
            ui.end_row();

            ui.label("Enter Upc:");
            ui.text_edit_singleline(&mut input.upc);
            ui.end_row();

            ui.label("Units/Case");
            ui.add(egui::DragValue::new(&mut input.units_per_case));
            ui.end_row();

            ui.label("Number of Cases:");
            ui.add(egui::DragValue::new(&mut input.cases));
            ui.end_row();
        });

        ui.separator();
        if ui.button("Submit").clicked() {
            // FIXME: This could just be a method on the
            // CheckEntry struct
            //
            // Only clear current input when conversion to Entry
            // does not fail.
            //
            // submit_check_entry clones the current input from the reference.
            if let Err(err) = Gui::submit_check_entry(input, branch) {
                self.check_entry_error = Some(err);
            } else {
                *input = CheckEntry::default();
            };
        };
        if ui.button("Clear Fields").clicked() {
            *input = CheckEntry::default();
        };
        ui.separator();

        if let Some(error_message) = &self.check_entry_error {
            let err_string = error_message.to_string();
            ui.label(err_string);
            ui.separator();
        };

        Grid::new("check-file").show(ui, |ui| {
            ui.label("Fnsku");
            ui.label("Units Checked");
            ui.label("Title");
            ui.end_row();

            memory.into_iter().for_each(|prev| {
                let fnsku = prev.get_fnsku();
                let title = self
                    .items
                    .iter()
                    .find(|branch| branch.str_fnsku() == fnsku)
                    .map(|x| x.str_title())
                    .unwrap_or_default();

                ui.label(fnsku);
                ui.label(title);
                ui.label(prev.get_units().to_string());
                ui.end_row();
            });
        });

        Ok(())
    }

    /// Submit the `input` by cloning its current contents and writing to a file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * `input` fails it convesrion into [`Entry`] by call of `TryFrom`.
    /// * Serializing the result of the conversion fails.
    /// * Writing the serialized json to the filesystem fails (`Plan::serialize_and_write`).
    ///
    fn submit_check_entry(input: &CheckEntry, branch: String) -> Result<(), anyhow::Error> {
        let entry_as_plan = Vec::<Entry>::try_from(input.to_owned())?;
        entry_as_plan.serialize_and_write(&branch, CHECKDIR)?;
        Ok(())
    }

    /// Show a file dialog so a google drive sheet can be uploaded.
    fn legacy_button(&mut self) -> Result<Branch> {
        let picked_file = FileDialog::new()
            .pick_file()
            .ok_or_else(|| anyhow::anyhow!("Uploading Failed"))?;

        let items = GDrivePlan::proc_from_path(picked_file)?;
        let trunk = gen_pw();
        items.serialize_and_write(&trunk, LOCALDIR)?;
        Status::mark(&Status::Open, STATUSDIR, &trunk)?;
        Ok(trunk)
    }

    /// Split the unselected items off to a new brnach, marked as open.
    ///
    /// This function will return the new branch name for the items split.
    fn new_branch_from_unselected(&self) -> Result<Branch> {
        let branching_items = self.get_unselected_items();
        if branching_items.is_empty() {
            return Err(anyhow!("Everything is selected, nothing to branch"));
        };

        let branch = gen_pw();
        branching_items.serialize_and_write(&branch, LOCALDIR)?;
        Status::mark(&Status::Open, STATUSDIR, &branch)?;

        let our_branch = &self
            .current_branch
            .as_ref()
            .ok_or(anyhow!("Not on a branch."))?;

        // Write our negated items to the current branch
        let negated_items = branching_items.as_negated();
        negated_items.serialize_and_write(our_branch, LOCALDIR)?;

        // Return the NEW branch
        Ok(branch)
    }

    /// Returns a clone of all of the items that are currently not selected.
    ///
    /// You probably mean to use [`Self::new_branch_from_unselected`].
    fn get_unselected_items(&self) -> Vec<Entry> {
        let unsel = self.unselected.clone();
        self.items
            .iter()
            .filter_map(|x| {
                let fnsku = x.get_fnsku();
                unsel.contains(fnsku).then_some(x.clone())
            })
            .collect()
    }

    /// Fill the Ui with a grid, displaying sums of the passed entries.
    fn show_current_branch_contents(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
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
                    .filter(|entry| entry.get_units() > 0)
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
        });
    }

    /// This functions will write out a file for box contents at the path.
    ///
    /// Any entries that do not have mskus will not be added to the file.
    fn make_box_contents_fil(&self) -> anyhow::Result<()> {
        // This is probably terribly done, but it for sure works
        // Good enough for now
        let clone: Vec<Entry> = self
            .items
            .entries()
            .as_group_by_case()
            .into_values()
            .flatten()
            .collect();
        let mut with_msku: Vec<_> = clone
            .into_iter()
            .filter(|x| x.get_msku().is_some())
            .collect();

        with_msku.sort_by_key(|entry| {
            let dimensions = entry.get_case_dimensions().unwrap_or_default();
            let better_dims: Vec<u32> = dimensions.into_iter().map(|x| x as u32).collect();
            (
                entry.get_msku().to_owned(),
                entry.get_units().to_owned(),
                better_dims.to_owned(),
                entry.get_total_pounds().map(|x| x as u32),
            )
        });
        let loose_items_case_id: Vec<String> = self
            .items
            .multi_fnsku_cases()
            .into_iter()
            .map(|x| x.get_id().to_string())
            .collect();

        // Separate into loose & packed.
        // Im pretty sure there is a better way to concat to iterators together.
        let (loose, mut packed): (Vec<Entry>, Vec<Entry>) = with_msku.into_iter().partition(|x| {
            let case_id = x.get_id().to_string();
            loose_items_case_id.contains(&case_id)
        });

        let branch = self
            .current_branch
            .clone()
            .ok_or(anyhow!("Not on a branch"))?;
        packed.extend(loose.into_iter());
        let s = serde_json::to_string(&packed)?;
        let path = format!("{BOXCONTENTS}{branch}-FlatBoxContents.json");
        Ok(std::fs::write(path, s)?)
    }
}
