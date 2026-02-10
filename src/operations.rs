use gtk::prelude::*;
use gtk::Window;
use std::fs;
use std::path::Path;

pub fn show_new_folder_dialog(parent: &impl IsA<Window>, current_path: &Path) {
    let dialog = gtk::Dialog::with_buttons(
        Some("New Folder"),
        Some(parent),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::USE_HEADER_BAR,
        &[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Create", gtk::ResponseType::Accept),
        ],
    );

    let content_area = dialog.content_area();
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);

    let label = gtk::Label::new(Some("Folder name:"));
    label.set_halign(gtk::Align::Start);
    content_area.append(&label);

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("New Folder"));
    entry.set_activates_default(true);
    content_area.append(&entry);

    dialog.set_default_response(gtk::ResponseType::Accept);

    let current_path = current_path.to_path_buf();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let folder_name = entry.text();
            if !folder_name.is_empty() {
                let new_folder_path = current_path.join(folder_name.as_str());
                if let Err(e) = fs::create_dir(&new_folder_path) {
                    eprintln!("Failed to create folder: {}", e);
                    show_error_dialog(dialog, "Failed to create folder", &e.to_string());
                    return;
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

pub fn show_new_file_dialog(parent: &impl IsA<Window>, current_path: &Path) {
    let dialog = gtk::Dialog::with_buttons(
        Some("New File"),
        Some(parent),
        gtk::DialogFlags::MODAL | gtk::DialogFlags::USE_HEADER_BAR,
        &[
            ("Cancel", gtk::ResponseType::Cancel),
            ("Create", gtk::ResponseType::Accept),
        ],
    );

    let content_area = dialog.content_area();
    content_area.set_margin_start(12);
    content_area.set_margin_end(12);
    content_area.set_margin_top(12);
    content_area.set_margin_bottom(12);

    let label = gtk::Label::new(Some("File name:"));
    label.set_halign(gtk::Align::Start);
    content_area.append(&label);

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("New File.txt"));
    entry.set_activates_default(true);
    content_area.append(&entry);

    dialog.set_default_response(gtk::ResponseType::Accept);

    let current_path = current_path.to_path_buf();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let file_name = entry.text();
            if !file_name.is_empty() {
                let new_file_path = current_path.join(file_name.as_str());
                if let Err(e) = fs::File::create(&new_file_path) {
                    eprintln!("Failed to create file: {}", e);
                    show_error_dialog(dialog, "Failed to create file", &e.to_string());
                    return;
                }
            }
        }
        dialog.close();
    });

    dialog.present();
}

#[allow(unused)]
pub fn show_delete_confirmation(parent: &impl IsA<Window>, paths: &[std::path::PathBuf]) -> bool {
    let count = paths.len();
    let message = if count == 1 {
        format!(
            "Are you sure you want to delete '{}'?",
            paths[0].file_name().unwrap_or_default().to_string_lossy()
        )
    } else {
        format!("Are you sure you want to delete {} items?", count)
    };

    let dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Warning,
        gtk::ButtonsType::OkCancel,
        &message,
    );

    dialog.set_secondary_text(Some("This action cannot be undone."));

    let _response = dialog.run_future();
    dialog.close();

    false // Placeholder - would need async handling
}

#[allow(unused)]
pub fn delete_files(paths: &[std::path::PathBuf]) -> Result<(), String> {
    for path in paths {
        if let Err(e) = trash::delete(path) {
            return Err(format!("Failed to delete {}: {}", path.display(), e));
        }
    }
    Ok(())
}

#[allow(unused)]
pub fn copy_files(sources: &[std::path::PathBuf], destination: &Path) -> Result<(), String> {
    for source in sources {
        let file_name = source.file_name().ok_or("Invalid source path")?;
        let dest_path = destination.join(file_name);

        if source.is_dir() {
            if let Err(e) = copy_dir_recursive(source, &dest_path) {
                return Err(format!("Failed to copy directory: {}", e));
            }
        } else {
            if let Err(e) = fs::copy(source, &dest_path) {
                return Err(format!("Failed to copy file: {}", e));
            }
        }
    }
    Ok(())
}

#[allow(unused)]
pub fn move_files(sources: &[std::path::PathBuf], destination: &Path) -> Result<(), String> {
    for source in sources {
        let file_name = source.file_name().ok_or("Invalid source path")?;
        let dest_path = destination.join(file_name);

        if let Err(e) = fs::rename(source, &dest_path) {
            return Err(format!("Failed to move file: {}", e));
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn show_error_dialog(parent: &impl IsA<Window>, title: &str, message: &str) {
    let dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        title,
    );
    dialog.set_secondary_text(Some(message));
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}
