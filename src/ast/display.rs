use super::{Kind, Node, RonFile};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter};

const TAB_SIZE: usize = 4;
const MAX_LINE_WIDTH: usize = 40;

fn indent(level: usize) -> String {
    " ".repeat(TAB_SIZE * level)
}

impl Display for RonFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let RonFile(extensions, value) = self;
        if !extensions.is_empty() {
            writeln!(f, "#![enable({})]", extensions.iter().join(", "));
        }

        value.fmt_rec(f, 0, false);
        Ok(())
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_rec(f, 0, false);
        Ok(())
    }
}

impl Node {
    fn fmt_rec(&self, f: &mut Formatter, indent_level: usize, indent_first_line: bool) {
        if indent_first_line {
            write!(f, "{}", indent(indent_level));
        }

        if TAB_SIZE * indent_level + self.0 > MAX_LINE_WIDTH {
            self.fmt_multiline(f, indent_level);
        } else {
            write!(f, "{}", self.fmt_single_line());
        }
    }

    fn fmt_multiline(&self, f: &mut Formatter, indent_level: usize) {
        match &self.1 {
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

    fn fmt_single_line(&self) -> String {
        match &self.1 {
            Kind::Atom(atom) => atom.clone(),

            Kind::List(elements) => format!("[{}]", elements.iter().join(", ")),

            Kind::Map(entries) => format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.fmt_single_line(), v.fmt_single_line()))
                    .join(", ")
            ),

            Kind::TupleType(ident, elements) => {
                let ident = ident.clone().unwrap_or_default();
                format!("{}({})", ident, elements.iter().join(", "))
            }

            Kind::FieldsType(ident, fields) => {
                let ident = ident.clone().unwrap_or_default();
                let fields = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.fmt_single_line()))
                    .join(", ");
                format!("{}({})", ident, fields)
            }
        }
    }
}
