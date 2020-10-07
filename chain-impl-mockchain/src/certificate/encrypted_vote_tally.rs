use crate::transaction::{SingleAccountBindingSignature, TransactionBindingAuthData};
use crate::vote::CommitteeId;
use crate::{
    certificate::{CertificateSlice, VotePlanId, VoteTallyPayload},
    transaction::{Payload, PayloadAuthData, PayloadData, PayloadSlice},
    vote::{PayloadType, TryFromIntError},
};
use chain_core::{
    mempack::{ReadBuf, ReadError, Readable},
    property,
};
use chain_crypto::Verification;
use typed_bytes::{ByteArray, ByteBuilder};

#[derive(Debug, Clone)]
pub struct EncryptedVoteTallyProof {
    pub id: CommitteeId,
    pub signature: SingleAccountBindingSignature,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct EncryptedVoteTally {
    id: VotePlanId,
    payload: VoteTallyPayload,
}

impl EncryptedVoteTallyProof {
    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.u8(0)
            .bytes(self.id.as_ref())
            .bytes(self.signature.as_ref())
    }

    pub fn verify<'a>(
        &self,
        tally_type: PayloadType,
        verify_data: &TransactionBindingAuthData<'a>,
    ) -> Verification {
        if tally_type != PayloadType::Private {
            Verification::Failed
        } else {
            let pk = self.id.public_key();
            self.signature.verify_slice(&pk, verify_data)
        }
    }
}

impl EncryptedVoteTally {
    pub fn new_private(id: VotePlanId) -> Self {
        Self {
            id,
            payload: VoteTallyPayload::Public,
        }
    }

    pub fn id(&self) -> &VotePlanId {
        &self.id
    }

    pub fn tally_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn serialize_in(&self, bb: ByteBuilder<Self>) -> ByteBuilder<Self> {
        bb.bytes(self.id().as_ref())
            .u8(self.payload.payload_type() as u8)
    }

    pub fn serialize(&self) -> ByteArray<Self> {
        self.serialize_in(ByteBuilder::new()).finalize()
    }
}

/* Auth/Payload ************************************************************* */

impl Payload for EncryptedVoteTally {
    const HAS_DATA: bool = true;
    const HAS_AUTH: bool = true; // TODO: true it is the Committee signatures
    type Auth = EncryptedVoteTallyProof;

    fn payload_data(&self) -> PayloadData<Self> {
        PayloadData(
            self.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn payload_auth_data(auth: &Self::Auth) -> PayloadAuthData<Self> {
        PayloadAuthData(
            auth.serialize_in(ByteBuilder::new())
                .finalize_as_vec()
                .into(),
            std::marker::PhantomData,
        )
    }

    fn to_certificate_slice(p: PayloadSlice<'_, Self>) -> Option<CertificateSlice<'_>> {
        Some(CertificateSlice::from(p))
    }
}

/* Ser/De ******************************************************************* */

impl property::Serialize for EncryptedVoteTally {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), Self::Error> {
        writer.write_all(self.serialize().as_slice())?;
        Ok(())
    }
}

impl Readable for EncryptedVoteTallyProof {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        match buf.peek_u8()? {
            0 => {
                let _ = buf.get_u8()?;
                let id = CommitteeId::read(buf)?;
                let signature = SingleAccountBindingSignature::read(buf)?;
                Ok(Self { id, signature })
            }
            _ => Err(ReadError::StructureInvalid(
                "Unknown Tally proof type".to_owned(),
            )),
        }
    }
}

impl Readable for EncryptedVoteTally {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        use std::convert::TryInto as _;

        let id = <[u8; 32]>::read(buf)?.into();
        let payload_type = buf
            .get_u8()?
            .try_into()
            .map_err(|e: TryFromIntError| ReadError::StructureInvalid(e.to_string()))?;

        let payload = match payload_type {
            PayloadType::Public => {
                unreachable!("The payload for an encrypted tally should not be public")
            }
            PayloadType::Private => VoteTallyPayload::Private,
        };

        Ok(Self { id, payload })
    }
}
