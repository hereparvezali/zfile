use gtk::prelude::*;
use gtk::{Label, ListBox, Orientation};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Clone)]
pub struct Sidebar {
    widget: gtk::Box,
    list_box: ListBox,
    on_location_activated: Rc<RefCell<Option<Rc<dyn Fn(PathBuf)>>>>,
    locations: Rc<RefCell<Vec<PathBuf>>>,
}

impl Sidebar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(Orientation::Vertical, 0);

        // Places section
        let places_label = Label::new(Some("Places"));
        places_label.set_halign(gtk::Align::Start);
        places_label.set_margin_start(12);
        places_label.set_margin_end(12);
        places_label.set_margin_top(12);
        places_label.set_margin_bottom(6);
        places_label.add_css_class("heading");
        widget.append(&places_label);

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.add_css_class("navigation-sidebar");

        widget.append(&list_box);

        let sidebar = Sidebar {
            widget,
            list_box,
            on_location_activated: Rc::new(RefCell::new(None)),
            locations: Rc::new(RefCell::new(Vec::new())),
        };

        sidebar.populate_places();
        sidebar.setup_activation();

        sidebar
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    fn populate_places(&self) {
        let mut locations = self.locations.borrow_mut();

        // Home
        if let Some(home_dir) = dirs::home_dir() {
            locations.push(home_dir.clone());
            let row = self.create_place_row("Home", "user-home-symbolic");
            self.list_box.append(&row);
        }

        // Desktop
        if let Some(home_dir) = dirs::home_dir() {
            let desktop_dir = home_dir.join("Desktop");
            if desktop_dir.exists() {
                locations.push(desktop_dir.clone());
                let row = self.create_place_row("Desktop", "user-desktop-symbolic");
                self.list_box.append(&row);
            }
        }

        // Documents
        if let Some(documents_dir) = dirs::document_dir() {
            locations.push(documents_dir.clone());
            let row = self.create_place_row("Documents", "folder-documents-symbolic");
            self.list_box.append(&row);
        }

        // Downloads
        if let Some(downloads_dir) = dirs::download_dir() {
            locations.push(downloads_dir.clone());
            let row = self.create_place_row("Downloads", "folder-download-symbolic");
            self.list_box.append(&row);
        }

        // Music
        if let Some(music_dir) = dirs::audio_dir() {
            locations.push(music_dir.clone());
            let row = self.create_place_row("Music", "folder-music-symbolic");
            self.list_box.append(&row);
        }

        // Pictures
        if let Some(pictures_dir) = dirs::picture_dir() {
            locations.push(pictures_dir.clone());
            let row = self.create_place_row("Pictures", "folder-pictures-symbolic");
            self.list_box.append(&row);
        }

        // Videos
        if let Some(videos_dir) = dirs::video_dir() {
            locations.push(videos_dir.clone());
            let row = self.create_place_row("Videos", "folder-videos-symbolic");
            self.list_box.append(&row);
        }

        // Root
        let root_dir = PathBuf::from("/");
        locations.push(root_dir.clone());
        let row = self.create_place_row("Root", "drive-harddisk-symbolic");
        self.list_box.append(&row);
    }

    fn create_place_row(&self, name: &str, icon_name: &str) -> gtk::Box {
        let row = gtk::Box::new(Orientation::Horizontal, 12);
        row.set_margin_start(12);
        row.set_margin_end(12);
        row.set_margin_top(6);
        row.set_margin_bottom(6);

        let icon = gtk::Image::from_icon_name(icon_name);
        icon.set_pixel_size(16);
        row.append(&icon);

        let label = Label::new(Some(name));
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        row.append(&label);

        row
    }

    fn setup_activation(&self) {
        let on_location_activated = self.on_location_activated.clone();
        let locations = self.locations.clone();
        self.list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            if let Some(path) = locations.borrow().get(index) {
                if let Some(callback) = on_location_activated.borrow().as_ref() {
                    callback(path.clone());
                }
            }
        });
    }

    pub fn connect_location_activated<F>(&self, callback: F)
    where
        F: Fn(PathBuf) + 'static,
    {
        *self.on_location_activated.borrow_mut() = Some(Rc::new(callback));
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}
