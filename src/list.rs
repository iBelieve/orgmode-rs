use element::Element;
use itertools::Itertools;
use parser::Parser;
use regex::Regex;
use std::fmt;
use text::Text;
use utils::StringUtils;
use std::cmp::min;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ListKind {
    Unordered,
    OrderedNumber,
    OrderedLetter,
    Definition,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
enum Counter {
    Number(u32),
    Letter(char),
}

impl Counter {
    fn start(kind: ListKind) -> Counter {
        match kind {
            ListKind::Unordered | ListKind::Definition => Counter::Number(0),
            ListKind::OrderedNumber => Counter::Number(1),
            ListKind::OrderedLetter => Counter::Letter('a'),
        }
    }

    fn increment(&mut self) {
        match self {
            Counter::Number(ref mut number) => {
                *number += 1;
            }
            Counter::Letter(ref mut letter) => {
                if (*letter >= 'a' && *letter < 'z') || (*letter >= 'A' && *letter < 'Z') {
                    *letter = (*letter as u8 + 1) as char;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Checkbox {
    Unchecked,
    Checked,
    Partial,
}

#[derive(Serialize, Deserialize)]
pub struct List {
    #[serde(rename = "type")]
    kind: ListKind,
    items: Vec<ListItem>,
}

impl List {
    pub fn parse(line: &str, parser: &mut Parser) -> Option<List> {
        if let Some(ParsedListItem {
            indent: list_indent,
            kind: list_kind,
            item,
        }) = ListItem::parse(line)
        {
            let mut list = List {
                kind: list_kind,
                items: vec![item],
            };
            let mut was_empty_line = false;

            while let Some(line) = parser.peek().map(|line| line.to_string()) {
                let indent = line.indent();
                let item_count = list.items.len();

                if line.trim().is_empty() {
                    if was_empty_line {
                        break;
                    } else {
                        parser.next();
                        was_empty_line = true;
                        list.items[item_count - 1].add_line(&line);
                    }
                } else if indent < list_indent {
                    break;
                } else if indent == list_indent {
                    if let Some(item) = ListItem::parse(&line) {
                        parser.next();
                        list.items.push(item.item);
                    } else {
                        break;
                    }
                } else {
                    parser.next();

                    if let Some(element) = Element::parse(&line, parser) {
                        list.items[item_count - 1].elements.push(element);
                    } else {
                        list.items[item_count - 1].add_line(&line[min(line.indent(), indent + 1)..]);
                    }
                }
            }

            Some(list)
        } else {
            None
        }
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut counter = Counter::start(self.kind);
        let mut is_first = true;

        for item in self.items.iter() {
            if !is_first {
                write!(f, "\n")?;
            } else {
                is_first = false;
            }

            if let Some(counter_set) = item.counter {
                counter = counter_set;
            }

            match self.kind {
                ListKind::Unordered | ListKind::Definition => write!(f, " -")?,
                ListKind::OrderedNumber | ListKind::OrderedLetter => {
                    match counter {
                        Counter::Number(number) => write!(f, " {}.", number)?,
                        Counter::Letter(letter) => write!(f, " {}.", letter)?,
                    }

                    if let Some(counter_set) = item.counter {
                        match counter_set {
                            Counter::Number(number) => write!(f, " [@{}]", number)?,
                            Counter::Letter(letter) => write!(f, " [@{}]", letter)?,
                        }
                    }
                }
            }

            if let Some(checkbox) = item.checkbox {
                match checkbox {
                    Checkbox::Checked => write!(f, " [X]")?,
                    Checkbox::Unchecked => write!(f, " [ ]")?,
                    Checkbox::Partial => write!(f, " [-]")?,
                }
            }

            write!(f, " {}", item.text.format(3))?;

            if let Some(ref definition) = item.definition {
                write!(f, " :: {}", definition)?;
            }

            if !item.elements.is_empty() {
                let mut elements = item.elements.iter()
                    .map(|element| {
                        if let Element::List(list) = element {
                            list.to_string().lines().map(|line| &line[1..]).join("\n")
                        } else {
                            element.to_string()
                        }
                    })
                    .join("\n");
                let indent = match self.kind {
                    ListKind::Unordered | ListKind::Definition => 3,
                    ListKind::OrderedNumber | ListKind::OrderedLetter => 4,
                };
                write!(f, "\n{}", elements.add_indent(indent))?;
            }

            counter.increment();
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListItem {
    counter: Option<Counter>,
    checkbox: Option<Checkbox>,
    text: Text,
    definition: Option<String>,
    elements: Vec<Element>,
}

struct ParsedListItem {
    indent: usize,
    kind: ListKind,
    item: ListItem,
}

impl ListItem {
    fn parse(line: &str) -> Option<ParsedListItem> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(
                r#"(?x)
                ^
                (?P<indent>\s*)
                (?P<bullet>-|\+|\s\*|(?P<counter>[0-9]+|[a-zA-Z])[.\)])
                \s*
                (\[@(?P<counter_set>[0-9]+|[a-zA-Z])\]\s*)?
                (\[(?P<checkbox>[xX\-\s])\]\s*)?
                (?P<text>.*?)
                \s*
                (::\s*(?P<definition>.*?)\s*)?
                $
            "#
            )
                .unwrap();
        }

        if let Some(captures) = REGEX.captures(line) {
            let indent = captures.name("indent").unwrap().as_str().len();
            let bullet = captures.name("bullet").unwrap().as_str().trim();
            let indent = if bullet == "*" { indent + 1 } else { indent };
            let counter_set = captures.name("counter_set").map(|c| c.as_str());
            let checkbox = captures.name("checkbox").map(|c| c.as_str());
            let text = captures.name("text").unwrap().as_str();
            let definition = captures.name("definition").map(|c| c.as_str());

            let kind = if definition.is_some() {
                ListKind::Definition
            } else if bullet == "*" || bullet == "-" || bullet == "+" {
                ListKind::Unordered
            } else {
                if bullet[..bullet.len() - 1].chars().all(|c| c.is_numeric()) {
                    ListKind::OrderedNumber
                } else {
                    ListKind::OrderedLetter
                }
            };
            let counter_set = counter_set.map(|counter_set| {
                if let Ok(number) = counter_set.parse() {
                    Counter::Number(number)
                } else {
                    Counter::Letter(counter_set.chars().next().unwrap())
                }
            });
            let checkbox = checkbox.map(|checkbox| match checkbox {
                "x" | "X" => Checkbox::Checked,
                " " => Checkbox::Unchecked,
                "-" => Checkbox::Partial,
                checkbox => panic!("Unsupported checkbox symbol: {}", checkbox),
            });

            Some(ParsedListItem {
                indent,
                kind,
                item: ListItem {
                    counter: counter_set,
                    checkbox,
                    text: Text::new(text),
                    definition: definition.map(|s| s.to_string()),
                    elements: Vec::new(),
                },
            })
        } else {
            None
        }
    }

    fn add_line(&mut self, line: &str) {
        if self.elements.len() > 0 {
            if let Some(Element::Paragraph(paragraph)) = self.elements.last_mut() {
                if !paragraph.at_end() {
                    paragraph.add_line(line);
                    return;
                }
            }

            self.elements.push(Element::new_paragraph(line));
        } else {
            self.text.add_line(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_list_item() {
        assert!(ListItem::parse("1. Test").is_some());
        assert!(ListItem::parse("A) Test").is_some());
        assert!(ListItem::parse(" * Test").is_some());
        assert!(ListItem::parse(" - Test").is_some());
        assert!(ListItem::parse("Test").is_none());
        assert!(ListItem::parse("* Test").is_none());
    }
}
