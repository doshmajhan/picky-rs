use crate::{
    pem::{to_pem, Pem},
    private::{private_key_info::PrivateKeyValue, PrivateKeyInfo, SubjectPublicKeyInfo},
};
use picky_asn1::wrapper::{IntegerAsn1, OctetStringAsn1Container};
use picky_asn1_der::Asn1DerError;
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum KeyError {
    /// asn1 serialization error
    #[snafu(display("(asn1) couldn't serialize {}: {}", element, source))]
    Asn1Serialization {
        element: &'static str,
        source: Asn1DerError,
    },

    /// asn1 deserialization error
    #[snafu(display("(asn1) couldn't deserialize {}: {}", element, source))]
    Asn1Deserialization {
        element: &'static str,
        source: Asn1DerError,
    },

    /// RSA error
    #[snafu(display("RSA error: {}", context))]
    Rsa { context: String },

    /// invalid PEM label error
    #[snafu(display("invalid PEM label: {}", label))]
    InvalidPemLabel { label: String },
}

impl From<rsa::errors::Error> for KeyError {
    fn from(e: rsa::errors::Error) -> Self {
        KeyError::Rsa { context: e.to_string() }
    }
}

// === private key === //

const PRIVATE_KEY_PEM_LABEL: &str = "PRIVATE KEY";
const RSA_PRIVATE_KEY_PEM_LABEL: &str = "RSA PRIVATE KEY";

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateKey(PrivateKeyInfo);

impl From<PrivateKeyInfo> for PrivateKey {
    fn from(key: PrivateKeyInfo) -> Self {
        Self(key)
    }
}

impl From<PrivateKey> for PrivateKeyInfo {
    fn from(key: PrivateKey) -> Self {
        key.0
    }
}

impl From<PrivateKey> for SubjectPublicKeyInfo {
    fn from(key: PrivateKey) -> Self {
        match key.0.private_key {
            PrivateKeyValue::RSA(OctetStringAsn1Container(key)) => {
                let (modulus, public_exponent) = key.into_public_components();
                SubjectPublicKeyInfo::new_rsa_key(modulus, public_exponent)
            }
        }
    }
}

impl PrivateKey {
    pub fn from_pem(pem: &Pem) -> Result<Self, KeyError> {
        match pem.label() {
            PRIVATE_KEY_PEM_LABEL => Self::from_pkcs8(pem.data()),
            RSA_PRIVATE_KEY_PEM_LABEL => Self::from_rsa_der(pem.data()),
            _ => Err(KeyError::InvalidPemLabel {
                label: pem.label().to_owned(),
            }),
        }
    }

    pub fn from_pkcs8<T: ?Sized + AsRef<[u8]>>(pkcs8: &T) -> Result<Self, KeyError> {
        Ok(Self(picky_asn1_der::from_bytes(pkcs8.as_ref()).context(
            Asn1Deserialization {
                element: "private key info (pkcs8)",
            },
        )?))
    }

    pub fn from_rsa_der<T: ?Sized + AsRef<[u8]>>(der: &T) -> Result<Self, KeyError> {
        use crate::{private::private_key_info::RSAPrivateKey, AlgorithmIdentifier};

        let private_key = picky_asn1_der::from_bytes::<RSAPrivateKey>(der.as_ref()).context(Asn1Deserialization {
            element: "rsa private key",
        })?;

        Ok(Self(PrivateKeyInfo {
            version: 0,
            private_key_algorithm: AlgorithmIdentifier::new_rsa_encryption(),
            private_key: PrivateKeyValue::RSA(private_key.into()),
        }))
    }

    pub fn to_pkcs8(&self) -> Result<Vec<u8>, KeyError> {
        picky_asn1_der::to_vec(&self.0).context(Asn1Serialization {
            element: "private key info (pkcs8)",
        })
    }

    pub fn to_pem(&self) -> Result<String, KeyError> {
        Ok(to_pem(PRIVATE_KEY_PEM_LABEL, &self.to_pkcs8()?))
    }

    pub fn to_public_key(&self) -> PublicKey {
        match &self.0.private_key {
            PrivateKeyValue::RSA(OctetStringAsn1Container(key)) => {
                SubjectPublicKeyInfo::new_rsa_key(key.modulus().clone(), key.public_exponent().clone()).into()
            }
        }
    }

