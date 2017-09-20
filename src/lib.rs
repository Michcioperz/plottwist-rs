#[macro_use] extern crate lazy_static;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate regex;
extern crate reqwest;

use std::fmt::{self,Display};
use std::io::Read;
use std::str::FromStr;
use regex::Regex;

#[derive(Deserialize,Debug)]
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


pub fn fetch_series_list() -> Vec<Series> {
    let mut resp = reqwest::get("https://twist.moe").unwrap();
    let mut content = String::new();
    resp.read_to_string(&mut content).unwrap();
    content.lines().map(Series::from_str).filter_map(Result::ok).collect()
}
