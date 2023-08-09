use gtk::{
    glib::{self, clone, MainContext},
    prelude::*,
    Application, ApplicationWindow, CssProvider,
};
use hyprland::event_loop::EventLoop;
use std::{cell::RefCell, rc::Rc};
use sysinfo::{CpuRefreshKind, RefreshKind, System, SystemExt};
use tokio::time::{sleep, Duration};
use widgets::{
    build_active_window, build_battery, build_clock, build_cpu_load, build_keyboard_label,
    build_memory_usage, build_network, build_workspaces,
};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

mod hyprland;
mod system_info;
mod widgets;

#[tokio::main]
async fn main() -> glib::ExitCode {
    let event_loop = Rc::new(RefCell::new(EventLoop::connect().await.unwrap()));

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(
        clone!(@weak event_loop => move |app: &Application| build_ui(app, event_loop.clone())),
    );
    app.connect_startup(|_| load_css());

    // Run hyprland event loop
    let main_context = MainContext::default();
    main_context.spawn_local(async move { event_loop.borrow_mut().run().await.unwrap() });

    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(&dbg!(grass::from_path(
        "/tmp/test.scss",
        &grass::Options::default()
    )
    .unwrap()));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application, mut event_loop: Rc<RefCell<EventLoop>>) {
    let specifics = RefreshKind::new()
        .with_cpu(CpuRefreshKind::new().with_cpu_usage())
        .with_memory()
        .with_networks()
        .with_networks_list();
    let sys = Rc::new(RefCell::new(System::new_with_specifics(specifics)));

    let main_context = MainContext::default();
    main_context.spawn_local(clone!(@weak sys => async move {
        loop {
            {
                let mut sys = sys.borrow_mut();
                sys.refresh_specifics(specifics);
            }
            sleep(Duration::from_secs(1)).await;
        }
    }));

    let event_loop = event_loop.borrow();
    let workspaces = build_workspaces(event_loop.receiver());
    // let active_window = build_active_window(event_loop.receiver());
    let kb_layout = build_keyboard_label(event_loop.receiver());
    let network = build_network(sys.clone());
    let battery = build_battery();
    let cpu_load = build_cpu_load(sys.clone());
    let memory_usage = build_memory_usage(sys.clone());
    let clock = build_clock();

    let widgets = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    widgets.append(&workspaces);
    // widgets.append(&active_window);
    widgets.append(&kb_layout);
    widgets.append(&network);
    widgets.append(&battery);
    widgets.append(&cpu_load);
    widgets.append(&memory_usage);
    widgets.append(&clock);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("cbar")
        .child(&widgets)
        .build();

    // gtk4_layer_shell::init_for_window(&window);
    // gtk4_layer_shell::set_layer(&window, gtk4_layer_shell::Layer::Top);
    // gtk4_layer_shell::auto_exclusive_zone_enable(&window);
    // gtk4_layer_shell::set_anchor(&window, gtk4_layer_shell::Edge::Top, true);

    window.present()
}
