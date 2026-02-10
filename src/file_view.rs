use crate::file_info::FileInfo;
use gtk::prelude::*;
use gtk::{FlowBox, Label, ListBox, Orientation};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
}

#[derive(Clone)]
pub struct FileView {
    widget: gtk::Box,
    grid_view: FlowBox,
    list_view: ListBox,
    current_mode: Rc<RefCell<ViewMode>>,
    current_path: Rc<RefCell<PathBuf>>,
    files: Rc<RefCell<Vec<FileInfo>>>,
    on_activated: Rc<RefCell<Option<Rc<dyn Fn(PathBuf)>>>>,
}

impl FileView {
    pub fn new() -> Self {
        let widget = gtk::Box::new(Orientation::Vertical, 0);

        let grid_view = FlowBox::builder()
            .selection_mode(gtk::SelectionMode::Multiple)
            .column_spacing(12)
            .row_spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .homogeneous(true)
            .min_children_per_line(2)
            .max_children_per_line(10)
            .build();

        let list_view = ListBox::builder()
            .selection_mode(gtk::SelectionMode::Multiple)
            .build();

        widget.append(&grid_view);

        let file_view = FileView {
            widget,
            grid_view,
            list_view,
            current_mode: Rc::new(RefCell::new(ViewMode::Grid)),
            current_path: Rc::new(RefCell::new(PathBuf::from("/"))),
            files: Rc::new(RefCell::new(Vec::new())),
            on_activated: Rc::new(RefCell::new(None)),
        };

        file_view.setup_activation();

        file_view
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    pub fn set_view_mode(&self, mode: ViewMode) {
        let mut current_mode = self.current_mode.borrow_mut();
        if *current_mode != mode {
            *current_mode = mode;
            drop(current_mode);

            // Remove old view
            if let Some(child) = self.widget.first_child() {
                self.widget.remove(&child);
            }

            // Add new view
            match mode {
                ViewMode::Grid => self.widget.append(&self.grid_view),
                ViewMode::List => self.widget.append(&self.list_view),
            }

            // Reload with new view
            let path = self.current_path.borrow().clone();
            self.load_directory(&path);
        }
    }

    pub fn load_directory(&self, path: &Path) {
        *self.current_path.borrow_mut() = path.to_path_buf();

        // Clear existing items
        while let Some(child) = self.grid_view.first_child() {
            self.grid_view.remove(&child);
        }
        while let Some(child) = self.list_view.first_child() {
            self.list_view.remove(&child);
        }

        // Read directory
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read directory: {}", e);
                return;
            }
        };

        let mut files = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_info) = FileInfo::from_path(&entry.path()) {
                    files.push(file_info);
                }
            }
        }

        // Sort: directories first, then by name
        files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        *self.files.borrow_mut() = files.clone();

        let mode = *self.current_mode.borrow();
        match mode {
            ViewMode::Grid => self.populate_grid_view(&files),
            ViewMode::List => self.populate_list_view(&files),
        }
    }

    fn populate_grid_view(&self, files: &[FileInfo]) {
        for file in files {
            let item_box = gtk::Box::new(Orientation::Vertical, 6);
            item_box.set_margin_start(6);
            item_box.set_margin_end(6);
            item_box.set_margin_top(6);
            item_box.set_margin_bottom(6);

            let icon = gtk::Image::from_icon_name(file.icon_name());
            icon.set_pixel_size(48);
            item_box.append(&icon);

            let label = Label::new(Some(&file.name));
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(20);
            item_box.append(&label);

            // Store path as a property using glib
            let path_str = file.path.to_string_lossy().to_string();
            item_box.set_property("tooltip-text", &path_str);

            self.grid_view.insert(&item_box, -1);
        }
    }

    fn populate_list_view(&self, files: &[FileInfo]) {
        for file in files {
            let row_box = gtk::Box::new(Orientation::Horizontal, 12);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(6);
            row_box.set_margin_bottom(6);

            let icon = gtk::Image::from_icon_name(file.icon_name());
            icon.set_pixel_size(24);
            row_box.append(&icon);

            let name_label = Label::new(Some(&file.name));
            name_label.set_halign(gtk::Align::Start);
            name_label.set_hexpand(true);
            row_box.append(&name_label);

            let size_label = Label::new(Some(&file.format_size()));
            size_label.set_width_chars(12);
            size_label.add_css_class("dim-label");
            row_box.append(&size_label);

            let date_label = Label::new(Some(&file.format_modified()));
            date_label.set_width_chars(20);
            date_label.add_css_class("dim-label");
            row_box.append(&date_label);

            // Store path as a property using glib
            let path_str = file.path.to_string_lossy().to_string();
            row_box.set_property("tooltip-text", &path_str);

            self.list_view.append(&row_box);
        }
    }

    fn setup_activation(&self) {
        // Grid view activation (double-click)
        let on_activated = self.on_activated.clone();
        self.grid_view.connect_child_activated(move |_, child| {
            // Get path from tooltip-text property
            if let Some(path_str) = child.property::<Option<String>>("tooltip-text") {
                if !path_str.is_empty() {
                    let path = PathBuf::from(path_str);
                    if let Some(callback) = on_activated.borrow().as_ref() {
                        callback(path);
                    }
                }
            }
        });

        // List view activation (double-click)
        let on_activated = self.on_activated.clone();
        self.list_view.connect_row_activated(move |_, row| {
            // Get path from tooltip-text property of the row's child
            if let Some(child) = row.child() {
                if let Some(path_str) = child.property::<Option<String>>("tooltip-text") {
                    if !path_str.is_empty() {
                        let path = PathBuf::from(path_str);
                        if let Some(callback) = on_activated.borrow().as_ref() {
                            callback(path);
                        }
                    }
                }
            }
        });
    }

    pub fn connect_activated<F>(&self, callback: F)
    where
        F: Fn(PathBuf) + 'static,
    {
        *self.on_activated.borrow_mut() = Some(Rc::new(callback));
    }

    pub fn item_count(&self) -> usize {
        self.files.borrow().len()
    }
}
