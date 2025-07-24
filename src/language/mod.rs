use symbol::SymbolId;

pub mod expression;
pub mod parsing;
pub mod symbol;

#[derive(Default)]
pub struct Language {
    // Runtime constraints specifying number of inputs/outputs?
    symbols: Vec<String>,
    // constraints: Vec<Vec<Constraint>>
}

impl Language {
    pub fn add_symbol(mut self, name: &str) -> Self {
        self.symbols.push(String::from(name));
        self
    }

    pub fn get_symbol(&self, id: SymbolId) -> &str {
        &self.symbols[id]
    }

    pub fn get_id(&self, name: &str) -> SymbolId {
        self.try_get_id(name)
            .unwrap_or_else(|| panic!("Symbol not present in the language: {name}"))
    }

    pub fn try_get_id(&self, name: &str) -> Option<SymbolId> {
        self.symbols.iter().position(|x| name == x)
    }

    pub fn math() -> Self {
        Self::default()
            .add_symbol("+")
            .add_symbol("-")
            .add_symbol("*")
            .add_symbol("/")
            .add_symbol("sin")
            .add_symbol("cos")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn symbols() {
        let lang = super::Language::default().add_symbol("f").add_symbol("g");

        assert_eq!("f", lang.get_symbol(0));
        assert_eq!("g", lang.get_symbol(1));
        assert!(lang.try_get_id("h").is_none());
    }
}
