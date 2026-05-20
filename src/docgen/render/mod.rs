use crate::docgen::model::DocSymbol;

pub mod html;
pub mod json;
pub mod markdown;

pub(crate) fn symbol_source_location(symbol: &DocSymbol) -> String {
    format!("{}:{}", symbol.source_path.display(), symbol.line)
}
