use orchard::{note::ExtractedNoteCommitment, note_encryption::OrchardDomain};
use zcash_note_encryption::{EphemeralKeyBytes, ShieldedOutput, COMPACT_NOTE_SIZE};
use zcash_primitives::{
    block::{BlockHash, BlockHeader},
    consensus::{BlockHeight, Parameters},
    sapling::note_encryption::SaplingDomain,
};

tonic::include_proto!("cash.z.wallet.sdk.rpc");

impl CompactBlock {
    /// Returns the [`BlockHash`] for this block.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.hash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.hash`]: #structfield.hash
    pub fn hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.hash()
        } else {
            BlockHash::from_slice(&self.hash)
        }
    }

    /// Returns the [`BlockHash`] for this block's parent.
    ///
    /// # Panics
    ///
    /// This function will panic if [`CompactBlock.header`] is not set and
    /// [`CompactBlock.prevHash`] is not exactly 32 bytes.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    /// [`CompactBlock.prevHash`]: #structfield.prevHash
    pub fn prev_hash(&self) -> BlockHash {
        if let Some(header) = self.header() {
            header.prev_block
        } else {
            BlockHash::from_slice(&self.prev_hash)
        }
    }

    /// Returns the [`BlockHeader`] for this block if present.
    ///
    /// A convenience method that parses [`CompactBlock.header`] if present.
    ///
    /// [`CompactBlock.header`]: #structfield.header
    pub fn header(&self) -> Option<BlockHeader> {
        if self.header.is_empty() {
            None
        } else {
            BlockHeader::read(&self.header[..]).ok()
        }
    }

    /// Returns the [`BlockHeight`] for this block.
    ///
    /// A convenience method that wraps [`CompactBlock.height`]
    ///
    /// [`CompactBlock.height`]: #structfield.height
    pub fn height(&self) -> BlockHeight {
        BlockHeight::from(self.height as u32)
    }
}

impl<P: Parameters> ShieldedOutput<SaplingDomain<P>, COMPACT_NOTE_SIZE> for CompactSaplingOutput {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(*slice_to_array(&self.epk))
    }
    fn cmstar_bytes(&self) -> [u8; 32] {
        *slice_to_array(&self.cmu)
    }
    fn enc_ciphertext(&self) -> &[u8; COMPACT_NOTE_SIZE] {
        slice_to_array(&self.ciphertext)
    }
}
impl ShieldedOutput<OrchardDomain, COMPACT_NOTE_SIZE> for CompactOrchardAction {
    fn ephemeral_key(&self) -> EphemeralKeyBytes {
        EphemeralKeyBytes(*slice_to_array(&self.ephemeral_key))
    }
    fn cmstar_bytes(&self) -> [u8; 32] {
        *slice_to_array(&self.cmx)
    }
    fn enc_ciphertext(&self) -> &[u8; COMPACT_NOTE_SIZE] {
        slice_to_array(&self.ciphertext)
    }
}

impl TryFrom<&CompactOrchardAction> for orchard::note_encryption::CompactAction {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &CompactOrchardAction) -> Result<Self, Self::Error> {
        Ok(Self::from_parts(
            Option::from(orchard::note::Nullifier::from_bytes(&<[u8; 32]>::try_from(
                value.nullifier.as_slice(),
            )?))
            .ok_or("bad nullifier")?,
            Option::from(ExtractedNoteCommitment::from_bytes(&<[u8; 32]>::try_from(
                value.cmx.as_slice(),
            )?))
            .ok_or("bad enc")?,
            <[u8; 32]>::try_from(value.ephemeral_key.as_slice())?.into(),
            <[u8; COMPACT_NOTE_SIZE]>::try_from(value.ciphertext.as_slice())?,
        ))
    }
}

pub(crate) fn slice_to_array<const N: usize>(slice: &[u8]) -> &[u8; N] {
    <&[u8; N]>::try_from(slice).unwrap_or(&[0; N])
    //todo: This default feels dangerous. Find better solution
}
