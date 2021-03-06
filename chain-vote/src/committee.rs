use crate::gang::{GroupElement, Scalar};
use crate::gargamel::{PublicKey, SecretKey};
use crate::hybrid;
use crate::hybrid::SymmetricKey;
use crate::math::Polynomial;
use rand_core::{CryptoRng, RngCore};

/// Committee member election secret key
#[derive(Clone)]
pub struct MemberSecretKey(pub(crate) SecretKey);

/// Committee member election public key
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemberPublicKey(pub(crate) PublicKey);

#[derive(Clone)]
pub struct MemberCommunicationKey(SecretKey);

/// Committee Member communication public key (with other committee members)
#[derive(Clone)]
pub struct MemberCommunicationPublicKey(PublicKey);

/// The overall committee public key used for everyone to encrypt their vote to.
#[derive(Clone)]
pub struct ElectionPublicKey(pub(crate) PublicKey);

impl ElectionPublicKey {
    #[doc(hidden)]
    pub fn as_raw(&self) -> &PublicKey {
        &self.0
    }
}

/// Initial state generated by a Member, which include keys for this election
#[derive(Clone)]
pub struct MemberState {
    sk: MemberSecretKey,
    owner_index: usize,
    apubs: Vec<GroupElement>,
    es: Vec<GroupElement>,
    encrypted: Vec<(hybrid::HybridCiphertext, hybrid::HybridCiphertext)>,
}

pub type CRS = GroupElement;

impl MemberState {
    /// Generate a new member state from random, where the number
    pub fn new<R: RngCore + CryptoRng>(
        rng: &mut R,
        t: usize,
        h: &CRS, // TODO: document
        committee_pks: &[MemberCommunicationPublicKey],
        my: usize,
    ) -> MemberState {
        let n = committee_pks.len();
        assert!(t > 0);
        assert!(t <= n);
        assert!(my < n);

        let pcomm = Polynomial::random(rng, t);
        let pshek = Polynomial::random(rng, t);

        let mut apubs = Vec::new();
        let mut es = Vec::new();

        for (ai, bi) in pshek.get_coefficients().zip(pcomm.get_coefficients()) {
            let apub = GroupElement::generator() * ai;
            let e = &apub + h * bi;
            apubs.push(apub);
            es.push(e);
        }

        let mut encrypted = Vec::new();
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            // don't generate share for self
            if i == my {
                continue;
            } else {
                let idx = Scalar::from_u64((i + 1) as u64);
                let share_comm = pcomm.evaluate(&idx);
                let share_shek = pshek.evaluate(&idx);

                let pk = &committee_pks[i];
                let sym_key_shares = SymmetricKey::new(rng);
                let sym_key_blinders = SymmetricKey::new(rng);

                let rcomm = Scalar::random(rng);
                let rshek = Scalar::random(rng);
                let ecomm =
                    hybrid::hybrid_encrypt(&pk.0, &sym_key_shares, &share_comm.to_bytes(), &rcomm);
                let eshek = hybrid::hybrid_encrypt(
                    &pk.0,
                    &sym_key_blinders,
                    &share_shek.to_bytes(),
                    &rshek,
                );

                encrypted.push((ecomm, eshek));
            }
        }

        assert_eq!(apubs.len(), t + 1);
        assert_eq!(es.len(), t + 1);
        assert_eq!(encrypted.len(), n - 1);

        MemberState {
            sk: MemberSecretKey(SecretKey {
                sk: pshek.at_zero(),
            }),
            owner_index: my + 1, // committee member are 1-indexed
            apubs,
            es,
            encrypted,
        }
    }

    pub fn secret_key(&self) -> &MemberSecretKey {
        &self.sk
    }

    pub fn public_key(&self) -> MemberPublicKey {
        MemberPublicKey(PublicKey {
            pk: self.apubs[0].clone(),
        })
    }
}

impl MemberSecretKey {
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.sk.to_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let sk = Scalar::from_bytes(bytes)?;
        Some(Self(SecretKey { sk }))
    }
}

impl MemberPublicKey {
    pub const BYTES_LEN: usize = PublicKey::BYTES_LEN;

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        Some(Self(PublicKey::from_bytes(buf)?))
    }
}

impl From<PublicKey> for MemberPublicKey {
    fn from(pk: PublicKey) -> MemberPublicKey {
        MemberPublicKey(pk)
    }
}

impl MemberCommunicationKey {
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let sk = SecretKey::generate(rng);
        MemberCommunicationKey(sk)
    }

    pub fn to_public(&self) -> MemberCommunicationPublicKey {
        MemberCommunicationPublicKey(PublicKey {
            pk: &GroupElement::generator() * &self.0.sk,
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<MemberCommunicationKey> {
        SecretKey::from_bytes(bytes).map(MemberCommunicationKey)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.sk.to_bytes()
    }
}

impl MemberCommunicationPublicKey {
    pub fn from_public_key(pk: PublicKey) -> Self {
        Self(pk)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl ElectionPublicKey {
    /// Create an election public key from all the participants of this committee
    pub fn from_participants(pks: &[MemberPublicKey]) -> Self {
        let mut k = pks[0].0.pk.clone();
        for pk in &pks[1..] {
            k = k + &pk.0.pk;
        }
        ElectionPublicKey(PublicKey { pk: k })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        PublicKey::from_bytes(buf).map(ElectionPublicKey)
    }
}
