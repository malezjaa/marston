pub mod rules;

pub type ValidationRule<T> = fn(&T);

pub trait Validate: Sized {
    fn rules() -> Vec<ValidationRule<Self>>;

    fn call_rules(&self) {
        for rule in Self::rules() {
            rule(self);
        }
    }

    fn validate(&self);
}
