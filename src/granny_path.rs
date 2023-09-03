use crate::parser::{Element, ElementType};

pub trait GrannyResolve {
    fn resolve(&self, path: &str) -> Option<&ElementType>;
}

impl GrannyResolve for Vec<Element> {
    fn resolve(&self, path: &str) -> Option<&ElementType> {
        let index = path.chars().position(|c| c == '.');

        let name = if let Some(index) = index {
            &path[0..index]
        } else {
            path
        };

        for e in self {
            if e.name == name {
                return if let Some(index) = index {
                    match &e.element {
                        ElementType::Reference(elements) => {
                            elements.resolve(&path[index + 1..])
                        }
                        _ => None
                    }
                } else {
                    Some(&e.element)
                }
            }
        }

        None
    }
}

pub enum GrannyPathError {
    UnresolvedPath,
    UnknownVariant(ElementType)
}

#[macro_export]
macro_rules! granny_path {
    ($elements:expr, $name:expr, $variant:path) => {
        if let Some(e) = $elements.resolve($name) {
            match e {
                $variant(val) => Ok(val),
                _ => Err(GrannyPathError::UnknownVariant(e))
            }
        } else {
            Err(GrannyPathError::UnresolvedPath)
        }
    }
}