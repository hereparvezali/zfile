use gtk::prelude::*;
use gtk::{Orientation, SearchEntry};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct SearchBar {
    widget: gtk::Box,
    search_entry: SearchEntry,
    on_search: Rc<RefCell<Option<Rc<dyn Fn(String)>>>>,
}

impl SearchBar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(Orientation::Horizontal, 6);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_margin_top(6);
        widget.set_margin_bottom(6);
        widget.set_visible(false);

        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some("Search files..."));
        search_entry.set_hexpand(true);

        widget.append(&search_entry);

        let search_bar = SearchBar {
            widget,
            search_entry,
            on_search: Rc::new(RefCell::new(None)),
        };

        search_bar.setup_signals();

        search_bar
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    pub fn set_visible(&self, visible: bool) {
        self.widget.set_visible(visible);
        if visible {
            self.search_entry.grab_focus();
        } else {
            self.search_entry.set_text("");
        }
    }

    pub fn grab_focus(&self) {
        self.search_entry.grab_focus();
    }

    fn setup_signals(&self) {
        let on_search = self.on_search.clone();
        self.search_entry.connect_search_changed(move |entry| {
            let text = entry.text().to_string();
            if let Some(callback) = on_search.borrow().as_ref() {
                callback(text);
            }
        });

        let on_search = self.on_search.clone();
        self.search_entry.connect_activate(move |entry| {
            let text = entry.text().to_string();
            if let Some(callback) = on_search.borrow().as_ref() {
                callback(text);
            }
        });
    }

    #[allow(unused)]
    pub fn connect_search<F>(&self, callback: F)
    where
        F: Fn(String) + 'static,
    {
        *self.on_search.borrow_mut() = Some(Rc::new(callback));
    }

    #[allow(unused)]
    pub fn get_text(&self) -> String {
        self.search_entry.text().to_string()
    }

    #[allow(unused)]
    pub fn clear(&self) {
        self.search_entry.set_text("");
    }
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}
