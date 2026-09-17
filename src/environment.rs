use crate::error::KarmaError;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug, Clone)]
struct Binding {
    value: Option<Value>,
    mutable: bool,
    movable: bool,
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

    pub fn define(
        &mut self,
        name: String,
        value: Value,
        mutable: bool,
        movable: bool,
    ) -> Result<(), KarmaError> {
        if self.values.contains_key(&name) {
            return Err(KarmaError::runtime(format!(
                "variable '{name}' is already defined in this scope"
            )));
        }
        self.values.insert(
            name,
            Binding {
                value: Some(value),
                mutable,
                movable,
            },
        );
        Ok(())
    }

    /// Read without transferring ownership. String clones are cheap in the
    /// bootstrap interpreter because String uses Rc<str> backing storage.
    pub fn get(&self, name: &str) -> Result<Value, KarmaError> {
        if let Some(binding) = self.values.get(name) {
            return binding
                .value
                .clone()
                .ok_or_else(|| KarmaError::runtime(format!("use of moved value '{name}'")));
        }
        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }

    /// Transfer ownership out of a binding. The slot remains present but is
    /// marked empty so the runtime can independently detect use-after-move.
    pub fn take(&mut self, name: &str) -> Result<Value, KarmaError> {
        if let Some(binding) = self.values.get_mut(name) {
            if !binding.movable {
                return Err(KarmaError::runtime(format!(
                    "cannot move out of borrowed binding '{name}'"
                )));
            }
            return binding
                .value
                .take()
                .ok_or_else(|| KarmaError::runtime(format!("use of moved value '{name}'")));
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().take(name);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }

    /// Assignment reinitializes a mutable binding even if its previous owned
    /// value was moved out.
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), KarmaError> {
        if let Some(binding) = self.values.get_mut(name) {
            if !binding.mutable {
                return Err(KarmaError::runtime(format!(
                    "cannot assign to immutable binding '{name}'"
                )));
            }
            if !binding.movable {
                return Err(KarmaError::runtime(format!(
                    "cannot assign to borrowed binding '{name}'"
                )));
            }
            binding.value = Some(value);
            return Ok(());
        }
        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign(name, value);
        }
        Err(KarmaError::runtime(format!("undefined variable '{name}'")))
    }
}
