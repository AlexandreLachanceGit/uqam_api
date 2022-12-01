use phf::{phf_map, Map};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use std::fmt::Debug;

mod course;
use crate::course::{Class, Semester};

#[derive(Serialize, Debug)]
#[allow(dead_code)]
struct Group {
    id: u32,
    available_places: u32,
    teachers: Vec<String>,
    periods: Vec<Period>,
    exams: Option<Vec<Exam>>,
}

#[derive(Serialize, Debug)]
#[allow(dead_code)]
struct Period {
    day: String,
    start_date: String,
    end_date: String,
    start_time: String,
    end_time: String,
    location: Option<Location>,
    #[serde(rename(serialize = "type"))]
    type_: String,
}

#[derive(Serialize, Debug)]
struct Exam {}

#[derive(Serialize, Debug)]
#[allow(dead_code)]
struct Location {
    classroom: Option<String>,
    campus: String,
}

static MONTHS_MAP: Map<&'static str, u8> = phf_map! {
    "janvier" => 1,
    "février" => 2,
    "mars" => 3,
    "avril" => 4,
    "mai" => 5,
    "juin" => 6,
    "juillet" => 7,
    "août" => 8,
    "septembre" => 9,
    "octobre" => 10,
    "novembre" => 11,
    "décembre" => 12,
};

fn main() {
    let class = Class {
        symbol: String::from("inf1070"),
        year: 2022,
        semester: Semester::Fall,
        program_code: 7316,
    };
    let url = class.get_url();

    let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
    let groups = parse_groups(response);

    println!("{}", serde_json::to_string(&groups).unwrap());
}

fn parse_groups(html: String) -> Vec<Group> {
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("div.groupe").unwrap();
    let mut groups: Vec<Group> = vec![];

    for group_element in doc.select(&selector) {
        let group_html = group_element.inner_html();
        let group = parse_group(group_html);

        groups.push(group);
    }

    groups
}

fn parse_group(html: String) -> Group {
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("div.ligne").unwrap();
    let lines = doc.select(&selector).collect::<Vec<ElementRef>>();

    Group {
        id: parse_group_id(&lines[1]),
        available_places: parse_group_places(&lines[2]),
        teachers: parse_group_teachers(&lines[3]),
        periods: parse_group_periods(&lines[4]),
        exams: None,
    }
}

fn parse_group_id(doc: &ElementRef) -> u32 {
    let selector = Selector::parse("h3").unwrap();

    doc.select(&selector)
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .split(' ')
        .collect::<Vec<&str>>()[1]
        .parse::<u32>()
        .unwrap()
}

fn parse_group_teachers(doc: &ElementRef) -> Vec<String> {
    let selector = Selector::parse("td>ul>li").unwrap();

    doc.select(&selector)
        .map(|x| x.inner_html())
        .collect::<Vec<String>>()
}

fn parse_group_places(doc: &ElementRef) -> u32 {
    let selector = Selector::parse("span").unwrap();

    doc.select(&selector)
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .split(' ')
        .collect::<Vec<&str>>()[0]
        .parse::<u32>()
        .unwrap()
}

fn parse_group_periods(doc: &ElementRef) -> Vec<Period> {
    let selector = Selector::parse("tr").unwrap();
    let mut period_elements = doc.select(&selector).collect::<Vec<ElementRef>>();
    period_elements.remove(0);

    let mut periods: Vec<Period> = vec![];

    for period_el in period_elements {
        periods.push(parse_period(&period_el));
    }

    periods
}

fn parse_period(doc: &ElementRef) -> Period {
    let selector = Selector::parse("td").unwrap();
    let parts = doc.select(&selector).collect::<Vec<ElementRef>>();
    let mut text_parts: Vec<String> = vec![];

    for part in parts {
        let a_selector = Selector::parse("a").unwrap();

        let a_parts = part
            .select(&a_selector)
            .map(|x| x.inner_html())
            .collect::<Vec<String>>();

        let text_part = match a_parts.len() {
            0 => part.inner_html(),
            _ => a_parts[0].clone(),
        };

        text_parts.push(text_part);
    }

    let (start_date, end_date) = parse_dates(&text_parts[1]);
    let times = text_parts[2]
        .split("&nbsp;")
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>();

    let trimmed_day = text_parts[0].trim().to_string();

    Period {
        day: trimmed_day,
        start_date,
        end_date,
        start_time: times[1].to_string(),
        end_time: times[3].to_string(),
        location: parse_location(&text_parts[3]),
        type_: text_parts[4].to_string(),
    }
}

fn parse_dates(text: &str) -> (String, String) {
    let parts = text
        .split("<br>")
        .map(|x| x.trim().replace("Du ", "").replace("au ", ""))
        .map(|x| format_date(&x))
        .collect::<Vec<String>>();

    (parts[0].to_string(), parts[1].to_string())
}

fn format_date(text: &str) -> String {
    let parts: Vec<&str> = text.split(' ').collect();

    format!(
        "{}-{:0>2}-{:0>2}",
        parts[2],
        MONTHS_MAP.get(parts[1]).unwrap(),
        parts[0]
    )
}

fn parse_location(text: &str) -> Option<Location> {
    let parts = text
        .split('|')
        .map(|x| x.trim().to_string())
        .collect::<Vec<String>>();

    match text.is_empty() {
        false => Some(Location {
            classroom: if !parts[0].is_empty() {
                Some(parts[0].to_string())
            } else {
                None
            },
            campus: parts[1].to_string(),
        }),
        true => None,
    }
}
