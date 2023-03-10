use crate::{
    plan::{Entry, Plan},
    Brn,
};
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

pub fn write_check_file(entry_vec: Vec<Entry>, plan_name: String) -> std::io::Result<()> {
    let mut contents = String::new();
    let name = format!("{plan_name}\n");

    let hdr = "ASIN,TITLE,UNITS,SIZE,FNSKU,UPC,COUNT,NOTES\n".to_string();
    contents.push_str(&name);
    contents.push_str(&hdr);
    for entry in entry_vec.get_as_sums() {
        // Is the fnsku in the selected set?

        let amz_size = entry.get_amz_size().clone().unwrap_or_default();
        let fnsku = entry.get_fnsku();
        let asin = entry.get_asin().clone().unwrap_or_default();
        let title = entry.get_title().clone().unwrap_or_default();
        let units = entry.get_units().to_string();
        let row = format!("\"{asin}\",\"{title}\",\"{units}\",\"{amz_size}\",\"{fnsku}\",'\n");
        contents.push_str(&row);
    }
    let path = PathBuf::from(format!("{plan_name}-CheckFile.csv"));
    std::fs::write(path, contents)
}
pub fn write_upload_txt(entry_vec: Vec<Entry>, brn: Brn) -> Result<()> {
    let mut header = std::fs::read_to_string(".local/upload.txt")?;
    let predicate = header.clone();

    let entry_w_msku: Vec<_> = entry_vec
        .into_iter()
        .filter(|x| x.get_msku().is_some())
        .collect();

    for entry in entry_w_msku.get_as_sums() {
        match entry.get_msku() {
            Some(msku) => {
                let units = entry.get_units();
                let row = format!("{msku}\t{units}\tSeller\tSeller\n");
                header.push_str(&row);
            }
            None => continue,
        };
    }
    if header == predicate {
        return Err(anyhow!("Upload File Empty"));
    }
    let path = PathBuf::from(format!("{brn}-Upload.txt"));
    std::fs::write(path, header).context("fs::write failed")
}
