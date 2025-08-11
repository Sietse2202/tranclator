use arboard::Clipboard;
use clap::Parser;
use indexmap::map::IndexMap;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{ErrorKind, Write};

#[derive(Parser, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Cli {
    #[clap(long, help = "Text to translate", conflicts_with = "repl")]
    text: Option<String>,
    #[clap(long, help = "Run in REPL mode")]
    repl: bool,
    #[clap(long, default_value = "tranclator.toml", help = "Path to config file")]
    config_path: String,
    #[clap(short, long, help = "Language to use")]
    language: Option<String>,
    #[clap(short, long, help = "Do not copy to clipboard")]
    no_clipboard: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct Config {
    global: Option<Global>,
    #[serde(rename = "language", default)]
    languages: Vec<Language>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
struct Global {
    default_language: Option<String>,
    copy_to_clipboard: Option<bool>,
    quit_keywords: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
struct Language {
    name: String,
    lower_mode: CapitalizationMode,
    dict: IndexMap<String, String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum CapitalizationMode {
    Lower,
    Preserve,
    Upper,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let result = std::fs::read_to_string(&args.config_path);

    let Ok(str) = result else {
        match result.unwrap_err().kind() {
            ErrorKind::NotFound => println!("Could not find `{}`", &args.config_path),
            _ => println!("Could not read config file"),
        }

        return Ok(());
    };

    let Ok(config) = toml::from_str::<Config>(&str) else {
        println!("Could not parse config file");
        return Ok(());
    };

    let mut cb = if args.no_clipboard {
        None
    } else {
        Some(Clipboard::new()?)
    };

    let Some(language) = args
        .language
        .or_else(|| config.global.as_ref()?.default_language.clone())
    else {
        println!("No language specified");
        return Ok(());
    };

    let Some(language) = config.languages.iter().find(|l| l.name == *language) else {
        println!("Language {} not found", language);
        return Ok(());
    };

    if let Some(text) = args.text {
        let translated = translate(&text, language);
        println!("{}", translated);

        if let Some(ref mut cb) = cb {
            cb.set_text(&translated)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        return Ok(());
    } else if args.repl {
        repl(
            language,
            cb,
            HashSet::from_iter(
                config
                    .global
                    .and_then(|c| c.quit_keywords)
                    .unwrap_or_default(),
            ),
        )?;
        return Ok(());
    }

    Ok(())
}

fn repl(
    language: &Language,
    mut cb: Option<Clipboard>,
    quit_words: HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to {} REPL", language.name);
    println!(
        "Type any of {} to exit",
        quit_words
            .iter()
            .map(|w| format!("\"{w}\""))
            .collect::<Vec<String>>()
            .join(", ")
    );

    loop {
        print!(">>> ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if quit_words.contains(input.trim()) {
            break Ok(());
        }

        let translated = translate(&input, language);
        println!("{translated}");

        if let Some(ref mut cb) = cb {
            cb.set_text(&translated)?;
        }
    }
}

fn translate(text: &str, language: &Language) -> String {
    let text = text.trim();
    let mut text = text.to_string();

    match language.lower_mode {
        CapitalizationMode::Lower => {
            text = text.to_lowercase();
            for (word, translation) in &language.dict {
                text = text.replace(&word.to_lowercase(), &translation.to_lowercase());
            }
        }
        CapitalizationMode::Upper => {
            text = text.to_uppercase();
            for (word, translation) in &language.dict {
                text = text.replace(&word.to_uppercase(), &translation.to_uppercase());
            }
        }
        CapitalizationMode::Preserve => {
            for (word, translation) in &language.dict {
                let lower_word = word.to_lowercase();

                let matches: Vec<usize> = text
                    .to_lowercase()
                    .match_indices(&lower_word)
                    .map(|(pos, _)| pos)
                    .collect();

                for &pos in matches.iter().rev() {
                    let end_pos = pos + word.len();
                    let original_segment = &text[pos..end_pos];

                    let replacement = if original_segment.to_lowercase() == original_segment {
                        translation.to_lowercase()
                    } else if original_segment.to_uppercase() == original_segment
                        && text.to_uppercase() == text
                    {
                        translation.to_uppercase()
                    } else {
                        let mut c = translation.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().chain(c).collect(),
                        }
                    };

                    text.replace_range(pos..end_pos, &replacement);
                }
            }
        }
    };

    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let language = Language {
            name: "test".to_string(),
            lower_mode: CapitalizationMode::Lower,
            dict: vec![
                ("hello".to_string(), "hola".to_string()),
                ("world".to_string(), "mundo".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        assert_eq!(translate("hello world", &language), "hola mundo");
        assert_eq!(translate("Hello WorLd", &language), "hola mundo");
    }
}
