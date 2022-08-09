use std::{collections::BTreeMap, convert::identity};

use anyhow::Result;
use log::{debug, warn};
use regex::{Captures, Regex, RegexBuilder};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Default, Serialize)]
pub struct Parsed {
    pub program: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: Option<String>,
}

pub fn to_re(format: &str) -> String {
    let mut ret = String::new();
    let mut escape = false;
    for c in format.chars() {
        if escape {
            match c {
                '%' => ret.push('%'),
                'p' => ret.push_str(r"(?P<p>[^:[[:cntrl:]]]+)"),
                'f' => ret.push_str(r"(?P<f>[^:[[:cntrl:]]]+)"),
                'l' => ret.push_str(r"(?P<l>\d+)"),
                'c' => ret.push_str(r"(?P<c>\d+)"),
                'm' => ret.push_str(r"(?P<m>.*)"),
                _ => warn!("invalid format %{}", c),
            }
            escape = false;
        } else if c == '%' {
            escape = true;
        } else {
            ret.push(c);
        }
    }
    ret
}

#[derive(Debug)]
pub struct Parser {
    regexes: Vec<Regex>,
}

impl Parser {
    pub fn new<I, S>(patterns: I) -> Result<Self>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let regexes = patterns
            .into_iter()
            .map(|pat| {
                RegexBuilder::new(&to_re(pat.as_ref()))
                    .multi_line(true)
                    .build()
            })
            .collect::<Result<Vec<_>, regex::Error>>()?;
        Ok(Self { regexes })
    }

    pub fn parse(&self, text: &str) -> Vec<Parsed> {
        let mut captures = BTreeMap::<(usize, usize), Captures>::new();
        for regex in &self.regexes {
            for cap in regex.captures_iter(text) {
                let mat = cap.get(0).unwrap();
                captures.entry((mat.start(), mat.end())).or_insert(cap);
            }
        }
        captures
            .values()
            .map(|cap| {
                debug!("{:?}", cap);
                let gets = |name: &str| cap.name(name).map(|m| m.as_str().to_string());
                let geti = |name: &str| {
                    cap.name(name)
                        .map(|m| m.as_str().parse().ok())
                        .and_then(identity)
                };
                Parsed {
                    program: gets("p"),
                    file: gets("f"),
                    line: geti("l"),
                    column: geti("c"),
                    message: gets("m"),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{to_re, Parsed, Parser};
    use test_log::test;

    #[test]
    fn re() {
        assert_eq!(to_re("%%"), "%");
        assert_eq!(to_re("%m"), r"(?P<m>.*)");
        assert_eq!(to_re("%%%m%%%"), r"%(?P<m>.*)%");
    }

    #[test]
    fn empty() {
        assert!(Parser::new(Vec::<String>::new())
            .unwrap()
            .parse("")
            .is_empty());
    }

    #[test]
    fn gnu() {
        let formats = ["^%f:%l:%c: %m$"];
        let text = r#"prog.cc:2:5: error: use of undeclared identifier 'std'
    std::cout << "hello world" << std::endl;
    ^
prog.cc:2:35: error: use of undeclared identifier 'std'
    std::cout << "hello world" << std::endl;
                                  ^
2 errors generated.
"#;
        assert_eq!(
            Parser::new(formats).unwrap().parse(text),
            vec![
                Parsed {
                    file: Some("prog.cc".to_string()),
                    line: Some(2),
                    column: Some(5),
                    message: Some("error: use of undeclared identifier 'std'".to_string()),
                    ..Default::default()
                },
                Parsed {
                    file: Some("prog.cc".to_string()),
                    line: Some(2),
                    column: Some(35),
                    message: Some("error: use of undeclared identifier 'std'".to_string()),
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn multi_pattern() {
        let formats = [r"^%f:%l:%c: %m$", r"^%f:%l: %m$"];
        let text = r#"prog.cc:1: error: expected unqualified-id
prog.cc:1:1: error: expected unqualified-id
"#;
        assert_eq!(
            Parser::new(formats).unwrap().parse(text),
            vec![
                Parsed {
                    file: Some("prog.cc".to_string()),
                    line: Some(1),
                    message: Some("error: expected unqualified-id".to_string()),
                    ..Default::default()
                },
                Parsed {
                    file: Some("prog.cc".to_string()),
                    line: Some(1),
                    column: Some(1),
                    message: Some("error: expected unqualified-id".to_string()),
                    ..Default::default()
                },
            ]
        );
    }
}
