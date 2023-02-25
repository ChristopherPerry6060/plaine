use eframe::egui::Ui;

/// Display instructions within the given `ui`.
pub fn instruction_window(ui: &mut Ui) {
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
