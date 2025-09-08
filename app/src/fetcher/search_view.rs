use anyhow::Result;
use db::ConnCache;
use db::models::{FullTx, NewSearch};

pub struct SearchView(Vec<FullTx>);

pub(crate) fn get_search_txs(
    search: NewSearch,
    db_conn: &mut impl ConnCache,
) -> Result<SearchView> {
    let result = search.search_txs(db_conn)?;

    let search_view = SearchView(result);

    Ok(search_view)
}

impl SearchView {
    pub fn tx_array(&self) -> Vec<Vec<String>> {
        self.0.iter().map(|tx| tx.to_array()).collect()
    }

    pub fn get_tx(&self, index: usize) -> &FullTx {
        &self.0[index]
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn new_empty() -> Self {
        SearchView(Vec::new())
    }
}
