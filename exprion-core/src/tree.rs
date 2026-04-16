use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwnedTree {
    label: String,
    children: Vec<OwnedTree>,
}

impl OwnedTree {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    pub fn with_child(mut self, child: OwnedTree) -> Self {
        self.children.push(child);
        self
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn children(&self) -> &[OwnedTree] {
        &self.children
    }
}

impl fmt::Display for OwnedTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}

impl OwnedTree {
    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        for _ in 0..depth {
            write!(f, "  ")?;
        }
        writeln!(f, "{}", self.label)?;

        for child in &self.children {
            child.fmt_with_indent(f, depth + 1)?;
        }

        Ok(())
    }
}
