#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate cursive;
extern crate regex;
extern crate reqwest;

use cursive::Cursive;
use cursive::event::EventResult;
use cursive::views::{SelectView,OnEventView};
use std::fmt::{self,Display,Formatter};
use std::io::Read;
use std::process::Command;
use std::str::FromStr;
use regex::Regex;

#[derive(Deserialize)]
pub struct Episode {
    pub source: String,
    pub number: usize,
}

#[derive(Deserialize)]
struct SeriesObject {
    episodes: Vec<Episode>,
}

pub struct Series {
    pub title: String,
    pub alt_title: String,
    pub slug: String,
}

impl Series {
    fn episodes(&self) -> Vec<Episode> {
        let mut resp = reqwest::get(reqwest::Url::parse(&self.url()).unwrap()).unwrap();
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        let found = false;
        for line in content.lines() {
            if found {
                let s: SeriesObject = serde_json::from_str(line).unwrap();
                return s.episodes;
            } else if line.trim() == r#"<script id="series-object" type="application/json">"# {
                found = true;
            }
        }
        return Vec::new();
    }
    fn url(&self) -> String {
        format!("https://twist.moe/a/{}", self.slug)
    }
}


impl Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.alt_title.is_empty() {
            write!(f, "{} [{}]", self.title, self.slug)
        } else {
            write!(f, "{} ({}) [{}]", self.title, self.alt_title, self.slug)
        }
    }
}

impl FromStr for Series {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"<a href="/a/(?P<slug>[a-zA-Z0-9-]+?)" class="series-title" data-title="(?P<title>[^"]*?)"(?: data-alt="(?P<alt>[^"]*?)")?>[^<]+"#).unwrap();
        }
        match RE.captures(s) {
            Some(c) => {
                let x = Self { slug: c.get(1).unwrap().as_str().to_string(), title: c.get(2).unwrap().as_str().to_string(), alt_title: match c.get(3) {
                    Some(m) => m.as_str().to_string(),
                    None => String::from(""),
                }};
                Ok(x)
            },
            None => Err(String::from("oh no")),
        }
    }
}


fn fetch_series_list() -> Vec<Series> {
    let mut resp = reqwest::get("https://twist.moe").unwrap();
    let mut content = String::new();
    resp.read_to_string(&mut content).unwrap();
    content.lines().map(Series::from_str).filter_map(Result::ok).collect()
}

fn main() {
    let series_list = fetch_series_list();
    let mut series_view = SelectView::new().h_align(cursive::align::HAlign::Left);
    for series in series_list {
        series_view.add_item(format!("{}", series), series);
    }
    series_view.set_on_submit(|s, serie| {
        let episodes = serie.episodes();
        let mut episodes_view = SelectView::new().h_align(cursive::align::HAlign::Left);
        for episode in episodes {
            episode.source = "https://twist.moe".to_owned() + episode.source.trim();
            episodes_view.add_item(format!("Episode {} – {}", episode.number, episode.source), &episode);
        }
        episodes_view.add_item(format!("Play all – {}", serie.url()), &Episode { number: 0, source: serie.url()});
        episodes_view.set_on_submit(|s, episode: &Episode| {
            Command::new("mpv").arg("--fs").arg(episode.source).status().unwrap();
        });
        let episodes_view = OnEventView::new(episodes_view)
            .on_event('q', |s| {
                s.pop_layer();
            });
        s.add_layer(episodes_view);
    });
    let mut siv = Cursive::new();
    siv.add_layer(series_view);
    siv.add_global_callback('q', Cursive::quit);
    siv.run();
}
