use super::{Node, Kind};
use itertools::Itertools;
use std::fmt::{self, Display, Formatter};

const TAB_SIZE: usize = 4;

fn indent(level: usize) -> String {
    " ".repeat(TAB_SIZE * level)
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_rec(f, 0, true);
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
                    writeln!(f, "#![enable({})]\n", extensions.iter().join(", "));
                }

                value.fmt_rec(f, 0, true);
            }

            Kind::Atom(atom) => {
                write!(f, "{}", atom);
            }

            Kind::Tuple(elements) => {
                writeln!(f, "(");
                for elem in elements {
                    elem.fmt_rec(f, indent_level + 1, true);
                    writeln!(f, ",");
                }
                write!(f, "{})", indent(indent_level));
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

            Kind::NamedTypeTuple(ident, elements) => {
                writeln!(f, "{}(", ident);
                for elem in elements {
                    elem.fmt_rec(f, indent_level + 1, true);
                    writeln!(f, ",");
                }
                write!(f, "{})", indent(indent_level));
            }

            Kind::NamedTypeFields(ident, fields) => {
                writeln!(f, "{}(", ident);
                for (field_name, field_value) in fields {
                    write!(
                        f,
                        "{indent}{}: ",
                        field_name,
                        indent = indent(indent_level + 1)
                    );
                    field_value.fmt_rec(f, indent_level + 1, false);
                    writeln!(f, ",");
                }
                write!(f, "{})", indent(indent_level));
            }

            Kind::AnonymousTypeFields(fields) => {
                writeln!(f, "(");
                for (field_name, field_value) in fields {
                    write!(
                        f,
                        "{indent}{}: ",
                        field_name,
                        indent = indent(indent_level + 1)
                    );
                    field_value.fmt_rec(f, indent_level + 1, false);
                    writeln!(f, ",");
                }
                write!(f, "{})", indent(indent_level));
            }
        }
    }
}
