use cosmwasm_std::{Attribute, Uint128};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::iter::zip;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteAllResult {
    pub attributes: Vec<Vec<Attribute>>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryAllResult<T> {
    pub responses: Vec<T>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AnswerCheck {
    pub answer_type: String,
    pub lesson: String,
    pub chapter: String,
    pub result: String,
    pub errors: Vec<String>,
    pub differences: Vec<Difference>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Difference {
    pub yours: String,
    pub expected: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Freight {
    pub denom: String,
    pub amount: Uint128,
    pub unit_weight: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata {
    pub unit_denom: String,
    pub price: u128,
    pub name: Option<String>,
    pub freight: Vec<Freight>,
    pub health: u128,
    pub fuel: u128,
}

pub trait Result {
    fn print_results(&self);
    fn write_answer_to_file(&self, path: &str)
        where
            Self: Serialize,
    {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        serde_json::to_writer_pretty(&file, &self).unwrap();
    }
    fn check_answer(&self, lesson: &str, chapter: &str, correct_answer_path: &str) -> AnswerCheck;
}

impl Result for ExecuteAllResult {
    fn print_results(&self) {
        for attr in &self.attributes {
            println!("{}", serde_json::to_string(attr).unwrap());
        }
    }
    fn check_answer(&self, lesson: &str, chapter: &str, correct_answer_path: &str) -> AnswerCheck {
        let mut differences: Vec<Difference> = vec![];

        if !self.errors.is_empty() {
            return AnswerCheck {
                answer_type: "execute".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "error".to_string(),
                errors: vec![],
                differences: vec![],
            };
        }

        let content: String = std::fs::read_to_string(correct_answer_path)
            .unwrap()
            .parse()
            .unwrap();
        let correct_answer: ExecuteAllResult = serde_json::from_str(&content).unwrap();

        for (yours, expected) in zip(&self.attributes, &correct_answer.attributes) {
            if yours != expected {
                differences.push(Difference {
                    yours: serde_json::to_string(&yours).unwrap(),
                    expected: serde_json::to_string(&expected).unwrap(),
                });
            }
        }

        if &correct_answer == self {
            AnswerCheck {
                answer_type: "execute".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "success".to_string(),
                errors: vec![],
                differences: vec![],
            }
        } else {
            AnswerCheck {
                answer_type: "execute".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "incorrect".to_string(),
                errors: self.errors.clone(),
                differences,
            }
        }
    }
}

impl<T> Result for QueryAllResult<T>
    where
        T: Debug + DeserializeOwned + PartialEq + Serialize + Clone,
{
    fn print_results(&self) {
        for result in &self.responses {
            println!("{:?}", result);
        }
    }

    fn check_answer(&self, lesson: &str, chapter: &str, correct_answer_path: &str) -> AnswerCheck {
        let mut differences: Vec<Difference> = vec![];

        if !self.errors.is_empty() {
            return AnswerCheck {
                answer_type: "query".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "error".to_string(),
                errors: vec![],
                differences: vec![],
            };
        }

        let content: String = std::fs::read_to_string(correct_answer_path)
            .unwrap()
            .parse()
            .unwrap();
        let correct_answer: QueryAllResult<T> = serde_json::from_str(&content).unwrap();

        for (yours, expected) in zip(&self.responses, &correct_answer.responses) {
            if yours != expected {
                differences.push(Difference {
                    yours: serde_json::to_string(&yours).unwrap(),
                    expected: serde_json::to_string(&expected).unwrap(),
                })
            }
        }
        if &correct_answer == self {
            AnswerCheck {
                answer_type: "query".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "success".to_string(),
                errors: vec![],
                differences: vec![],
            }
        } else {
            AnswerCheck {
                answer_type: "query".to_string(),
                lesson: lesson.to_string(),
                chapter: chapter.to_string(),
                result: "incorrect".to_string(),
                errors: self.errors.clone(),
                differences,
            }
        }
    }
}

impl AnswerCheck {
    pub fn write_to_file(&self, file_path: &str) {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
            .unwrap();
        serde_json::to_writer_pretty(&file, &self).unwrap();
    }

    pub fn print_serialized(&self) {
        println!(
            "{}",
            serde_json::to_string(self)
                .unwrap()
                .replace('\\', "")
                .replace("\"[", "[")
                .replace("]\"", "]")
                .replace('\\', "")
                .replace("\"{", "{")
                .replace("}\"", "}")
                .replace('\n', "")
        );
    }
}
