use parser::{Parser, Error};
use regex::Regex;
use std::collections::HashMap;

const DRAWER_END: &str = ":END:";
const PROPERTIES_DRAWER_NAME: &str = "PROPERTIES";

/// See <https://orgmode.org/worg/dev/org-syntax.html#Property_Drawers>
#[derive(Debug)]
pub struct Drawer {
    pub name: String,
    pub contents: Vec<String>
}

impl Drawer {
    pub(super) fn parse(line: &str, lines: &mut Parser) -> Result<Option<Drawer>, Error> {
        let drawer = if let Some(name) = parse_drawername(line) {
            Some(Drawer {
                name: name.to_string(),
                contents: lines.take_until(DRAWER_END)?
            })
        } else {
            None
        };
        Ok(drawer)
    }

    /// See <https://orgmode.org/worg/dev/org-syntax.html#Node_Properties>
    pub fn is_properties_drawer(&self) -> bool {
        self.name == PROPERTIES_DRAWER_NAME
    }

    /// See <https://orgmode.org/worg/dev/org-syntax.html#Node_Properties>
    pub fn as_properties(&self) -> Option<HashMap<String, String>> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"^\s*:([^\s:]+):\s+(.*)$"#).unwrap();
        }
        if self.is_properties_drawer() {
            let properties = self.contents.iter()
                .filter_map(|line| {
                    if let Some(captures) = REGEX.captures(line) {
                        let name = captures.get(1).unwrap().as_str().to_string();
                        let value = captures.get(2).unwrap().as_str().to_string();
                        Some((name, value))
                    } else {
                        None
                    }
                })
                .collect();
            Some(properties)
        } else {
            None
        }
    }
}

fn parse_drawername(line: &str) -> Option<&str> {
    lazy_static! {
        static ref REGEX: Regex = Regex::new("^:([a-zA-Z0-9_\\-]+):$").unwrap();
    }

    if let Some(captures) = REGEX.captures(line) {
        Some(captures.get(1).unwrap().as_str())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_drawername() {
        assert_eq!(parse_drawername(":PROPERTIES:"), Some("PROPERTIES"));
        assert_eq!(parse_drawername(":LOGBOOK:"), Some("LOGBOOK"));
        assert_eq!(parse_drawername(":ANOTHER_drawer-2:"), Some("ANOTHER_drawer-2"));
        assert_eq!(parse_drawername(":INVALID DRAWER:"), None);
        assert_eq!(parse_drawername(":DRAWER: 123"), None);
    }
}
