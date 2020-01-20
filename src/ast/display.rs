use crate::ast::{FileText, RonValue, TextFragment};
use crate::config;
use itertools::Itertools;
use std::fmt::Write;

impl FileText {
    pub fn pretty_print(&self, config: &config::Config) -> String {
        let mut output = String::new();

        for item in &self.ron_text {
            writeln!(output, "{}", item.format(config))
                .expect("unable to write formatted RON output");
        }

        output
    }
}

impl TextFragment {
    fn format(&self, config: &config::Config) -> String {
        self.format_rec(0, config)
    }

    fn format_rec(
        &self,
        indent_level: usize,
        config: &config::Config,
    ) -> String {
        let projected_width =
            indent_level * config.soft_tab_width + self.minimum_length;

        if let RonValue::ExtensionBlock(exts) = &self.ron_value {
            format!("#![enable({})]", exts.join(", "))
        } else if projected_width > config.max_line_width {
            self.to_multiline(indent_level, config)
        } else {
            self.to_single_line()
        }
    }

    fn to_multiline(
        &self,
        indent_level: usize,
        config: &config::Config,
    ) -> String {
        fn indent(level: usize, config: &config::Config) -> String {
            " ".repeat(config.soft_tab_width * level)
        }

        match &self.ron_value {
            RonValue::ExtensionBlock(_) => unreachable!(),
            RonValue::Atom(value) => value.clone(),

            RonValue::List(elements) => {
                let list_inner = elements
                    .iter()
                    .map(|elem| {
                        format!(
                            "{indent}{elem},\n",
                            indent = indent(indent_level + 1, config),
                            elem = elem.format_rec(indent_level + 1, config)
                        )
                    })
                    .collect::<String>();

                format!(
                    "[\n{list_inner}{indent}]",
                    list_inner = list_inner,
                    indent = indent(indent_level, config)
                )
            }

            RonValue::Map(entries) => {
                let map_inner = entries
                    .iter()
                    .map(|(key, val)| {
                        format!(
                            "{indent}{key}: {val},\n",
                            indent = indent(indent_level + 1, config),
                            key = key.format_rec(indent_level + 1, config),
                            val = val.format_rec(indent_level + 1, config)
                        )
                    })
                    .collect::<String>();

                format!(
                    "{{\n{map_inner}{indent}}}",
                    map_inner = map_inner,
                    indent = indent(indent_level, config)
                )
            }

            RonValue::TupleType {
                maybe_ident,
                elements,
            } => {
                let ident = maybe_ident.clone().unwrap_or_default();
                let tuple_inner = elements
                    .iter()
                    .map(|elem| {
                        format!(
                            "{indent}{elem},\n",
                            indent = indent(indent_level + 1, config),
                            elem = elem.format_rec(indent_level + 1, config)
                        )
                    })
                    .collect::<String>();

                format!(
                    "{ident}(\n{tuple_inner}{indent})",
                    ident = ident,
                    tuple_inner = tuple_inner,
                    indent = indent(indent_level, config)
                )
            }

            RonValue::FieldsType {
                maybe_ident,
                fields,
            } => {
                let ident = maybe_ident.clone().unwrap_or_default();
                let fields_inner = fields
                    .iter()
                    .map(|(key, val)| {
                        format!(
                            "{indent}{key}: {val},\n",
                            indent = indent(indent_level + 1, config),
                            key = key,
                            val = val.format_rec(indent_level + 1, config)
                        )
                    })
                    .collect::<String>();

                format!(
                    "{ident}(\n{fields_inner}{indent})",
                    ident = ident,
                    fields_inner = fields_inner,
                    indent = indent(indent_level, config)
                )
            }
        }
    }

    fn to_single_line(&self) -> String {
        match &self.ron_value {
            RonValue::ExtensionBlock(_) => unreachable!(),
            RonValue::Atom(value) => value.clone(),

            RonValue::List(elements) => {
                let list_inner =
                    elements.iter().map(|e| e.to_single_line()).join(", ");

                format!("[{}]", list_inner)
            }

            RonValue::Map(entries) => {
                let map_inner = entries
                    .iter()
                    .map(|(key, val)| {
                        format!(
                            "{key}: {val}",
                            key = key.to_single_line(),
                            val = val.to_single_line()
                        )
                    })
                    .join(", ");

                format!("{{{}}}", map_inner)
            }

            RonValue::TupleType {
                maybe_ident,
                elements,
            } => {
                let ident = maybe_ident.clone().unwrap_or_default();
                let tuple_inner =
                    elements.iter().map(|e| e.to_single_line()).join(", ");

                format!(
                    "{ident}({tuple_inner})",
                    ident = ident,
                    tuple_inner = tuple_inner
                )
            }

            RonValue::FieldsType {
                maybe_ident,
                fields,
            } => {
                let ident = maybe_ident.clone().unwrap_or_default();
                let fields = fields
                    .iter()
                    .map(|(key, val)| {
                        format!(
                            "{key}: {val}",
                            key = key,
                            val = val.to_single_line()
                        )
                    })
                    .join(", ");

                format!("{ident}({fields})", ident = ident, fields = fields)
            }
        }
    }
}
