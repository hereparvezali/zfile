mod file_info;
mod file_view;
mod operations;
mod preferences;
mod search;
mod sidebar;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, HeaderBar, Orientation, Paned, ScrolledWindow};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

const APP_ID: &str = "com.example.zfile";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // Create header bar with navigation and view controls
    let header = HeaderBar::new();

    // Create main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("zfile")
        .default_width(1000)
        .default_height(600)
        .build();

    // Set the header bar as titlebar
    window.set_titlebar(Some(&header));

    // Navigation buttons
    let back_button = gtk::Button::from_icon_name("go-previous-symbolic");
    let forward_button = gtk::Button::from_icon_name("go-next-symbolic");
    let up_button = gtk::Button::from_icon_name("go-up-symbolic");

    header.pack_start(&back_button);
    header.pack_start(&forward_button);
    header.pack_start(&up_button);

    // Path bar (breadcrumb navigation)
    let path_bar = gtk::Box::new(Orientation::Horizontal, 4);
    path_bar.add_css_class("linked");
    header.set_title_widget(Some(&path_bar));

    // View toggle buttons
    let view_box = gtk::Box::new(Orientation::Horizontal, 0);
    view_box.add_css_class("linked");

    let grid_view_button = gtk::ToggleButton::builder()
        .icon_name("view-grid-symbolic")
        .active(true)
        .build();
    let list_view_button = gtk::ToggleButton::builder()
        .icon_name("view-list-symbolic")
        .build();

    view_box.append(&grid_view_button);
    view_box.append(&list_view_button);
    header.pack_end(&view_box);

    // Search button
    let search_button = gtk::ToggleButton::builder()
        .icon_name("system-search-symbolic")
        .build();
    header.pack_end(&search_button);

    // Menu button
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();

    let menu = gio::Menu::new();
    menu.append(Some("New Folder"), Some("app.new-folder"));
    menu.append(Some("New File"), Some("app.new-file"));
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("About"), Some("app.about"));
    menu_button.set_menu_model(Some(&menu));
    header.pack_end(&menu_button);

    // Main content area
    let main_paned = Paned::new(Orientation::Horizontal);

    // Sidebar
    let sidebar = sidebar::Sidebar::new();
    let sidebar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .width_request(100)
        .child(&sidebar.widget())
        .build();

    main_paned.set_start_child(Some(&sidebar_scroll));

    // File view area
    let file_view_container = Box::new(Orientation::Vertical, 0);

    // Search bar (hidden by default)
    let search_bar = search::SearchBar::new();
    file_view_container.append(&search_bar.widget());

    // File view (grid/list)
    let file_view = file_view::FileView::new();
    let file_view_scroll = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .child(&file_view.widget())
        .build();

    file_view_container.append(&file_view_scroll);

    // Status bar
    let status_bar = gtk::Box::new(Orientation::Horizontal, 12);
    status_bar.set_margin_start(12);
    status_bar.set_margin_end(12);
    status_bar.set_margin_top(6);
    status_bar.set_margin_bottom(6);

    let items_label = gtk::Label::new(Some("0 items"));
    let selected_label = gtk::Label::new(None);

    status_bar.append(&items_label);
    status_bar.append(&selected_label);

    file_view_container.append(&gtk::Separator::new(Orientation::Horizontal));
    file_view_container.append(&status_bar);

    main_paned.set_end_child(Some(&file_view_container));

    // Set main content
    window.set_child(Some(&main_paned));

    // Application state
    let app_state = Arc::new(Mutex::new(AppState {
        current_path: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        history: Vec::new(),
        history_index: 0,
        view_mode: ViewMode::Grid,
    }));

    // Connect signals
    setup_actions(&window, app, app_state.clone());
    setup_navigation(
        &back_button,
        &forward_button,
        &up_button,
        &path_bar,
        app_state.clone(),
        &file_view,
        &items_label,
    );
    setup_view_toggle(
        &grid_view_button,
        &list_view_button,
        &file_view,
        app_state.clone(),
    );
    setup_search(&search_button, &search_bar);
    setup_sidebar_navigation(
        &sidebar,
        app_state.clone(),
        &file_view,
        &items_label,
        &path_bar,
    );

    // Load initial directory
    let initial_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    file_view.load_directory(&initial_path);
    update_path_bar(&path_bar, &initial_path);
    update_status(&items_label, &file_view);

    // Handle file activation (double-click)
    let file_view_clone = file_view.clone();
    let app_state_clone = app_state.clone();
    let items_label_clone = items_label.clone();
    let path_bar_clone = path_bar.clone();

    file_view.connect_activated(move |path| {
        if path.is_dir() {
            file_view_clone.load_directory(&path);
            update_path_bar(&path_bar_clone, &path);
            update_status(&items_label_clone, &file_view_clone);

            glib::spawn_future_local({
                let app_state = app_state_clone.clone();
                let path = path.clone();
                async move {
                    let mut state = app_state.lock().await;
                    let history_index = state.history_index;
                    state.history.truncate(history_index + 1);
                    state.history.push(path.clone());
                    state.history_index = state.history.len() - 1;
                    state.current_path = path;
                }
            });
        } else {
            // Open file with default application
            if let Err(e) = open::that(&path) {
                eprintln!("Failed to open file: {}", e);
            }
        }
    });

    window.present();
}

