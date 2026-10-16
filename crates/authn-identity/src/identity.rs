use crate::id::{MachineId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Identity {
    Machine(Machine),
    User(User),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Machine {
    pub id: MachineId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
}

impl From<Machine> for Identity {
    fn from(value: Machine) -> Self {
        Self::Machine(value)
    }
}
impl From<User> for Identity {
    fn from(value: User) -> Self {
        Self::User(value)
    }
}
