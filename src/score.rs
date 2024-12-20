use crate::university_weight::*;
use paste::paste;
use peroxide::fuga::*;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Copy, Clone)]
pub struct Score {
    standard_score: f64,
    percentile: f64,
    rank: usize,
}

impl Score {
    pub fn standard_score(&self) -> f64 {
        self.standard_score
    }

    pub fn percentile(&self) -> f64 {
        self.percentile
    }

    pub fn rank(&self) -> usize {
        self.rank
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Subject {
    Korean,
    Math,
    English,
    Chemistry,
    EarthScience,
}

impl Subject {
    pub fn name(&self) -> &'static str {
        match self {
            Subject::Korean => "Korean",
            Subject::Math => "Math",
            Subject::English => "English",
            Subject::Chemistry => "Chemistry",
            Subject::EarthScience => "EarthScience",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    name: String,
    scores: HashMap<Subject, Score>,
}

impl Record {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            scores: HashMap::new(),
        }
    }

    pub fn record(&mut self, subject: Subject, standard_score: f64, percentile: f64, rank: usize) {
        self.scores.insert(
            subject,
            Score {
                standard_score,
                percentile,
                rank,
            },
        );
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn korean(&self) -> Score {
        *self.scores.get(&Subject::Korean).unwrap()
    }

    pub fn math(&self) -> Score {
        *self.scores.get(&Subject::Math).unwrap()
    }

    pub fn english(&self) -> Score {
        *self.scores.get(&Subject::English).unwrap()
    }

    pub fn chemistry(&self) -> Score {
        *self.scores.get(&Subject::Chemistry).unwrap()
    }

    pub fn earth_science(&self) -> Score {
        *self.scores.get(&Subject::EarthScience).unwrap()
    }

    pub fn standard_score(&self, subject: Subject) -> f64 {
        self.scores.get(&subject).unwrap().standard_score
    }

    pub fn percentile(&self, subject: Subject) -> f64 {
        self.scores.get(&subject).unwrap().percentile
    }

    pub fn rank(&self, subject: Subject) -> usize {
        self.scores.get(&subject).unwrap().rank
    }

    pub fn to_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::new(vec![]);
        df.push(
            "Korean",
            Series::new(vec![
                self.korean().standard_score(),
                self.korean().percentile(),
                self.korean().rank() as f64,
            ]),
        );
        df.push(
            "Math",
            Series::new(vec![
                self.math().standard_score(),
                self.math().percentile(),
                self.math().rank() as f64,
            ]),
        );
        df.push(
            "English",
            Series::new(vec![0f64, 0f64, self.english().rank() as f64]),
        );
        df.push(
            "Chemistry",
            Series::new(vec![
                self.chemistry().standard_score(),
                self.chemistry().percentile(),
                self.chemistry().rank() as f64,
            ]),
        );
        df.push(
            "EarthScience",
            Series::new(vec![
                self.earth_science().standard_score(),
                self.earth_science().percentile(),
                self.earth_science().rank() as f64,
            ]),
        );

        df
    }

    pub fn write_parquet(&self) -> Result<(), Box<dyn std::error::Error>> {
        let df = self.to_dataframe();
        let path = format!("data/{}", self.name());
        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir(&path)?;
        }
        df.write_parquet(
            &format!("data/{}/record.parquet", self.name()),
            CompressionOptions::Uncompressed,
        )?;
        Ok(())
    }

    pub fn read_parquet(name: &str) -> Self {
        let df = DataFrame::read_parquet(&format!("data/{}/record.parquet", name)).unwrap();
        let korean: Vec<f64> = df["Korean"].to_vec();
        let math: Vec<f64> = df["Math"].to_vec();
        let english: Vec<f64> = df["English"].to_vec();
        let chemistry: Vec<f64> = df["Chemistry"].to_vec();
        let earth_science: Vec<f64> = df["EarthScience"].to_vec();

        let mut record = Record::new(name);

        record.record(Subject::Korean, korean[0], korean[1], korean[2] as usize);
        record.record(Subject::Math, math[0], math[1], math[2] as usize);
        record.record(Subject::English, 0f64, 0f64, english[2] as usize);
        record.record(
            Subject::Chemistry,
            chemistry[0],
            chemistry[1],
            chemistry[2] as usize,
        );
        record.record(
            Subject::EarthScience,
            earth_science[0],
            earth_science[1],
            earth_science[2] as usize,
        );

        record
    }

