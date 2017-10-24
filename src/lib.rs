#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate reqwest;
extern crate htmlescape;

use std::fmt::{self, Display};
use std::io::Read;
use regex::Regex;

#[derive(Deserialize, Debug)]
pub struct Episode {
    pub source: String,
    pub number: usize,
}

impl Episode {
    pub fn url(&self) -> String {
        format!("https://twist.moe{}", self.source.trim())
    }
}

#[derive(Deserialize)]
struct SeriesObject {
    episodes: Vec<Episode>,
}

#[derive(Debug)]
pub struct Series {
    pub title: String,
    pub alt_title: String,
    pub slug: String,
    pub ongoing: bool,
}

impl Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{} [{}]",
            if self.ongoing { "{O} " } else { "    " },
            self.title,
            if !self.alt_title.is_empty() {
                format!(" ({})", self.alt_title)
            } else {
                format!("")
            },
            self.slug
        )
    }
}

impl Series {
    pub fn episodes(&self) -> Vec<Episode> {
        let mut resp = reqwest::get(reqwest::Url::parse(&self.url()).unwrap()).unwrap();
        let mut content = String::new();
        resp.read_to_string(&mut content).unwrap();
        let mut found = false;
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
    pub fn url(&self) -> String {
        format!("https://twist.moe/a/{}", self.slug)
    }

    fn from_match(s: &str, ongoings: &mut Vec<String>) -> Option<Series> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r#"<a href="/a/(?P<slug>[a-zA-Z0-9-]+?)" class="series-title" data-title="(?P<title>[^"]*?)"(?: data-alt="(?P<alt>[^"]*?)")?>[^<]+"#).unwrap();
        }
        match RE.captures(s) {
            Some(c) => {
                let slug: String = htmlescape::decode_html(c.get(1).unwrap().as_str()).unwrap();
                let ongoing: bool = if let Some(last_ongoing) = ongoings.pop() {
                    if last_ongoing.as_str() == slug.as_str() {
                        true
                    } else {
                        ongoings.push(last_ongoing);
                        false
                    }
                } else {
                    false
                };
                let x = Series {
                    ongoing: ongoing,
                    slug: slug,
                    title: htmlescape::decode_html(c.get(2).unwrap().as_str()).unwrap(),
                    alt_title: match c.get(3) {
                        Some(m) => htmlescape::decode_html(m.as_str()).unwrap(),
                        None => String::from(""),
                    },
                };
                Some(x)
            }
            None => None,
        }
    }
}


pub fn fetch_series_list() -> Vec<Series> {
    let mut resp = reqwest::get("https://twist.moe").unwrap();
    let mut content = String::new();
    resp.read_to_string(&mut content).unwrap();
    let ongoings_regex = Regex::new(r#"<a href="/a/(?P<slug>[a-zA-Z0-9-]+?)/last" tabindex="-1" class="fixed ongoing">ONGOING</a>"#).unwrap();
    let ongoings: regex::CaptureMatches = ongoings_regex.captures_iter(&content);
    let mut ongoings: Vec<String> = ongoings
        .map(|capture: regex::Captures| {
            let m: regex::Match = capture.get(1).unwrap();
            let s: &str = m.as_str();
            String::from(s)
        })
        .collect();
    ongoings.reverse();
    content
        .lines()
        .map(|line| Series::from_match(line, &mut ongoings))
        .filter_map(|x| x)
        .collect()
}
