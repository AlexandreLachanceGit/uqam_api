use serde::Serialize;

pub struct Class {
    pub symbol: String,
    pub year: u32,
    pub semester: Semester,
    pub program_code: u32,
}

impl Class {
    pub fn get_url(&self) -> String {
        format!(
            "https://etudier.uqam.ca/wshoraire/cours/{}/{}{}/{}",
            self.symbol, self.year, self.semester as u8, self.program_code
        )
    }
}

#[derive(Serialize, Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Semester {
    Winter = 1,
    Summer = 2,
    Fall = 3,
}
