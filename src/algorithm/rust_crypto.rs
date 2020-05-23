use base64;
use crypto_mac::Mac;
use digest::generic_array::ArrayLength;
use digest::{BlockInput, FixedOutput, Input, Reset};
use hmac::Hmac;
use sha2;

use crate::algorithm::{AlgorithmType, SigningAlgorithm, VerifyingAlgorithm};
use crate::error::Error;
use crate::SEPARATOR;
/// A trait used to make the implementation of `SigningAlgorithm` and
/// `VerifyingAlgorithm` easier.
/// RustCrypto crates tend to have algorithm types defined at the type level,
/// so they cannot accept a self argument.
pub trait TypeLevelAlgorithmType {
    fn algorithm_type() -> AlgorithmType;
}

macro_rules! type_level_algorithm_type {
    ($rust_crypto_type: ty, $algorithm_type: expr) => {
        impl TypeLevelAlgorithmType for $rust_crypto_type {
            fn algorithm_type() -> AlgorithmType {
                $algorithm_type
            }
        }
    };
}

type_level_algorithm_type!(sha2::Sha256, AlgorithmType::Hs256);
type_level_algorithm_type!(sha2::Sha384, AlgorithmType::Hs384);
type_level_algorithm_type!(sha2::Sha512, AlgorithmType::Hs512);

impl<D> SigningAlgorithm for Hmac<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone + TypeLevelAlgorithmType,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
{
    fn algorithm_type(&self) -> AlgorithmType {
        D::algorithm_type()
    }

    fn sign(&self, header: &str, claims: &str) -> Result<String, Error> {
        let hmac = get_hmac_with_data(&self, header, claims);
        let mac_result = hmac.result();
        let code = mac_result.code();
        Ok(base64::encode_config(&code, base64::URL_SAFE_NO_PAD))
    }
}

impl<D> VerifyingAlgorithm for Hmac<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone + TypeLevelAlgorithmType,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
{
    fn algorithm_type(&self) -> AlgorithmType {
        D::algorithm_type()
    }

    fn verify_bytes(&self, header: &str, claims: &str, signature: &[u8]) -> Result<bool, Error> {
        let hmac = get_hmac_with_data(self, header, claims);
        hmac.verify(&signature)?;
        Ok(true)
    }
}

fn get_hmac_with_data<D>(hmac: &Hmac<D>, header: &str, claims: &str) -> Hmac<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone + TypeLevelAlgorithmType,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
{
    let mut hmac = hmac.clone();
    hmac.reset();
    hmac.input(header.as_bytes());
    hmac.input(SEPARATOR.as_bytes());
    hmac.input(claims.as_bytes());
    hmac
}

#[cfg(test)]
mod tests {
    use crate::algorithm::{SigningAlgorithm, VerifyingAlgorithm};
    use crypto_mac::Mac;
    use hmac::Hmac;
    use sha2::Sha256;

    #[test]
    pub fn sign() {
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let claims = "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWV9";
        let expected_signature = "TJVA95OrM7E2cBab30RMHrHDcEfxjoYZgeFONFh7HgQ";

        let signer: Hmac<Sha256> = Hmac::new_varkey(b"secret").unwrap();
        let computed_signature = SigningAlgorithm::sign(&signer, &header, &claims).unwrap();

        assert_eq!(computed_signature, expected_signature);
    }

    #[test]
    pub fn verify() {
        let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let claims = "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWV9";
        let signature = "TJVA95OrM7E2cBab30RMHrHDcEfxjoYZgeFONFh7HgQ";

        let verifier: Hmac<Sha256> = Hmac::new_varkey(b"secret").unwrap();
        assert!(VerifyingAlgorithm::verify(&verifier, &header, &claims, &signature).unwrap());
    }
}