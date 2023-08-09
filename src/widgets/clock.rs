use chrono::{DateTime, Local};
use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, ButtonExt, WidgetExt},
};
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

pub fn build_clock() -> gtk::Box {
    let button = gtk::Button::new();

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "clock"]);
    container.append(&button);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let date: DateTime<Local> = Local::now();
            button.set_label(&date.format("%H:%M").to_string());
            sleep(Duration::from_secs(15)).await;
        }
    });

    container
}