    /// **Beware**: this is insanely slow in debug builds.
    pub fn generate_rsa(bits: usize) -> Result<Self, KeyError> {
        use rand::rngs::OsRng;
        use rsa::{PublicKey, RSAPrivateKey};

        let key = RSAPrivateKey::new(&mut OsRng, bits)?;
        let modulus = IntegerAsn1::from_signed_bytes_be(key.n().to_bytes_be());
        let public_exponent = IntegerAsn1::from_signed_bytes_be(key.e().to_bytes_be());
        let private_exponent = IntegerAsn1::from_signed_bytes_be(key.d().to_bytes_be());

        Ok(Self(PrivateKeyInfo::new_rsa_encryption(
            modulus,
            public_exponent,
            private_exponent,
            key.primes()
                .iter()
                .map(|p| IntegerAsn1::from_signed_bytes_be(p.to_bytes_be()))
                .collect(),
        )))
    }

    pub(crate) fn as_inner(&self) -> &PrivateKeyInfo {
        &self.0
    }
}

// === public key === //

const PUBLIC_KEY_PEM_LABEL: &str = "PUBLIC KEY";
const RSA_PUBLIC_KEY_PEM_LABEL: &str = "RSA PUBLIC KEY";

#[derive(Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct PublicKey(SubjectPublicKeyInfo);

impl<'a> From<&'a SubjectPublicKeyInfo> for &'a PublicKey {
    #[inline]
    fn from(spki: &'a SubjectPublicKeyInfo) -> Self {
        unsafe { &*(spki as *const SubjectPublicKeyInfo as *const PublicKey) }
    }
}

impl<'a> From<&'a PublicKey> for &'a SubjectPublicKeyInfo {
    #[inline]
    fn from(key: &'a PublicKey) -> Self {
        unsafe { &*(key as *const PublicKey as *const SubjectPublicKeyInfo) }
    }
}

impl From<SubjectPublicKeyInfo> for PublicKey {
    #[inline]
    fn from(spki: SubjectPublicKeyInfo) -> Self {
        Self(spki)
    }
}

impl From<PublicKey> for SubjectPublicKeyInfo {
    #[inline]
    fn from(key: PublicKey) -> Self {
        key.0
    }
}

impl From<PrivateKey> for PublicKey {
    #[inline]
    fn from(key: PrivateKey) -> Self {
        Self(key.into())
    }
}

impl AsRef<SubjectPublicKeyInfo> for PublicKey {
    #[inline]
    fn as_ref(&self) -> &SubjectPublicKeyInfo {
        self.into()
    }
}

impl AsRef<PublicKey> for PublicKey {
    #[inline]
    fn as_ref(&self) -> &PublicKey {
        self
    }
}

impl PublicKey {
    pub fn to_der(&self) -> Result<Vec<u8>, KeyError> {
        picky_asn1_der::to_vec(&self.0).context(Asn1Serialization {
            element: "subject public key info",
        })
    }

    pub fn to_pem(&self) -> Result<String, KeyError> {
        Ok(to_pem(PUBLIC_KEY_PEM_LABEL, &self.to_der()?))
    }

    pub fn from_pem(pem: &Pem) -> Result<Self, KeyError> {
        match pem.label() {
            PUBLIC_KEY_PEM_LABEL => Self::from_der(pem.data()),
            RSA_PUBLIC_KEY_PEM_LABEL => Self::from_rsa_der(pem.data()),
            _ => Err(KeyError::InvalidPemLabel {
                label: pem.label().to_owned(),
            }),
        }
    }

    pub fn from_der<T: ?Sized + AsRef<[u8]>>(der: &T) -> Result<Self, KeyError> {
        Ok(Self(picky_asn1_der::from_bytes(der.as_ref()).context(
            Asn1Deserialization {
                element: "subject public key info",
            },
        )?))
    }

    pub fn from_rsa_der<T: ?Sized + AsRef<[u8]>>(der: &T) -> Result<Self, KeyError> {
        use crate::{
            private::subject_public_key_info::{PublicKey, RSAPublicKey},
            AlgorithmIdentifier,
        };

        let public_key = picky_asn1_der::from_bytes::<RSAPublicKey>(der.as_ref()).context(Asn1Deserialization {
            element: "rsa public key",
        })?;

        Ok(Self(SubjectPublicKeyInfo {
            algorithm: AlgorithmIdentifier::new_rsa_encryption(),
            subject_public_key: PublicKey::RSA(public_key.into()),
        }))
    }

