extern crate cursive;
extern crate plottwist;

use cursive::Cursive;
use cursive::views::{Dialog, SelectView};
use std::process::Command;

use plottwist::fetch_series_list;

fn main() {
    let series_list = fetch_series_list();
    let mut series_view = SelectView::new().h_align(cursive::align::HAlign::Left);
    for series in series_list {
        series_view.add_item(format!("{}", series), series);
    }
    series_view.set_on_submit(|_, serie| {
        Command::new("urxvt")
            .arg("-e")
            .arg("plottwist-episodes")
            .arg(&serie.slug)
            .spawn()
            .unwrap();
    });
    let mut siv = Cursive::new();
    siv.add_layer(Dialog::around(series_view).title("Series list"));
    siv.add_global_callback(cursive::event::Key::Esc, Cursive::quit);
    siv.run();
}
