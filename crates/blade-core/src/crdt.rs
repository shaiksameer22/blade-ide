use yrs::{Doc, Text, TextRef, Transact, StateVector, Update, ReadTxn};
use yrs::updates::decoder::Decode;
use anyhow::Result;

pub struct SharedDocument {
    pub doc: Doc,
    pub text_ref: TextRef,
}

impl SharedDocument {
    pub fn new() -> Self {
        let doc = Doc::new();
        let text_ref = doc.get_or_insert_text("content");
        Self { doc, text_ref }
    }
}

impl Default for SharedDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedDocument {

    pub fn sync_from_local(&mut self, local_text: &str) {
        let mut txn = self.doc.transact_mut();
        let len = self.text_ref.len(&txn);
        if len > 0 {
            self.text_ref.remove_range(&mut txn, 0, len);
        }
        self.text_ref.insert(&mut txn, 0, local_text);
    }

    pub fn generate_update(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    pub fn apply_update(&mut self, update: &[u8]) -> Result<()> {
        let mut txn = self.doc.transact_mut();
        let update = Update::decode_v1(update)?;
        txn.apply_update(update);
        Ok(())
    }
}

