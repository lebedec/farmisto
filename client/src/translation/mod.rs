use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::mem::{forget, take};
use std::sync::Arc;

use log::error;
//
// lazy_static! {
//     static ref TRANSLATOR_DATA: TranslatorData = TranslatorData::default();
// }

#[derive(Default)]
pub struct TranslatorData {
    translations: HashMap<String, Translation>,
}

#[derive(Default)]
struct Translation {
    messages: Vec<String>,
}

#[derive(Clone)]
pub struct Translator {
    data: Arc<RefCell<TranslatorData>>,
}

fn index_en_plural(n: usize) -> usize {
    if n == 1 {
        0
    } else {
        1
    }
}

fn index_ru_plural(n: usize) -> usize {
    if n % 10 == 1 && n % 100 != 11 {
        0
    } else if n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14) {
        1
    } else {
        2
    }
}

impl Translator {
    pub fn new(path: &str) -> Translator {
        let file = File::open(path).unwrap();
        let mut reader = io::BufReader::new(file).lines();
        let mut data = TranslatorData::default();
        let mut translation_key = String::new();
        let mut translation = Translation::default();
        while let Some(Ok(line)) = reader.next() {
            if line.starts_with("msgid_plural ") {
                continue;
            }
            if line.starts_with("msgid ") {
                data.translations
                    .insert(translation_key, take(&mut translation));
                translation_key = line
                    .strip_prefix("msgid ")
                    .unwrap()
                    .replace("\"", "")
                    .to_string();
            }
            if line.starts_with("msgstr") {
                let (_, message) = line.split_once(" ").unwrap();
                translation
                    .messages
                    .push(message.replace("\"", "").to_string());
            }
        }
        data.translations.insert(translation_key, translation);
        let translator = Translator {
            data: Arc::new(RefCell::new(data)),
        };
        translator
    }

    pub fn say(&self, message: &str) -> String {
        self.translate_with_params(message, None, &[])
    }

    pub fn translate_plural(&self, message: &str, n: usize) -> String {
        self.translate_with_params(message, Some(n), &[])
    }

    pub fn format<'a, const N: usize>(&self, message: &str, params: [impl ToString; N]) -> String {
        self.translate_with_params(message, None, &params.map(|value| value.to_string()))
    }

    pub fn format_plural(&self, message: &str, n: usize, params: &[String]) -> String {
        self.translate_with_params(message, Some(n), params)
    }

    pub fn translate_with_params(
        &self,
        message: &str,
        n: Option<usize>,
        params: &[String],
    ) -> String {
        let fallback = Translation {
            messages: vec![message.to_string()],
        };
        let data = self.data.borrow();
        let translation = match data.translations.get(message) {
            Some(translation) => translation,
            None => {
                error!("Unable to translate message not found: {message}");
                &fallback
            }
        };
        let index = match n {
            None => 0,
            Some(n) => index_ru_plural(n),
        };
        let mut text = translation.messages[index].clone();
        for i in 1..=params.len() {
            let placeholder = format!("${i}");
            let value = &params[i - 1];
            text = text.replace(&placeholder, value);
        }
        text
    }
}
