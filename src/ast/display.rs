use super::*;
use crate::{MAX_LINE_WIDTH, TAB_SIZE};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter};

impl Display for RonFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let RonFile(extensions, value) = self;
        if !extensions.is_empty() {
            writeln!(f, "#![enable({})]", extensions.iter().join(", "));
        }

        write!(f, "{}", value.to_string_rec(0))
    }
}

fn space(level: usize) -> String {
    " ".repeat(unsafe { TAB_SIZE } * level)
}

fn add_space(s: String) -> String {
    format!("{} ", s)
}

fn maybe_add_newline(s: String) -> String {
    if s.ends_with("\n") {
        s.to_owned()
    } else {
        format!("{}\n", s)
    }
}

impl Commented {
    fn post_string(&self, tabs: usize) -> String {
        match &self.post {
            None => "".into(),
            Some(v) => v
                .clone()
                .into_iter()
                .map(|s| space(tabs + 1) + &maybe_add_newline(s))
                .collect::<String>(),
        }
    }
    fn pre_string(&self, tabs: usize) -> String {
        match &self.pre {
            None => "".into(),
            Some(v) => v
                .clone()
                .into_iter()
                .map(|s| space(tabs + 1) + &maybe_add_newline(s))
                .collect::<String>(),
        }
    }

    fn pre_string_single(&self) -> String {
        match &self.pre {
            None => "".into(),
            Some(v) => v.clone().into_iter().map(add_space).collect::<String>(),
        }
    }
    fn post_string_single(&self) -> String {
        match &self.post {
            None => "".into(),
            Some(v) => v.clone().into_iter().map(add_space).collect::<String>(),
        }
    }

    fn to_string_rec(&self, tabs: usize) -> String {
        match (&self.pre, &self.post, &self.eol) {
            (Some(_), _, _) | (_, Some(_), _) | (_, _, Some(_)) => format!(
                "{}{}{}",
                self.pre_string(tabs),
                self.value.multiline(tabs),
                self.post_string(tabs)
            ),
            _ => self.value.to_string_rec(tabs),
        }
    }
    fn single_line(&self) -> String {
        // format!(
        //     "--{}{}{}--",
        //     self.pre_string_single(),
        //     self.value.single_line(),
        //     self.post_string_single()
        // )
        match (&self.pre, &self.post, &self.eol) {
            (Some(_), _, _) | (_, Some(_), _) | (_, _, Some(_)) => format!(
                "{}{}{}",
                self.pre_string_single(),
                self.value.single_line(),
                self.post_string_single()
            ),
            _ => self.value.single_line(),
        }
    }
}

impl Value {
    fn to_string_rec(&self, tabs: usize) -> String {
        if self.2 || tabs * unsafe { TAB_SIZE } + self.0 > unsafe { MAX_LINE_WIDTH } {
            self.multiline(tabs)
        } else {
            self.single_line()
        }
    }

    fn multiline(&self, tabs: usize) -> String {
        match &self.1 {
            Kind::Atom(atom) => atom.clone(),

            Kind::List(values) => {
                let elements = values
                    .iter()
                    .map(|e| space(tabs + 1) + &e.to_string_rec(tabs + 1) + ",\n")
                    .collect::<String>();

                format!("[\n{}{}]", elements, space(tabs))
            }

            Kind::Map(entries) => {
                let entries = entries
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}{}: {},{}\n",
                            v.pre_string(tabs),
                            space(tabs + 1) + &k.to_string_rec(tabs + 1),
                            v.value.to_string_rec(tabs + 1),
                            v.post_string(tabs),
                        )
                    })
                    .collect::<String>();

                format!("{{\n{}{}}}", entries, space(tabs))
            }

            Kind::TupleType(ident, values) => {
                let ident = ident.clone().unwrap_or_default();
                let elements = values
                    .iter()
                    .map(|e| space(tabs + 1) + &e.to_string_rec(tabs + 1) + ",\n")
                    .collect::<String>();

                format!("{}(\n{}{})", ident, elements, space(tabs))
            }

            Kind::FieldsType(ident, fields) => {
                let ident = ident.clone().unwrap_or_default();
                let fields = fields
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}{}: {},{}\n",
                            v.pre_string(tabs),
                            space(tabs + 1) + &k,
                            v.value.to_string_rec(tabs + 1),
                            v.post_string(tabs)
                        )
                    })
                    .collect::<String>();

                format!("{}(\n{}{})", ident, fields, space(tabs))
            }
        }
    }

    fn single_line(&self) -> String {
        match &self.1 {
            Kind::Atom(atom) => atom.clone(),

            Kind::List(elements) => {
                format!("[{}]", elements.iter().map(|e| e.single_line()).join(", "))
            }

            Kind::Map(entries) => format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.single_line(), v.single_line()))
                    .join(", ")
            ),

            Kind::TupleType(ident, elements) => {
                let ident = ident.clone().unwrap_or_default();
                format!(
                    "{}({})",
                    ident,
                    elements.iter().map(|e| e.single_line()).join(", ")
                )
            }

            Kind::FieldsType(ident, fields) => {
                let ident = ident.clone().unwrap_or_default();
                let fields = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.single_line()))
                    .join(", ");
                format!("{}({})", ident, fields)
            }
        }
    }
}
