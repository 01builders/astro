use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error, Read, ReadExt as _, Write};
use commonware_consensus::Block as Bl;
use commonware_consensus::{
    threshold_simplex::types::{Finalization, Notarization},
    Viewable,
};
use commonware_cryptography::{
    bls12381::primitives::variant::MinPk, sha256::Digest, Committable, Digestible, Hasher, Sha256,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub block: u8,
    pub app: u8,
}

impl Default for Version {
    fn default() -> Self {
        Version { block: 1, app: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub version: Version,
    pub chain_id: String,
    pub parent: Digest,
    pub height: u64,
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<u8>,

    /// Pre-computed digest of the block.
    digest: Digest,
}

impl Block {
    pub fn block_hash(&self) -> [u8; 32] {
            self.digest.as_ref().try_into().unwrap()
    }


    pub fn compute_digest(
        parent: Digest,
        height: u64,
        timestamp: u64,
        transactions: Vec<u8>,
    ) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(parent);
        hasher.update(&height.to_be_bytes());
        hasher.update(&timestamp.to_be_bytes());
        hasher.finalize();
        Self {
            header: Header {
                parent,
                height,
                timestamp,
            }
                transactions: transactions,
                digest: hasher,
        }
    }

    pub fn genesis(genesis_hash: [u8; 32]) -> Self {
        Self {
            header: Header {
                chain_id: String::new(), // TODO: implement
                version: Version::default(),
                parent: genesis_hash.into(),
                height: 0,
                timestamp: 0,
            },
            transactions: vec![],
            digest: genesis_hash.into(),
        }
    }
}

impl Bl for Block {
    fn height(&self) -> u64 {
        self.header.height
    }

    fn parent(&self) -> Self::Commitment {
        self.header.parent
    }
}

impl Viewable for Block {
    type View = u64;

    fn view(&self) -> commonware_consensus::simplex::types::View {
        self.header.height
    }
}

impl EncodeSize for Block {
    fn encode_size(&self) -> usize {
        panic!("Not implemented")
    }
}

impl Write for Block {
    fn write(&self, buf: &mut impl BufMut) {
        panic!("Not implemented")
    }
}

impl Read for Block {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _cfg: &Self::Cfg) -> Result<Self, commonware_codec::Error> {
        let len = buf.get_u32();
    }
}

impl Digestible for Block {
    type Digest = Digest;

    fn digest(&self) -> Digest {
        self.digest
    }
}

impl Committable for Block {
    type Commitment = Digest;

    fn commitment(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notarized {
    pub proof: Notarization<MinPk, Digest>,
    pub block: Block,
}

impl Notarized {
    pub fn new(proof: Notarization<MinPk, Digest>, block: Block) -> Self {
        Self { proof, block }
    }
}

impl Write for Notarized {
    fn write(&self, buf: &mut impl BufMut) {
        self.proof.write(buf);
        self.block.write(buf);
    }
}

impl Read for Notarized {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &Self::Cfg) -> Result<Self, Error> {
        let proof = Notarization::<MinPk, Digest>::read_cfg(buf, &())?; // todo: get a test on this to make sure buf.remaining is safe
        let block = Block::read(buf)?;

        // Ensure the proof is for the block
        if proof.proposal.payload != block.digest() {
            return Err(Error::Invalid(
                "types::Notarized",
                "Proof payload does not match block digest",
            ));
        }
        Ok(Self { proof, block })
    }
}

impl EncodeSize for Notarized {
    fn encode_size(&self) -> usize {
        self.proof.encode_size() + self.block.encode_size()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finalized {
    pub proof: Finalization<MinPk, Digest>,
    pub block: Block,
}

impl Finalized {
    pub fn new(proof: Finalization<MinPk, Digest>, block: Block) -> Self {
        Self { proof, block }
    }
}

impl Write for Finalized {
    fn write(&self, buf: &mut impl BufMut) {
        self.proof.write(buf);
        self.block.write(buf);
    }
}

impl Read for Finalized {
    type Cfg = ();

    fn read_cfg(buf: &mut impl Buf, _: &Self::Cfg) -> Result<Self, Error> {
        let proof = Finalization::<MinPk, Digest>::read_cfg(buf, &())?;
        let block = Block::read(buf)?;

        // Ensure the proof is for the block
        if proof.proposal.payload != block.digest() {
            return Err(Error::Invalid(
                "types::Finalized",
                "Proof payload does not match block digest",
            ));
        }
        Ok(Self { proof, block })
    }
}

impl EncodeSize for Finalized {
    fn encode_size(&self) -> usize {
        self.proof.encode_size() + self.block.encode_size()
    }
}
