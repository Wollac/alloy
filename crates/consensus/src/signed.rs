use crate::transaction::SignableTransaction;
use alloy_primitives::{Signature, B256};

/// A transaction with a signature and hash seal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signed<T, Sig = Signature> {
    tx: T,
    signature: Sig,
    hash: B256,
}

impl<T, Sig> Signed<T, Sig> {
    /// Returns a reference to the transaction.
    pub const fn tx(&self) -> &T {
        &self.tx
    }

    /// Returns a reference to the signature.
    pub const fn signature(&self) -> &Sig {
        &self.signature
    }

    /// Returns a reference to the transaction hash.
    pub const fn hash(&self) -> &B256 {
        &self.hash
    }

    /// Splits the transaction into parts.
    pub fn into_parts(self) -> (T, Sig, B256) {
        (self.tx, self.signature, self.hash)
    }
}

impl<T: SignableTransaction<Sig>, Sig> Signed<T, Sig> {
    /// Instantiate from a transaction and signature. Does not verify the signature.
    pub const fn new_unchecked(tx: T, signature: Sig, hash: B256) -> Self {
        Self { tx, signature, hash }
    }

    /// Calculate the signing hash for the transaction.
    pub fn signature_hash(&self) -> B256 {
        self.tx.signature_hash()
    }
}

#[cfg(feature = "k256")]
impl<T: SignableTransaction<Signature>> Signed<T, Signature> {
    /// Recover the signer of the transaction
    pub fn recover_signer(
        &self,
    ) -> Result<alloy_primitives::Address, alloy_primitives::SignatureError> {
        let sighash = self.tx.signature_hash();
        self.signature.recover_address_from_prehash(&sighash)
    }
}
