use super::{Kind, Node};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter};

const TAB_SIZE: usize = 4;

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
    fn fmt_rec(&self, f: &mut Formatter, indent_level: usize, indent_first_line: bool) {
        if indent_first_line {
            write!(f, "{}", indent(indent_level));
        }

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
}
