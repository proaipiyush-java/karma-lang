use crate::error::KarmaError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, Binding>,
    parent: Option<EnvRef>,
}

impl Environment {
    pub fn root() -> EnvRef {
        Rc::new(RefCell::new(Self::default()))
    }

    pub fn child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            parent: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Value, mutable: bool) -> Result<(), KarmaError> {
        if self.values.contains_key(&name) {
            return Err(KarmaError::runtime(format!(
                "variable '{name}' is already defined in this scope"
            )));
        }
        self.values.insert(name, Binding { value, mutable });
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value, KarmaError> {
        if let Some(binding) = self.values.get(name) {
            return Ok(binding.value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), KarmaError> {
        if let Some(binding) = self.values.get_mut(name) {
            if !binding.mutable {
                return Err(KarmaError::runtime(format!(
                    "cannot assign to immutable binding '{name}'"
                )));
            }
            binding.value = value;
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }
}