    pub fn calc_with_university(&self, university: University, year: usize) -> f64 {
        let weight = UniversityWeight::load(university, year);
        let weight_sum_except_eng = weight.korean + weight.math + weight.science;
        let weight_eng = weight.english;
        let weight_sum = weight_sum_except_eng + weight_eng;

        let korean = self.korean().standard_score() * weight.korean / weight_sum_except_eng;
        let math = self.math().standard_score() * weight.math / weight_sum_except_eng;
        let science_required = weight.science_required();
        let science_cand = match science_required {
            1 => {
                self.chemistry()
                    .standard_score()
                    .max(self.earth_science().standard_score())
                    * 2f64
            }
            2 => self.chemistry().standard_score() + self.earth_science().standard_score(),
            _ => unreachable!(),
        };
        let science = science_cand * weight.science / weight_sum_except_eng;

        let total = (korean + math + science) * 3f64;

        let eng_rank = self.english().rank();
        let eng_required_rank = weight.english_required();
        let eng_table = weight.english_table();

        let eng_default_score = eng_table[eng_required_rank];
        let eng_score = eng_table[eng_rank];

        if weight_eng > 0f64 {
            total + (eng_score - eng_default_score) * weight_eng / weight_sum
        } else {
            total + (eng_score - eng_default_score) / 4f64
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum University {
    KYUNGHEE,
    DONGGUK,
    SEOULSCITECH,
    KWANGWOON,
    INHA,
    ERICA,
    SEJONG,
    KOOKMIN,
    AJU,
    SOONGSIL,
    KONKUK,
    CATHOLIC,
    CHUNGANG,
    SEOUL,
    SOGANG,
}

impl University {
    pub fn name(&self) -> &'static str {
        match self {
            University::KYUNGHEE => "경희대(서울)",
            University::DONGGUK => "동국대",
            University::SEOULSCITECH => "서울과기대",
            University::KWANGWOON => "광운대",
            University::INHA => "인하대",
            University::ERICA => "한양대(ERICA)",
            University::SEJONG => "세종대",
            University::KOOKMIN => "국민대",
            University::AJU => "아주대",
            University::SOONGSIL => "숭실대",
            University::KONKUK => "건국대",
            University::CATHOLIC => "가톨릭대",
            University::CHUNGANG => "중앙대",
            University::SEOUL => "서울시립대",
            University::SOGANG => "서강대",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniversityWeight {
    korean: f64,
    math: f64,
    english: f64,
    science: f64,
    science_required: usize, // Number of required subjects
    english_required: usize, // Default rank
    english_table: Vec<f64>,
}

macro_rules! make_university_weight {
    ($univ:ident, $year:expr) => {
        {
            paste! {
                let weight = [<$univ _ $year _WEIGHT>].to_vec();
                let korean = weight[0];
                let math = weight[1];
                let english = weight[2];
                let science = weight[3];
                let science_required = [<$univ _ $year _SCI_REQ>];
                let english_required = [<$univ _ $year _ENG_REQ>];
                let english_table = [<$univ _$year _ENG>].to_vec().iter().map(|x| *x as f64).collect::<Vec<f64>>();

                UniversityWeight {
                    korean: korean as f64,
                    math: math as f64,
                    english: english as f64,
                    science: science as f64,
                    science_required,
                    english_required,
                    english_table,
                }
            }
        }
    }
}

impl UniversityWeight {
    pub fn load(univ: University, year: usize) -> Self {
        match (univ, year) {
            // 2022
            (University::KYUNGHEE, 2022) => make_university_weight!(KYUNGHEE, 2022),
            (University::DONGGUK, 2022) => make_university_weight!(DONGGUK, 2022),
            (University::SEOULSCITECH, 2022) => make_university_weight!(SEOULSCITECH, 2022),
            (University::KWANGWOON, 2022) => make_university_weight!(KWANGWOON, 2022),
            (University::INHA, 2022) => make_university_weight!(INHA, 2022),
            (University::ERICA, 2022) => make_university_weight!(ERICA, 2022),
            (University::SEJONG, 2022) => make_university_weight!(SEJONG, 2022),
            (University::KOOKMIN, 2022) => make_university_weight!(KOOKMIN, 2022),
            (University::AJU, 2022) => make_university_weight!(AJU, 2022),
            (University::SOONGSIL, 2022) => make_university_weight!(SOONGSIL, 2022),
            (University::CATHOLIC, 2022) => make_university_weight!(CATHOLIC, 2022),
            // 2023
            (University::KYUNGHEE, 2023) => make_university_weight!(KYUNGHEE, 2023),
            (University::DONGGUK, 2023) => make_university_weight!(DONGGUK, 2023),
            (University::SEOULSCITECH, 2023) => make_university_weight!(SEOULSCITECH, 2023),
            (University::KWANGWOON, 2023) => make_university_weight!(KWANGWOON, 2023),
            (University::INHA, 2023) => make_university_weight!(INHA, 2023),
            (University::ERICA, 2023) => make_university_weight!(ERICA, 2023),
            (University::SEJONG, 2023) => make_university_weight!(SEJONG, 2023),
            (University::KOOKMIN, 2023) => make_university_weight!(KOOKMIN, 2023),
            (University::AJU, 2023) => make_university_weight!(AJU, 2023),
            (University::SOONGSIL, 2023) => make_university_weight!(SOONGSIL, 2023),
            (University::CATHOLIC, 2023) => make_university_weight!(CATHOLIC, 2023),
            // 2024
            (University::SOGANG, 2024) => make_university_weight!(SOGANG, 2024),
            (University::CHUNGANG, 2024) => make_university_weight!(CHUNGANG, 2024),
            (University::KYUNGHEE, 2024) => make_university_weight!(KYUNGHEE, 2024),
            (University::SEOUL, 2024) => make_university_weight!(SEOUL, 2024),
            (University::DONGGUK, 2024) => make_university_weight!(DONGGUK, 2024),
            (University::SEOULSCITECH, 2024) => make_university_weight!(SEOULSCITECH, 2024),
            (University::KWANGWOON, 2024) => make_university_weight!(KWANGWOON, 2024),
            (University::INHA, 2024) => make_university_weight!(INHA, 2024),
            (University::ERICA, 2024) => make_university_weight!(ERICA, 2024),
            (University::SEJONG, 2024) => make_university_weight!(SEJONG, 2024),
            (University::KOOKMIN, 2024) => make_university_weight!(KOOKMIN, 2024),
            (University::AJU, 2024) => make_university_weight!(AJU, 2024),
            (University::SOONGSIL, 2024) => make_university_weight!(SOONGSIL, 2024),
            (University::KONKUK, 2024) => make_university_weight!(KONKUK, 2024),
            (University::CATHOLIC, 2024) => make_university_weight!(CATHOLIC, 2024),
            // 2025
            (University::SOGANG, 2025) => make_university_weight!(SOGANG, 2025),
            (University::CHUNGANG, 2025) => make_university_weight!(CHUNGANG, 2025),
            (University::KYUNGHEE, 2025) => make_university_weight!(KYUNGHEE, 2025),
            (University::SEOUL, 2025) => make_university_weight!(SEOUL, 2025),
            (University::KONKUK, 2025) => make_university_weight!(KONKUK, 2025),
            (University::DONGGUK, 2025) => make_university_weight!(DONGGUK, 2025),
            _ => unimplemented!(),
        }
    }

    pub fn korean(&self) -> f64 {
        self.korean
    }

    pub fn math(&self) -> f64 {
        self.math
    }

    pub fn english(&self) -> f64 {
        self.english
    }

    pub fn science(&self) -> f64 {
        self.science
    }

    pub fn science_required(&self) -> usize {
        self.science_required
    }

    pub fn english_required(&self) -> usize {
        self.english_required
    }

    pub fn english_table(&self) -> &Vec<f64> {
        &self.english_table
    }
}
