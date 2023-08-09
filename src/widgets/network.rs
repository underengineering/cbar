use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use std::{cell::RefCell, rc::Rc};
use sysinfo::{NetworkExt, System, SystemExt};
use tokio::time::{sleep, Duration};

pub fn build_network(sys: Rc<RefCell<System>>) -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["network"]);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        loop {
            dbg!(sys.borrow().networks());
            if let Some((iface, data)) = sys
                .borrow()
                .networks()
                .into_iter()
                .find(|(iface, data)| *iface == "wlp4s0")
            {
                label.set_text(&format!(
                    "UP {} : DOWN {}",
                    data.transmitted(),
                    data.received()
                ))
            }

            sleep(Duration::from_secs(2)).await;
        }
    });

    container
}