#[derive(Clone)]
struct AppState {
    current_path: PathBuf,
    history: Vec<PathBuf>,
    history_index: usize,
    view_mode: ViewMode,
}

#[derive(Clone, PartialEq)]
enum ViewMode {
    Grid,
    List,
}

fn setup_actions(window: &ApplicationWindow, app: &Application, state: Arc<Mutex<AppState>>) {
    // New folder action
    let new_folder_action = gio::SimpleAction::new("new-folder", None);
    let window_weak = window.downgrade();
    let state_clone = state.clone();
    new_folder_action.connect_activate(move |_, _| {
        let window = window_weak.upgrade().unwrap();
        let state = state_clone.clone();

        glib::spawn_future_local(async move {
            let current_path = {
                let state = state.lock().await;
                state.current_path.clone()
            };

            operations::show_new_folder_dialog(&window, &current_path);
        });
    });
    app.add_action(&new_folder_action);

    // New file action
    let new_file_action = gio::SimpleAction::new("new-file", None);
    let window_weak = window.downgrade();
    let state_clone = state.clone();
    new_file_action.connect_activate(move |_, _| {
        let window = window_weak.upgrade().unwrap();
        let state = state_clone.clone();

        glib::spawn_future_local(async move {
            let current_path = {
                let state = state.lock().await;
                state.current_path.clone()
            };

            operations::show_new_file_dialog(&window, &current_path);
        });
    });
    app.add_action(&new_file_action);

    // Preferences action
    let preferences_action = gio::SimpleAction::new("preferences", None);
    let window_weak = window.downgrade();
    preferences_action.connect_activate(move |_, _| {
        let window = window_weak.upgrade().unwrap();
        preferences::show_preferences_dialog(&window);
    });
    app.add_action(&preferences_action);

    // About action
    let about_action = gio::SimpleAction::new("about", None);
    let window_weak = window.downgrade();
    about_action.connect_activate(move |_, _| {
        let window = window_weak.upgrade().unwrap();
        show_about_dialog(&window);
    });
    app.add_action(&about_action);
}

fn setup_navigation(
    back_button: &gtk::Button,
    forward_button: &gtk::Button,
    up_button: &gtk::Button,
    path_bar: &gtk::Box,
    state: Arc<Mutex<AppState>>,
    file_view: &file_view::FileView,
    items_label: &gtk::Label,
) {
    // Back button
    let state_clone = state.clone();
    let file_view_clone = file_view.clone();
    let items_label_clone = items_label.clone();
    let path_bar_clone = path_bar.clone();
    back_button.connect_clicked(move |_| {
        let state = state_clone.clone();
        let file_view = file_view_clone.clone();
        let items_label = items_label_clone.clone();
        let path_bar = path_bar_clone.clone();

        glib::spawn_future_local(async move {
            let mut state_guard = state.lock().await;
            if state_guard.history_index > 0 {
                state_guard.history_index -= 1;
                let path = state_guard.history[state_guard.history_index].clone();
                drop(state_guard);

                file_view.load_directory(&path);
                update_path_bar(&path_bar, &path);
                update_status(&items_label, &file_view);
            }
        });
    });

    // Forward button
    let state_clone = state.clone();
    let file_view_clone = file_view.clone();
    let items_label_clone = items_label.clone();
    let path_bar_clone = path_bar.clone();
    forward_button.connect_clicked(move |_| {
        let state = state_clone.clone();
        let file_view = file_view_clone.clone();
        let items_label = items_label_clone.clone();
        let path_bar = path_bar_clone.clone();

        glib::spawn_future_local(async move {
            let mut state_guard = state.lock().await;
            if state_guard.history_index < state_guard.history.len() - 1 {
                state_guard.history_index += 1;
                let path = state_guard.history[state_guard.history_index].clone();
                drop(state_guard);

                file_view.load_directory(&path);
                update_path_bar(&path_bar, &path);
                update_status(&items_label, &file_view);
            }
        });
    });

    // Up button
    let state_clone = state.clone();
    let file_view_clone = file_view.clone();
    let items_label_clone = items_label.clone();
    let path_bar_clone = path_bar.clone();
    up_button.connect_clicked(move |_| {
        let state = state_clone.clone();
        let file_view = file_view_clone.clone();
        let items_label = items_label_clone.clone();
        let path_bar = path_bar_clone.clone();

        glib::spawn_future_local(async move {
            let current_path = {
                let state_lock = state.lock().await;
                state_lock.current_path.clone()
            };

            if let Some(parent) = current_path.parent() {
                let parent_path = parent.to_path_buf();
                file_view.load_directory(&parent_path);
                update_path_bar(&path_bar, &parent_path);
                update_status(&items_label, &file_view);

                let mut state_lock = state.lock().await;
                let history_index = state_lock.history_index;
                state_lock.history.truncate(history_index + 1);
                state_lock.history.push(parent_path.clone());
                state_lock.history_index = state_lock.history.len() - 1;
                state_lock.current_path = parent_path;
            }
        });
    });
}

