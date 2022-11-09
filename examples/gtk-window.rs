use std::{error::Error, time::Duration};

use glib::{clone, Continue};
use gtk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    traits::{ButtonExt, ContainerExt, GtkWindowExt, WidgetExt},
};

fn on_active(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .default_height(500)
        .default_width(500)
        .window_position(gtk::WindowPosition::None)
        .build();
    let row = gtk::ListBoxRow::new();
    let button = gtk::Button::with_label("Hide and show");
    button.connect_clicked(clone!(@weak window => move |_| {
        let (dx,dy ) = window.position();
        let (width,height ) = window.size();
        window.hide();
        glib::timeout_add_local(Duration::from_secs(1), clone!(@strong window => move || {
            window.show_all();

            window.move_(dx, dy);
            window.resize(width, height);
            Continue(false)
        }));
    }));

    row.add(&button);

    window.set_child(Some(&row));
    window.show_all();
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = gtk::Application::builder()
        .application_id("test-window")
        .build();
    app.connect_activate(on_active);
    app.run();
    Ok(())
}
