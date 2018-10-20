use super::{Kind, Node};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter, Write};

const TAB_SIZE: usize = 4;
const MAX_LINE_WIDTH: usize = 40;

fn indent(level: usize) -> String {
    " ".repeat(TAB_SIZE * level)
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_rec(f, 0, false);
        Ok(())
    }
}

impl Node {
    fn fmt_rec(&self, f: &mut dyn Write, indent_level: usize, indent_first_line: bool) {
        if indent_first_line {
            write!(f, "{}", indent(indent_level));
        }

        if self.0 + TAB_SIZE * indent_level > MAX_LINE_WIDTH {
            self.fmt_multiline(f, indent_level);
        } else {
            write!(f, "{}", self.fmt_singleline());
        };
    }

    fn fmt_multiline(&self, f: &mut dyn Write, indent_level: usize) {
        match &self.1 {
            Kind::RonFile(extensions, value) => {
                if !extensions.is_empty() {
                    writeln!(f, "#![enable({})]", extensions.iter().join(", "));
                }

                value.fmt_rec(f, 0, false);
            }

            Kind::Atom(atom) => {
                write!(f, "{}", atom);
            }

            Kind::List(elements) => {
                writeln!(f, "[");

                for elem in elements {
                    elem.fmt_rec(f, indent_level + 1, true);
                    writeln!(f, ",");
                }

                write!(f, "{}]", indent(indent_level));
            }

            Kind::Map(entries) => {
                writeln!(f, "{{");

                for (key, value) in entries {
                    key.fmt_rec(f, indent_level + 1, true);
                    write!(f, ": ");
                    value.fmt_rec(f, indent_level + 1, false);
                    writeln!(f, ",");
                }

                write!(f, "{}}}", indent(indent_level));
            }

            Kind::TupleType(ident, elements) => {
                if let Some(ident) = ident {
                    write!(f, "{}", ident);
                }
                writeln!(f, "(");

                for elem in elements {
                    elem.fmt_rec(f, indent_level + 1, true);
                    writeln!(f, ",");
                }

                write!(f, "{})", indent(indent_level));
            }

            Kind::FieldsType(ident, fields) => {
                if let Some(ident) = ident {
                    write!(f, "{}", ident);
                }
                writeln!(f, "(");

                for (key, value) in fields {
                    write!(f, "{indent}{}: ", key, indent = indent(indent_level + 1));
                    value.fmt_rec(f, indent_level + 1, false);
                    writeln!(f, ",");
                }

                write!(f, "{})", indent(indent_level));
            }
        }
    }

    fn fmt_singleline(&self) -> String {
        match &self.1 {
            Kind::RonFile(extensions, value) => {
                if !extensions.is_empty() {
                    let mut buf = format!("#![enable({})]\n", extensions.iter().join(", "));
                    value.fmt_rec(&mut buf, 0, false);
                    buf
                } else {
                    let mut buf = String::new();
                    value.fmt_rec(&mut buf, 0, false);
                    buf
                }
            }

            Kind::Atom(atom) => atom.clone(),

            Kind::List(elements) => format!("[{}]", elements.iter().join(", ")),

            Kind::Map(entries) => format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.fmt_singleline(), v.fmt_singleline()))
                    .join(", ")
            ),

            Kind::TupleType(ident, elements) => {
                if let Some(ident) = ident {
                    format!("{}({})", ident, elements.iter().join(", "))
                } else {
                    format!("({})", elements.iter().join(", "))
                }
            }

            Kind::FieldsType(ident, fields) => {
                if let Some(ident) = ident {
                    format!(
                        "{}({})",
                        ident,
                        fields
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v.fmt_singleline()))
                            .join(", ")
                    )
                } else {
                    format!(
                        "({})",
                        fields
                            .iter()
                            .map(|(k, v)| format!("{}: {}", k, v.fmt_singleline()))
                            .join(", ")
                    )
                }
            }
        }
    }
}
