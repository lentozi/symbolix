pub mod compile;
pub mod runtime;
mod symbol_table;

pub use compile::CompileContext;
pub use runtime::RuntimeContext;
pub use symbol_table::NameId;

#[doc(hidden)]
pub mod testing {
    pub use super::symbol_table::{NameId, SymbolTable, VariableStack};
}
