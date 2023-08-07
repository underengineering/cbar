use gtk::glib;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct KeyboardLayout(ObjectSubclass<imp::KeyboardLayout>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub mod imp {
    use gtk::glib;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct KeyboardLayout;

    #[glib::object_subclass]
    impl ObjectSubclass for KeyboardLayout {
        const NAME: &'static str = "CBarKeyboardLayout";
        type Type = super::KeyboardLayout;
        type ParentType = gtk::Button;
    }

    impl ObjectImpl for KeyboardLayout {}
    impl WidgetImpl for KeyboardLayout {}
    impl ButtonImpl for KeyboardLayout {}
}
