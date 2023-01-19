use crate::parser::{Element, ElementType};

pub trait GrannyResolve {
    fn resolve(&self, path: &str) -> Option<&Element>;
}

impl GrannyResolve for Vec<Element> {
    fn resolve(&self, path: &str) -> Option<&Element> {
        let index = path.chars().position(|c| c == '.');

        let name = if let Some(index) = index {
            &path[0..index]
        } else {
            path
        };

        for e in self {
            if e.name == name {
                return if index.is_none() {
                    Some(e)
                } else {
                    let index = index.unwrap();

                    match &e.element {
                        ElementType::Reference(elements) => {
                            elements.resolve(&path[index + 1..])
                        }
                        _ => None
                    }
                }
            }
        }

        None
    }
}