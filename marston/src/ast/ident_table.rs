use lasso::{Rodeo, Spur};

#[derive(Debug)]
pub struct IdentTable {
    interner: Rodeo,
}

impl IdentTable {
    pub fn new() -> Self {
        Self { interner: Rodeo::default() }
    }

    pub fn intern(&mut self, name: &str) -> Spur {
        self.interner.get_or_intern(name)
    }

    pub fn resolve(&self, sym: Spur) -> &str {
        self.interner.resolve(&sym)
    }
}
