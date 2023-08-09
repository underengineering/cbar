use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use std::{cell::RefCell, rc::Rc};
use sysinfo::{System, SystemExt};
use tokio::time::{sleep, Duration};

pub fn build_memory_usage(sys: Rc<RefCell<System>>) -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "memory"]);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let memory_usage = {
                let sys = sys.borrow();
                let used = sys.used_memory();
                let total = sys.total_memory();
                (used as f64) / (total as f64) * 100f64
            };

            label.set_text(&format!("{}%", memory_usage.round()));

            sleep(Duration::from_secs(2)).await;
        }
    });

    container
}
