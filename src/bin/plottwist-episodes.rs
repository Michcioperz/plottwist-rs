extern crate cursive;
extern crate plottwist;

use cursive::Cursive;
use cursive::views::SelectView;
use std::env;
use std::process::Command;

use plottwist::Series;

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let serie = Series { slug: env::args().next().expect("you must specify series slug as argument"), title: String::from(""), alt_title: String::from("") };
    let episodes = serie.episodes();
    let mut episodes_view = SelectView::new().h_align(cursive::align::HAlign::Left);
    for episode in episodes {
        episodes_view.add_item(format!("Episode {}", episode.number), episode);
    }
    episodes_view.set_on_submit(|_, episode| {
        Command::new("mpv").arg("--fs").arg(episode.url()).status().unwrap();
    });
    let mut siv = Cursive::new();
    siv.add_layer(episodes_view);
    siv.add_global_callback('q', Cursive::quit);
    siv.run();
}
