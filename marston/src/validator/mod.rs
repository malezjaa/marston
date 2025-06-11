use crate::info::Info;

pub mod rules;

pub type ValidationRule<T> = fn(&T, &mut Info);

pub trait Validate: Sized {
    fn rules() -> Vec<ValidationRule<Self>>;

    fn call_rules(&self, info: &mut Info) {
        for rule in Self::rules() {
            rule(self, info);
        }
    }

    fn validate(&self, info: &mut Info);
}
