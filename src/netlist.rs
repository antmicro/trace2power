use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Module {
    pub cells: HashMap<String, Cell>
}

#[derive(Deserialize)]
pub struct Cell {
    #[serde(rename = "type")]
    pub type_name: String
}

#[derive(Deserialize)]
pub struct Netlist {
    pub modules: HashMap<String, Module>
}

impl Cell {
    pub fn get_module<'n>(&self, netlist: &'n Netlist) -> Option<&'n Module> {
        netlist.modules.get(&self.type_name)
    }
}

#[derive(Copy, Clone)]
pub enum ModuleLookupError {
    CellNotFound,
    ModuleUndefined,
}

impl Module {
    pub fn get_module_of_cell<'s>(
        &self,
        netlist: &'s Netlist,
        cell_name: &str
    ) -> Result<&'s Self, ModuleLookupError> {
        let cell = self.cells.get(cell_name).ok_or(ModuleLookupError::CellNotFound)?;
        let module = cell.get_module(netlist).ok_or(ModuleLookupError::ModuleUndefined)?;
        Ok(module)
    }
}
