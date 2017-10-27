extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate reqwest;
extern crate htmlescape;

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs::File;
use std::env;
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

#[derive(Debug, Eq)]
pub struct Series {
    pub title: String,
    pub alt_title: String,
    pub slug: String,
    pub ongoing: bool,
    pub favourite: bool,
}

impl Display for Series {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{} {}{} [{}]",
            if self.ongoing { "O" } else { " " },
            if self.favourite { "F" } else { " " },
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

impl Ord for Series {
    fn cmp(&self, other: &Series) -> Ordering {
        match (self.favourite, other.favourite) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (_, _) => match (self.ongoing, other.ongoing) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                (_, _) => self.title.cmp(&other.title),
            },
        }
    }
}

impl PartialOrd for Series {
    fn partial_cmp(&self, other: &Series) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Series {
    fn eq(&self, other: &Series) -> bool {
        self.slug == other.slug
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

    fn from_match(s: &str, ongoings: &mut HashSet<String>, favourites: &mut HashSet<String>, series_regex: &Regex) -> Option<Series> {
        match series_regex.captures(s) {
            Some(c) => {
                let slug: String = htmlescape::decode_html(c.get(1).unwrap().as_str()).unwrap();
                let ongoing: bool = ongoings.remove(&slug);
                let favourite: bool = favourites.remove(&slug);
                let x = Series {
                    ongoing: ongoing,
                    favourite: favourite,
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
    let mut ongoings: HashSet<String> = ongoings
        .map(|capture: regex::Captures| {
            let m: regex::Match = capture.get(1).unwrap();
            let s: &str = m.as_str();
            String::from(s)
        })
        .collect();
    let mut favourites: HashSet<String> = match env::home_dir() {
        None => {
            println!("couldn't find the way home to .plottwistfavs");
            HashSet::new()
        },
        Some(home_path) => {
            match File::open(home_path.join(".plottwistfavs")) {
                Err(why) => {
                    println!("couldn't open ~/.plottwistfavs: {}", why);
                    HashSet::new()
                },
                Ok(mut file) => {
                    let mut s = String::new();
                    match file.read_to_string(&mut s) {
                        Err(why) => {
                            println!("couldn't read ~/.plottwistfavs: {}", why);
                            HashSet::new()
                        },
                        Ok(_) => {
                            s.lines().map(|x| String::from(x)).collect()
                        },
                    }
                },
            }
        },
    };
    let series_regex: Regex = Regex::new(r#"<a href="/a/(?P<slug>[a-zA-Z0-9-]+?)" class="series-title" data-title="(?P<title>[^"]*?)"(?: data-alt="(?P<alt>[^"]*?)")?>[^<]+"#).unwrap();
    let mut sercolle: Vec<Series> = content
        .lines()
        .map(|line| Series::from_match(line, &mut ongoings, &mut favourites, &series_regex))
        .filter_map(|x| x)
        .collect();
    sercolle.sort();
    sercolle
}
