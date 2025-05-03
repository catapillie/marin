use crate::com::{ir, Checker};
use std::collections::BTreeSet;

impl Checker<'_, '_> {
    pub fn create_function_info(&mut self) -> ir::FunInfo {
        self.restore_function_info(ir::FunInfo {
            depth: self.scope.depth(),
            captured: BTreeSet::new(),
        })
    }

    pub fn restore_function_info(&mut self, info: ir::FunInfo) -> ir::FunInfo {
        std::mem::replace(&mut self.current_function, info)
    }

    pub fn get_function_info(&self) -> &ir::FunInfo {
        &self.current_function
    }

    pub fn get_function_info_mut(&mut self) -> &mut ir::FunInfo {
        &mut self.current_function
    }
}
