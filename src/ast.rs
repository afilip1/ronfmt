mod display;

use crate::parser::Rule;
use pest::iterators::Pair;

// structured representation of a single RON file
pub struct FileText {
    ron_text: Vec<TextFragment>,
}

// wrapper over a RON value with minimum length required for proper formatting
pub struct TextFragment {
    minimum_length: usize,
    ron_value: RonValue,
}

// the actual underlying RON value text slice
pub enum RonValue {
    ExtensionBlock(Vec<String>),

    Atom(String), // atomic types: bool, char, str, int, float, unit type
    List(Vec<TextFragment>),
    Map(Vec<(TextFragment, TextFragment)>),
    TupleType {
        maybe_ident: Option<String>,
        elements: Vec<TextFragment>,
    },
    FieldsType {
        maybe_ident: Option<String>,
        fields: Vec<(String, TextFragment)>,
    },

    StandaloneComment(String),
    TrailingComment(String),
}

impl FileText {
    pub fn parse_from(pair: Pair<Rule>) -> Self {
        assert!(pair.as_rule() == Rule::ron_file);

        let ron_text = pair.into_inner().map(TextFragment::from).collect();

        FileText { ron_text }
    }
}

impl TextFragment {
    fn from(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::extension => {
                let exts = pair
                    .into_inner()
                    .map(|ext| ext.as_str().to_string())
                    .collect();

                TextFragment {
                    minimum_length: 0, //TODO: don't care about this for now
                    ron_value: RonValue::ExtensionBlock(exts),
                }
            }

            Rule::standalone => {
                let comment =
                    pair.into_inner().next().unwrap().as_str().to_string();

                TextFragment {
                    minimum_length: 0, //TODO: don't care for now
                    ron_value: RonValue::StandaloneComment(comment),
                }
            }

            Rule::trailing => {
                let comment =
                    pair.into_inner().next().unwrap().as_str().to_string();

                TextFragment {
                    minimum_length: 0, //TODO: don't care for now
                    ron_value: RonValue::TrailingComment(comment),
                }
            }

            Rule::bool
            | Rule::char
            | Rule::string
            | Rule::signed_int
            | Rule::float
            | Rule::unit_type => {
                let value = pair.as_str().to_string();

                TextFragment {
                    minimum_length: value.len(),
                    ron_value: RonValue::Atom(value),
                }
            }

            Rule::list => {
                let elements: Vec<_> =
                    pair.into_inner().map(TextFragment::from).collect();

                // N elements requires N-1 ", " (N-1 * 2 chars) + 1 "[]" (2 chars)
                // e.g. [1, 2, 3]
                // N-1 * 2 + 2 == N * 2 extra chars, or +2 chars per element
                let minimum_length =
                    elements.iter().map(|elem| elem.minimum_length + 2).sum();

                TextFragment {
                    minimum_length,
                    ron_value: RonValue::List(elements),
                }
            }

            Rule::map => {
                let entries: Vec<_> = pair
                    .into_inner()
                    .map(|entry| {
                        let mut kv_iter = entry.into_inner();
                        let key = TextFragment::from(kv_iter.next().unwrap());
                        let val = TextFragment::from(kv_iter.next().unwrap());
                        (key, val)
                    })
                    .collect();

                // N entries requires N ": " (N * 2 chars) + N-1 ", " (N-1 * 2 chars) + 1 "{}" (2 chars)
                // e.g. {"a": 1, "b": 2, "c": 3}
                // N * 2 + N-1 * 2 + 2 == N * 4 extra chars, or +4 chars per entry
                let minimum_length = entries
                    .iter()
                    .map(|(key, val)| {
                        key.minimum_length + val.minimum_length + 4
                    })
                    .sum();

                TextFragment {
                    minimum_length,
                    ron_value: RonValue::Map(entries),
                }
            }

            Rule::tuple_type => {
                let mut tokens_iter = pair.into_inner();
                let maybe_ident = match tokens_iter.peek().map(|p| p.as_rule())
                {
                    Some(Rule::ident) => {
                        let ident =
                            tokens_iter.next().unwrap().as_str().to_string();
                        Some(ident)
                    }
                    _ => None,
                };

                let elements: Vec<_> =
                    tokens_iter.map(TextFragment::from).collect();

                // N elements requires N-1 "," (N-1 * 2 chars) + 1 "()" (2 chars)
                // e.g. (1, 2, 3)
                // N-1 * 2 + 2 == N * 2, or +2 chars per element
                let minimum_length = {
                    let ident_length =
                        maybe_ident.as_ref().map_or(0, |ident| ident.len());
                    let elements_length = elements
                        .iter()
                        .map(|elem| elem.minimum_length + 2)
                        .sum::<usize>();
                    ident_length + elements_length
                };

                TextFragment {
                    minimum_length,
                    ron_value: RonValue::TupleType {
                        maybe_ident,
                        elements,
                    },
                }
            }

            Rule::fields_type => {
                let mut tokens_iter = pair.into_inner();
                let maybe_ident = match tokens_iter.peek().map(|p| p.as_rule())
                {
                    Some(Rule::ident) => {
                        let ident =
                            tokens_iter.next().unwrap().as_str().to_string();
                        Some(ident)
                    }
                    _ => None,
                };

                let fields: Vec<_> = tokens_iter
                    .map(|field| {
                        let mut kv_iter = field.into_inner();
                        let key = kv_iter.next().unwrap().as_str().to_string();
                        let val = TextFragment::from(kv_iter.next().unwrap());
                        (key, val)
                    })
                    .collect();

                // N fields requires N ": " (N * 2 chars) + N-1 ", " (N-1 * 2 chars) + 1 "()" (2 chars)
                // e.g. (a: 1, b: 2, c: 3)
                // N * 2 + N-1 * 2 + 2 == N * 4, or +4 chars per field
                let minimum_length = {
                    let ident_length =
                        maybe_ident.as_ref().map_or(0, |ident| ident.len());
                    let fields_length = fields
                        .iter()
                        .map(|(key, val)| key.len() + val.minimum_length + 4)
                        .sum::<usize>();
                    ident_length + fields_length
                };

                TextFragment {
                    minimum_length,
                    ron_value: RonValue::FieldsType {
                        maybe_ident,
                        fields,
                    },
                }
            }

            // necessary wrappers due to the grammar structure
            Rule::COMMENT | Rule::value => {
                TextFragment::from(pair.into_inner().next().unwrap())
            }

            // handled in other rules
            _ => unreachable!("{:?}", pair.as_rule()),
        }
    }
}
