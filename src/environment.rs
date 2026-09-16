use crate::error::KarmaError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Default)]
pub struct Environment {
    values: HashMap<String, Value>,
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

    pub fn define(&mut self, name: String, value: Value) -> Result<(), KarmaError> {
        if self.values.contains_key(&name) {
            return Err(KarmaError::runtime(format!(
                "variable '{name}' is already defined in this scope"
            )));
        }
        self.values.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value, KarmaError> {
        if let Some(value) = self.values.get(name) {
            return Ok(value.clone());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), KarmaError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }
}
