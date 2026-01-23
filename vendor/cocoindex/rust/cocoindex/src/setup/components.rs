use super::{ResourceSetupChange, SetupChangeType};
use crate::prelude::*;

impl<A: ResourceSetupChange, B: ResourceSetupChange> ResourceSetupChange for (A, B) {
    fn describe_changes(&self) -> Vec<setup::ChangeDescription> {
        let mut result = vec![];
        result.extend(self.0.describe_changes());
        result.extend(self.1.describe_changes());
        result
    }

    fn change_type(&self) -> SetupChangeType {
        match (self.0.change_type(), self.1.change_type()) {
            (SetupChangeType::Invalid, _) | (_, SetupChangeType::Invalid) => {
                SetupChangeType::Invalid
            }
            (SetupChangeType::NoChange, b) => b,
            (a, _) => a,
        }
    }
}