    pub(crate) fn as_inner(&self) -> &SubjectPublicKeyInfo {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generating RSA keys in debug is very slow. Therefore, this test only run in release mode.
    cfg_if::cfg_if! { if #[cfg(not(debug_assertions))] {
        cfg_if::cfg_if! { if #[cfg(feature = "x509")] {
            use crate::x509::{certificate::CertificateBuilder, date::UTCDate, name::DirectoryName};

            fn generate_certificate_from_pk(private_key: PrivateKey) {
                // validity
                let valid_from = UTCDate::ymd(2019, 10, 10).unwrap();
                let valid_to = UTCDate::ymd(2019, 10, 11).unwrap();

                CertificateBuilder::new()
                    .valididy(valid_from, valid_to)
                    .self_signed(DirectoryName::new_common_name("Test Root CA"), &private_key)
                    .ca(true)
                    .build()
                    .expect("couldn't build root ca");
            }
        } else {
            fn generate_certificate_from_pk(_: PrivateKey) {}
        }}

        #[test]
        fn generate_rsa_keys() {
            let private_key = PrivateKey::generate_rsa(4096).expect("couldn't generate rsa key");
            generate_certificate_from_pk(private_key);
        }
    }}

    const RSA_PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----\n\
                                       MIIEpAIBAAKCAQEA5Kz4i/+XZhiE+fyrgtx/4yI3i6C6HXbC4QJYpDuSUEKN2bO9\n\
                                       RsE+Fnds/FizHtJVWbvya9ktvKdDPBdy58+CIM46HEKJhYLnBVlkEcg9N2RNgR3x\n\
                                       HnpRbKfv+BmWjOpSmWrmJSDLY0dbw5X5YL8TU69ImoouCUfStyCgrpwkctR0GD3G\n\
                                       fcGjbZRucV7VvVH9bS1jyaT/9yORyzPOSTwb+K9vOr6XlJX0CGvzQeIOcOimejHx\n\
                                       ACFOCnhEKXiwMsmL8FMz0drkGeMuCODY/OHVmAdXDE5UhroL0oDhSmIrdZ8CxngO\n\
                                       xHr1WD2yC0X0jAVP/mrxjSSfBwmmqhSMmONlvQIDAQABAoIBAQCJrBl3L8nWjayB\n\
                                       VL1ta5MTC+alCX8DfhyVmvQC7FqKN4dvKecqUe0vWXcj9cLhK4B3JdAtXfNLQOgZ\n\
                                       pYRoS2XsmjwiB20EFGtBrS+yBPvV/W0r7vrbfojHAdRXahBZhjl0ZAdrEvNgMfXt\n\
                                       Kr2YoXDhUQZFBCvzKmqSFfKnLRpEhsCBOsp+Sx0ZbP3yVPASXnqiZmKblpY4qcE5\n\
                                       KfYUO0nUWBSzY8I5c/29IY5oBbOUGS1DTMkx3R7V0BzbH/xmskVACn+cMzf467vp\n\
                                       yupTKG9hIX8ff0QH4Ggx88uQTRTI9IvfrAMnICFtR6U7g70hLN6j9ujXkPNhmycw\n\
                                       E5nQCmuBAoGBAPVbYtGBvnlySN73UrlyJ1NItUmOGhBt/ezpRjMIdMkJ6dihq7i2\n\
                                       RpE76sRvwHY9Tmw8oxR/V1ITK3dM2jZP1SRcm1mn5Y1D3K38jwFS0C47AXzIN2N+\n\
                                       LExekI1J4YOPV9o378vUKQuWpbQrQOOvylQBkRJ0Cd8DI3xhiBT/AVGbAoGBAO6Y\n\
                                       WBP3GMloO2v6PHijhRqrNdaI0qht8tDhO5L1troFLst3sfpK9fUP/KTlhHOzNVBF\n\
                                       fIJnNdcYAe9BISBbfSat+/R9F+GoUvpoC4j8ygHTQkT6ZMcMDfR8RQ4BlqGHIDKZ\n\
                                       YaAJoPZVkg7hNRMcvIruYpzFrheDE/4xvnC51GeHAoGAHzCFyFIw72lKwCU6e956\n\
                                       B0lH2ljZEVuaGuKwjM43YlMDSgmLNcjeAZpXRq9aDO3QKUwwAuwJIqLTNLAtURgm\n\
                                       5R9slCIWuTV2ORvQ5f8r/aR8lOsyt1ATu4WN5JgOtdWj+laAAi4vJYz59YRGFGuF\n\
                                       UdZ9JZZgptvUR/xx+xFLjp8CgYBMRzghaeXqvgABTUb36o8rL4FOzP9MCZqPXPKG\n\
                                       0TdR0UZcli+4LS7k4e+LaDUoKCrrNsvPhN+ZnHtB2jiU96rTKtxaFYQFCKM+mvTV\n\
                                       HrwWSUvucX62hAwSFYieKbPWgDSy+IZVe76SAllnmGg3bAB7CitMo4Y8zhMeORkB\n\
                                       QOe/EQKBgQDgeNgRud7S9BvaT3iT7UtizOr0CnmMfoF05Ohd9+VE4ogvLdAoDTUF\n\
                                       JFtdOT/0naQk0yqIwLDjzCjhe8+Ji5Y/21pjau8bvblTnASq26FRRjv5+hV8lmcR\n\
                                       zzk3Y05KXvJL75ksJdomkzZZb0q+Omf3wyjMR8Xl5WueJH1fh4hpBw==\n\
                                       -----END RSA PRIVATE KEY-----";

    #[test]
    fn private_key_from_rsa_pem() {
        PrivateKey::from_pem(&RSA_PRIVATE_KEY_PEM.parse::<Pem>().expect("pem")).expect("private key");
    }

    const PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----\n\
                                  MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA61BjmfXGEvWmegnBGSuS\n\
                                  +rU9soUg2FnODva32D1AqhwdziwHINFaD1MVlcrYG6XRKfkcxnaXGfFDWHLEvNBS\n\
                                  EVCgJjtHAGZIm5GL/KA86KDp/CwDFMSwluowcXwDwoyinmeOY9eKyh6aY72xJh7n\n\
                                  oLBBq1N0bWi1e2i+83txOCg4yV2oVXhBo8pYEJ8LT3el6Smxol3C1oFMVdwPgc0v\n\
                                  Tl25XucMcG/ALE/KNY6pqC2AQ6R2ERlVgPiUWOPatVkt7+Bs3h5Ramxh7XjBOXeu\n\
                                  lmCpGSynXNcpZ/06+vofGi/2MlpQZNhHAo8eayMp6FcvNucIpUndo1X8dKMv3Y26\n\
                                  ZQIDAQAB\n\
                                  -----END PUBLIC KEY-----";

    #[test]
    fn public_key_from_pem() {
        PublicKey::from_pem(&PUBLIC_KEY_PEM.parse::<Pem>().expect("pem")).expect("public key");
    }

    const RSA_PUBLIC_KEY_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----\n\
                                      MIIBCgKCAQEA61BjmfXGEvWmegnBGSuS+rU9soUg2FnODva32D1AqhwdziwHINFa\n\
                                      D1MVlcrYG6XRKfkcxnaXGfFDWHLEvNBSEVCgJjtHAGZIm5GL/KA86KDp/CwDFMSw\n\
                                      luowcXwDwoyinmeOY9eKyh6aY72xJh7noLBBq1N0bWi1e2i+83txOCg4yV2oVXhB\n\
                                      o8pYEJ8LT3el6Smxol3C1oFMVdwPgc0vTl25XucMcG/ALE/KNY6pqC2AQ6R2ERlV\n\
                                      gPiUWOPatVkt7+Bs3h5Ramxh7XjBOXeulmCpGSynXNcpZ/06+vofGi/2MlpQZNhH\n\
                                      Ao8eayMp6FcvNucIpUndo1X8dKMv3Y26ZQIDAQAB\n\
                                      -----END RSA PUBLIC KEY-----";

    #[test]
    fn public_key_from_rsa_pem() {
        PublicKey::from_pem(&RSA_PUBLIC_KEY_PEM.parse::<Pem>().expect("pem")).expect("public key");
    }

    const GARBAGE_PEM: &str = "-----BEGIN GARBAGE-----GARBAGE-----END GARBAGE-----";

    #[test]
    fn public_key_from_garbage_pem_err() {
        let err = PublicKey::from_pem(&GARBAGE_PEM.parse::<Pem>().expect("pem"))
            .err()
            .expect("key error");
        assert_eq!(err.to_string(), "invalid PEM label: GARBAGE");
    }
}
