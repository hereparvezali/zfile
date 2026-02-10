use gtk::prelude::*;
use gtk::{
    Box, ComboBoxText, Dialog, DialogFlags, Label, Orientation, ResponseType, Switch, Window,
};

pub fn show_preferences_dialog(parent: &impl IsA<Window>) {
    let dialog = Dialog::with_buttons(
        Some("Preferences"),
        Some(parent),
        DialogFlags::MODAL | DialogFlags::USE_HEADER_BAR,
        &[("Close", ResponseType::Close)],
    );

    dialog.set_default_size(500, 400);

    let content_area = dialog.content_area();
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);
    content_area.set_spacing(12);

    // View preferences
    let view_section = create_section("View");
    content_area.append(&view_section);

    let show_hidden_row = create_preference_row(
        "Show hidden files",
        "Display files and folders that start with a dot",
    );
    let show_hidden_switch = Switch::new();
    show_hidden_switch.set_valign(gtk::Align::Center);
    show_hidden_row.append(&show_hidden_switch);
    view_section.append(&show_hidden_row);

    let default_view_row =
        create_preference_row("Default view", "Choose the default view mode for folders");
    let default_view_combo = ComboBoxText::new();
    default_view_combo.append_text("Grid View");
    default_view_combo.append_text("List View");
    default_view_combo.set_active(Some(0));
    default_view_combo.set_valign(gtk::Align::Center);
    default_view_row.append(&default_view_combo);
    view_section.append(&default_view_row);

    // Behavior preferences
    let behavior_section = create_section("Behavior");
    content_area.append(&behavior_section);

    let single_click_row = create_preference_row(
        "Single click to open",
        "Open files and folders with a single click",
    );
    let single_click_switch = Switch::new();
    single_click_switch.set_valign(gtk::Align::Center);
    single_click_row.append(&single_click_switch);
    behavior_section.append(&single_click_row);

    let confirm_delete_row = create_preference_row(
        "Confirm before delete",
        "Show confirmation dialog before deleting files",
    );
    let confirm_delete_switch = Switch::new();
    confirm_delete_switch.set_active(true);
    confirm_delete_switch.set_valign(gtk::Align::Center);
    confirm_delete_row.append(&confirm_delete_switch);
    behavior_section.append(&confirm_delete_row);

    // Performance preferences
    let performance_section = create_section("Performance");
    content_area.append(&performance_section);

    let thumbnail_row = create_preference_row(
        "Show thumbnails",
        "Generate and display thumbnails for image files",
    );
    let thumbnail_switch = Switch::new();
    thumbnail_switch.set_active(true);
    thumbnail_switch.set_valign(gtk::Align::Center);
    thumbnail_row.append(&thumbnail_switch);
    performance_section.append(&thumbnail_row);

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.present();
}

fn create_section(title: &str) -> Box {
    let section = Box::new(Orientation::Vertical, 6);

    let title_label = Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("title-4");
    section.append(&title_label);

    section
}

fn create_preference_row(title: &str, description: &str) -> Box {
    let row = Box::new(Orientation::Horizontal, 12);
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(6);
    row.set_margin_bottom(6);

    let text_box = Box::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    text_box.append(&title_label);

    let desc_label = Label::new(Some(description));
    desc_label.set_halign(gtk::Align::Start);
    desc_label.add_css_class("dim-label");
    desc_label.add_css_class("caption");
    text_box.append(&desc_label);

    row.append(&text_box);

    row
}
