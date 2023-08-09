use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::time::{sleep, Duration};

pub fn build_cpu_load(sys: Rc<RefCell<System>>) -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "cpu"]);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        loop {
            let cpu_usage = {
                let sys = sys.borrow();
                sys.global_cpu_info().cpu_usage()
            };

            label.set_text(&format!("{}%", cpu_usage.round()));

            sleep(Duration::from_secs(2)).await;
        }
    });

    container
}