fn setup_view_toggle(
    grid_button: &gtk::ToggleButton,
    list_button: &gtk::ToggleButton,
    file_view: &file_view::FileView,
    state: Arc<Mutex<AppState>>,
) {
    let list_button_clone = list_button.clone();
    let file_view_clone = file_view.clone();
    let state_clone = state.clone();
    grid_button.connect_toggled(move |button| {
        if button.is_active() {
            list_button_clone.set_active(false);
            file_view_clone.set_view_mode(file_view::ViewMode::Grid);
            let state = state_clone.clone();
            glib::spawn_future_local(async move {
                let mut state = state.lock().await;
                state.view_mode = ViewMode::Grid;
            });
        }
    });

    let grid_button_clone = grid_button.clone();
    let file_view_clone = file_view.clone();
    let state_clone = state.clone();
    list_button.connect_toggled(move |button| {
        if button.is_active() {
            grid_button_clone.set_active(false);
            file_view_clone.set_view_mode(file_view::ViewMode::List);
            let state = state_clone.clone();
            glib::spawn_future_local(async move {
                let mut state = state.lock().await;
                state.view_mode = ViewMode::List;
            });
        }
    });
}

fn setup_search(search_button: &gtk::ToggleButton, search_bar: &search::SearchBar) {
    let search_bar_clone = search_bar.clone();
    search_button.connect_toggled(move |button| {
        search_bar_clone.set_visible(button.is_active());
        if button.is_active() {
            search_bar_clone.grab_focus();
        }
    });
}

fn setup_sidebar_navigation(
    sidebar: &sidebar::Sidebar,
    state: Arc<Mutex<AppState>>,
    file_view: &file_view::FileView,
    items_label: &gtk::Label,
    path_bar: &gtk::Box,
) {
    let state_clone = state.clone();
    let file_view_clone = file_view.clone();
    let items_label_clone = items_label.clone();
    let path_bar_clone = path_bar.clone();

    sidebar.connect_location_activated(move |path| {
        let state = state_clone.clone();
        let file_view = file_view_clone.clone();
        let items_label = items_label_clone.clone();
        let path_bar = path_bar_clone.clone();
        let path = path.clone();

        glib::spawn_future_local(async move {
            file_view.load_directory(&path);
            update_path_bar(&path_bar, &path);
            update_status(&items_label, &file_view);

            let mut state_lock = state.lock().await;
            let history_index = state_lock.history_index;
            state_lock.history.truncate(history_index + 1);
            state_lock.history.push(path.clone());
            state_lock.history_index = state_lock.history.len() - 1;
            state_lock.current_path = path;
        });
    });
}

fn update_path_bar(path_bar: &gtk::Box, path: &PathBuf) {
    // Clear existing buttons
    while let Some(child) = path_bar.first_child() {
        path_bar.remove(&child);
    }

    let components: Vec<_> = path.components().collect();

    for (i, component) in components.iter().enumerate() {
        let component_str = component.as_os_str().to_string_lossy().to_string();
        let display_name = if component_str.is_empty() || component_str == "/" {
            "/".to_string()
        } else {
            component_str
        };

        let button = gtk::Button::with_label(&display_name);
        button.add_css_class("flat");

        path_bar.append(&button);

        if i < components.len() - 1 {
            let separator = gtk::Label::new(Some("/"));
            separator.add_css_class("dim-label");
            path_bar.append(&separator);
        }
    }
}

fn update_status(items_label: &gtk::Label, file_view: &file_view::FileView) {
    let count = file_view.item_count();
    items_label.set_text(&format!("{} items", count));
}

fn show_about_dialog(parent: &ApplicationWindow) {
    let dialog = gtk::AboutDialog::builder()
        .program_name("File Manager")
        .logo_icon_name("system-file-manager")
        .version("0.1.0")
        .authors(vec!["Developer".to_string()])
        .license_type(gtk::License::MitX11)
        .comments("A modern file manager built with GTK4 and Rust")
        .transient_for(parent)
        .modal(true)
        .build();

    dialog.present();
}
