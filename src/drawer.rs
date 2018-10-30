use parser::{Parser, Error};
use regex::Regex;
use std::collections::HashMap;
use std::fmt;

const DRAWER_END: &str = ":END:";
const PROPERTIES_DRAWER_NAME: &str = "PROPERTIES";

/// See <https://orgmode.org/worg/dev/org-syntax.html#Property_Drawers>
#[derive(Debug, Serialize, Deserialize)]
pub struct Drawer {
    pub name: String,
    pub contents: Vec<String>
}

impl Drawer {
    pub fn parse(line: &str, lines: &mut Parser) -> Result<Option<Drawer>, Error> {
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

    pub fn from_properties(properties: &HashMap<String, String>) -> Self {
        let name_width = properties.keys().map(|name| name.len()).max().unwrap_or(0);

        Drawer {
            name: PROPERTIES_DRAWER_NAME.to_string(),
            contents: properties.iter()
                .map(|(name, value)| {
                    let name = format!(":{}:", name);
                    format!("{:width$}{}", name, value, width = name_width + 3)
                })
                .collect()
        }
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

impl fmt::Display for Drawer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ":{}:\n{}\n:END:", self.name, self.contents.join("\n"))
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
