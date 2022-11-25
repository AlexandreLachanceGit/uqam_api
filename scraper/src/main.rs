use scraper::{Html, Selector};
use std::fmt::Debug;

struct Class {
    symbol: String,
    year: usize,
    semester: Semester,
    program_code: usize,
}

impl Class {
    pub fn get_url(&self) -> String {
        format!(
            "https://etudier.uqam.ca/wshoraire/cours/{}/{}{}/{}",
            self.symbol, self.year, self.semester as u8, self.program_code
        )
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum Semester {
    Winter = 1,
    Summer = 2,
    Fall = 3,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Group {
    id: usize,
    teachers: Vec<String>,
}

fn main() {
    let class = Class {
        symbol: String::from("mkg3300"),
        year: 2022,
        semester: Semester::Winter,
        program_code: 7316,
    };
    let url = class.get_url();

    let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
    let _groups = parse_groups(response);
}

fn parse_groups(html: String) -> Vec<Group> {
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("div.groupe").unwrap();
    let mut groups: Vec<Group> = vec![];

    for group_element in doc.select(&selector) {
        let group_html = group_element.inner_html();
        let group = parse_group(group_html);
        println!("{:?}", group);

        groups.push(group);
    }

    groups
}

fn parse_group(html: String) -> Group {
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("div.ligne").unwrap();
    let lines = doc
        .select(&selector)
        .map(|x| x.inner_html())
        .collect::<Vec<String>>();

    Group {
        id: parse_group_id(&lines[1]),
        teachers: parse_group_teachers(&lines[3]),
    }
}

fn parse_group_id(html: &str) -> usize {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("h3").unwrap();

    doc.select(&selector)
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .split(' ')
        .collect::<Vec<&str>>()[1]
        .parse::<usize>()
        .unwrap()
}

fn parse_group_teachers(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let selector = Selector::parse("td>ul>li").unwrap();

    doc.select(&selector)
        .map(|x| x.inner_html())
        .collect::<Vec<String>>()
}
