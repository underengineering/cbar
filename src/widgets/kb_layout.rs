use crate::hyprland::{
    events::Event,
    ipc::{self, commands::Devices},
};
use gtk::{
    glib::{self, clone, MainContext},
    traits::{BoxExt, WidgetExt},
};
use tokio::sync::broadcast;

pub fn build_keyboard_label(mut receiver: broadcast::Receiver<Event>) -> gtk::Box {
    let label = gtk::Label::new(None);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    container.set_css_classes(&["widget", "layout"]);
    container.append(&label);

    let main_context = MainContext::default();
    main_context.spawn_local(async move {
        let mut buffer = Vec::new();
        if let Some(keyboard) = ipc::request::<Devices>(&mut buffer)
            .await
            .unwrap()
            .keyboards
            .iter()
            .find(|kb| kb.main)
        {
            label.set_text(&keyboard.active_keymap);
        }

        while let Ok(event) = receiver.recv().await {
            if let Event::ActiveLayout {
                keyboard_name: _,
                layout_name,
            } = event
            {
                label.set_text(&layout_name);
            }
        }
    });

    container
}
