use lasso::{Rodeo, Spur, ThreadedRodeo};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct IdentTable {
    interner: ThreadedRodeo,
}

impl IdentTable {
    pub fn new() -> Self {
        Self { interner: ThreadedRodeo::default() }
    }

    pub fn intern(&mut self, name: &str) -> Spur {
        self.interner.get_or_intern(name)
    }

    pub fn resolve(&self, sym: Spur) -> &str {
        self.interner.resolve(&sym)
    }
}

pub static GLOBAL_TABLE: Lazy<Mutex<IdentTable>> = Lazy::new(|| Mutex::new(IdentTable::new()));

#[inline]
pub fn get_or_intern(name: &str) -> Spur {
    GLOBAL_TABLE.lock().unwrap().intern(name)
}

#[inline]
pub fn resolve(sym: Spur) -> String {
    GLOBAL_TABLE.lock().unwrap().resolve(sym).to_string()
}
