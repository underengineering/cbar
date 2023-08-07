use gtk::{glib, Application, Label};
use gtk::{prelude::*, ApplicationWindow};

const APP_ID: &str = "org.gtk_rs.HelloWorld1";

mod widgets;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let label = Label::builder().label("bro").build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("cbar")
        .child(&label)
        .build();

    window.present()
}
