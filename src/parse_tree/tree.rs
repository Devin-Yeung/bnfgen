use std::fmt;
use std::fmt::Debug;

pub enum ParseTree<T> {
    Leaf(T),
    Branch {
        name: String,
        children: Vec<ParseTree<T>>,
    },
}

impl<T> ParseTree<T> {
    pub(crate) fn branch(name: String) -> ParseTree<T> {
        ParseTree::Branch {
            name,
            children: Vec::new(),
        }
    }

    pub(crate) fn children_len(&self) -> usize {
        match self {
            ParseTree::Leaf(_) => 1, // TODO: 0 or 1 ?
            ParseTree::Branch { children, .. } => children.len(),
        }
    }
}

// derive debug if T: Debug
impl<T: Debug> Debug for ParseTree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseTree::Leaf(value) => f.debug_tuple("Leaf").field(value).finish(),
            ParseTree::Branch { name, children } => f
                .debug_struct("Branch")
                .field("name", name)
                .field("children", children)
                .finish(),
        }
    }
}
