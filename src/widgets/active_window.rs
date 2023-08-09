use crate::hyprland::{
    events::Event,
    ipc::{self, commands::ActiveWindow},
};
use gtk::{
    glib::{self, clone, MainContext},
    traits::BoxExt,
};
use tokio::sync::broadcast;

pub fn build_active_window(mut receiver: broadcast::Receiver<Event>) -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        let mut buffer = Vec::new();
        let active_window = ipc::request::<ActiveWindow>(&mut buffer).await.unwrap();
        if let Some(title) = active_window.title {
            label.set_text(&title);
        }

        loop {
            let event = receiver.recv().await.unwrap();
            println!("ev {:?}", event);
            match event {
                Event::ActiveWindow { class, title } => {
                    label.set_text(&title);
                }
                _ => {}
            }
        }
    });

    container
}
