use symbol_map::indexing::HashIndexing;

type SymbolTable = HashIndexing<String, usize>;

pub struct Symbol {
    id: usize
}
