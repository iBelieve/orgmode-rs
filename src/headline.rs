use regex::Regex;

/// A headline is defined as `STARS KEYWORD PRIORITY TITLE TAGS`
///
/// See <https://orgmode.org/worg/dev/org-syntax.html#Headlines_and_Sections>
#[derive(Debug, PartialEq, Default)]
pub struct Headline {
    pub indent: u16,
    pub keyword: Option<String>,
    pub priority: Option<String>,
    pub is_commented: bool,
    pub title: String,
    pub tags: Vec<String>
}

impl Headline {
    pub(super)fn parse(line: &str, keywords: &[String]) -> Option<Self> {
        if is_headline(line) {
            let (indent, text) = line.split_at(line.find(' ').unwrap());
            let indent = indent.len() as u16;
            let text = text.trim();

            let keyword = keywords.iter()
                .find(|keyword| text.starts_with(&((*keyword).clone() + " ")))
                .map(|keyword| keyword.to_string());
            let text = if let Some(ref keyword) = keyword {
                text[keyword.len() + 1..].trim()
            } else {
                text
            };

            lazy_static! {
                static ref REGEX: Regex = Regex::new(r#"(?x)
                    ^
                    (\[\#(?P<priority>.)\])?
                    \s*
                    (?P<comment>COMMENT(\s|$))?
                    (?P<title>.*?)
                    \s*
                    (\s+(?P<tags>:([a-zA-Z0-9_@\#%]*:)+))?
                    $
                "#).unwrap();
            }

            let captures = REGEX.captures(text).unwrap();

            let priority = captures.name("priority").map(|c| c.as_str().to_string());
            let is_commented = captures.name("comment").is_some();
            let title = captures.name("title").unwrap().as_str().to_string();
            let tags = if let Some(tags) = captures.name("tags").map(|c| c.as_str()) {
                tags[1..tags.len() - 1].split(':')
                    .filter(|tag| !tag.is_empty())
                    .map(|tag| tag.to_string())
                    .collect()
            } else {
                Vec::new()
            };

            Some(Headline {
                indent,
                keyword,
                priority,
                is_commented,
                title,
                tags
            })
        } else {
            None
        }
    }
}

fn is_headline(line: &str) -> bool {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("^\\*+\\s").unwrap();
    }

    REGEX.is_match(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_headline() {
        assert!(is_headline("* My header"));
        assert!(is_headline("* TODO [#A] My header"));
        assert!(is_headline("* DONE [#C] My header :PROJECT:"));
        assert!(is_headline("** A subheader"));
        assert!(!is_headline("*Bold text*"));
        assert!(!is_headline(" * List item 1"));
    }

    #[test]
    fn test_parse_basic_headline() {
        assert_eq!(Headline::parse("* My header", &["TODO".to_string(), "DOING".to_string(), "DONE".to_string()]),
                   Some(Headline {
                       indent: 1,
                       keyword: None,
                       priority: None,
                       is_commented: false,
                       title: "My header".to_string(),
                       tags: vec![]
                   }));
    }

    #[test]
    fn test_parse_full_headline() {
        assert_eq!(Headline::parse("** DOING [#C] Comment about my header :TAG1_%::@TAG2::", &["TODO".to_string(), "DOING".to_string(), "DONE".to_string()]),
                   Some(Headline {
                       indent: 2,
                       keyword: Some("DOING".to_string()),
                       priority: Some("C".to_string()),
                       is_commented: false,
                       title: "Comment about my header".to_string(),
                       tags: vec!["TAG1_%".to_string(), "@TAG2".to_string()]
                   }));
    }

    #[test]
    fn test_parse_commented_headline() {
        assert_eq!(Headline::parse("**** TODO [#A] COMMENT Title :tag:a2%:", &["TODO".to_string(), "DOING".to_string(), "DONE".to_string()]),
                   Some(Headline {
                       indent: 4,
                       keyword: Some("TODO".to_string()),
                       priority: Some("A".to_string()),
                       is_commented: true,
                       title: "Title".to_string(),
                       tags: vec!["tag".to_string(), "a2%".to_string()]
                   }));
    }
}
