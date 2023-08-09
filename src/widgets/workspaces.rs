use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use std::collections::HashMap;
use tokio::{
    sync::broadcast,
    time::{sleep, Duration},
};

use crate::hyprland::{
    events::Event,
    ipc::{
        self,
        commands::{Command, Workspaces},
    },
};

pub fn build_workspaces(mut receiver: broadcast::Receiver<Event>) -> gtk::Box {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "workspaces"]);

    let main_context = MainContext::default();
    let container_ref = container.clone(); // glib::clone breaks formatting
    main_context.spawn_local(async move {
        let mut labels = HashMap::new();

        let mut buffer = Vec::new();
        for workspace in ipc::request::<Workspaces>(&mut buffer).await.unwrap() {
            let label = gtk::Button::with_label(&workspace.name);
            label.set_css_classes(&["workspace"]);
            container_ref.append(&label);
            labels.insert(workspace.name, label);
        }

        loop {
            let event = receiver.recv().await.unwrap();
            println!("ev {:?}", event);
            match event {
                Event::CreateWorkspace { name } => {
                    let label = gtk::Button::with_label(&name);
                    container_ref.append(&label);
                    labels.insert(name, label);
                }
                Event::DestroyWorkspace { name } => {
                    if let Some(label) = labels.remove(&name) {
                        container_ref.remove(&label);
                    }
                }
                _ => (),
            }
        }
    });

    container
}
