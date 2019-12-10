/****************************************************************************
 * https://tools.ietf.org/html/rfc5280#section-4.1
 *
 * SubjectPublicKeyInfo  ::=  SEQUENCE  {
 *     algorithm            AlgorithmIdentifier,
 *     subjectPublicKey     BIT STRING  }
 *
 * https://tools.ietf.org/html/rfc8017#appendix-A.1
 *
 * RSAPublicKey ::= SEQUENCE {
 *     modulus           INTEGER,  -- n
 *     publicExponent    INTEGER   -- e
 * }
 ****************************************************************************/
// https://lapo.it/asn1js/#MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsiLoIxmXaZAFRBKtHYZhiF8m-pYR-xGIpupvsdDEvKO92D6fIccgVLIW6p6sSNkoXx5J6KDSMbA_chy5M6pRvJkaCXCI4zlCPMYvPhI8OxN3RYPfdQTLpgPywrlfdn2CAum7o4D8nR4NJacB3NfPnS9tsJ2L3p5iHviuTB4xm03IKmPPqsaJy-nXUFC1XS9E_PseVHRuNvKa7WmlwSZngQzKAVSIwqpgCc-oP1pKEeJ0M3LHFo8ao5SuzhfXUIGrPnkUKEE3m7B0b8xXZfP1N6ELoonWDK-RMgYIBaZdgBhPfHxF8KfTHvSzcUzWZojuR-ynaFL9AJK-8RiXnB4CJwIDAQAB

use super::ocsp_request::AlgorithmIdentifier;
use num_bigint_dig::BigInt;
use oid::prelude::*;
use picky_asn1::wrapper::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SubjectPublicKeyInfoRsa {
    pub algorithm: AlgorithmIdentifier,
    pub subject_public_key: EncapsulatedRSAPublicKey,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RSAPublicKey {
    pub modulus: IntegerAsn1,         // n
    pub public_exponent: IntegerAsn1, // e
}
type EncapsulatedRSAPublicKey = BitStringAsn1Container<RSAPublicKey>;

#[test]
fn subject_public_key_info() {
    let encoded = base64::decode(
        "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAsiLoIx\
         mXaZAFRBKtHYZhiF8m+pYR+xGIpupvsdDEvKO92D6fIccgVLIW6p6sSNk\
         oXx5J6KDSMbA/chy5M6pRvJkaCXCI4zlCPMYvPhI8OxN3RYPfdQTLpgPy\
         wrlfdn2CAum7o4D8nR4NJacB3NfPnS9tsJ2L3p5iHviuTB4xm03IKmPPq\
         saJy+nXUFC1XS9E/PseVHRuNvKa7WmlwSZngQzKAVSIwqpgCc+oP1pKEe\
         J0M3LHFo8ao5SuzhfXUIGrPnkUKEE3m7B0b8xXZfP1N6ELoonWDK+RMgY\
         IBaZdgBhPfHxF8KfTHvSzcUzWZojuR+ynaFL9AJK+8RiXnB4CJwIDAQAB",
    )
    .expect("invalid base64");

    // RSA algorithm identifier

    let rsa_encryption = ObjectIdentifier::try_from("1.2.840.113549.1.1.1").unwrap();
    let algorithm = AlgorithmIdentifier {
        algorithm: rsa_encryption.into(),
        parameters: (),
    };
    check!(algorithm: AlgorithmIdentifier in encoded[4..19]);

    // RSA modulus and public exponent

    let modulus = IntegerAsn1::from_signed_bytes_be(vec![
        0x00, 0xb2, 0x22, 0xe8, 0x23, 0x19, 0x97, 0x69, 0x90, 0x5, 0x44, 0x12, 0xad, 0x1d, 0x86,
        0x61, 0x88, 0x5f, 0x26, 0xfa, 0x96, 0x11, 0xfb, 0x11, 0x88, 0xa6, 0xea, 0x6f, 0xb1, 0xd0,
        0xc4, 0xbc, 0xa3, 0xbd, 0xd8, 0x3e, 0x9f, 0x21, 0xc7, 0x20, 0x54, 0xb2, 0x16, 0xea, 0x9e,
        0xac, 0x48, 0xd9, 0x28, 0x5f, 0x1e, 0x49, 0xe8, 0xa0, 0xd2, 0x31, 0xb0, 0x3f, 0x72, 0x1c,
        0xb9, 0x33, 0xaa, 0x51, 0xbc, 0x99, 0x1a, 0x9, 0x70, 0x88, 0xe3, 0x39, 0x42, 0x3c, 0xc6,
        0x2f, 0x3e, 0x12, 0x3c, 0x3b, 0x13, 0x77, 0x45, 0x83, 0xdf, 0x75, 0x4, 0xcb, 0xa6, 0x3,
        0xf2, 0xc2, 0xb9, 0x5f, 0x76, 0x7d, 0x82, 0x2, 0xe9, 0xbb, 0xa3, 0x80, 0xfc, 0x9d, 0x1e,
        0xd, 0x25, 0xa7, 0x1, 0xdc, 0xd7, 0xcf, 0x9d, 0x2f, 0x6d, 0xb0, 0x9d, 0x8b, 0xde, 0x9e,
        0x62, 0x1e, 0xf8, 0xae, 0x4c, 0x1e, 0x31, 0x9b, 0x4d, 0xc8, 0x2a, 0x63, 0xcf, 0xaa, 0xc6,
        0x89, 0xcb, 0xe9, 0xd7, 0x50, 0x50, 0xb5, 0x5d, 0x2f, 0x44, 0xfc, 0xfb, 0x1e, 0x54, 0x74,
        0x6e, 0x36, 0xf2, 0x9a, 0xed, 0x69, 0xa5, 0xc1, 0x26, 0x67, 0x81, 0xc, 0xca, 0x1, 0x54,
        0x88, 0xc2, 0xaa, 0x60, 0x9, 0xcf, 0xa8, 0x3f, 0x5a, 0x4a, 0x11, 0xe2, 0x74, 0x33, 0x72,
        0xc7, 0x16, 0x8f, 0x1a, 0xa3, 0x94, 0xae, 0xce, 0x17, 0xd7, 0x50, 0x81, 0xab, 0x3e, 0x79,
        0x14, 0x28, 0x41, 0x37, 0x9b, 0xb0, 0x74, 0x6f, 0xcc, 0x57, 0x65, 0xf3, 0xf5, 0x37, 0xa1,
        0xb, 0xa2, 0x89, 0xd6, 0xc, 0xaf, 0x91, 0x32, 0x6, 0x8, 0x5, 0xa6, 0x5d, 0x80, 0x18, 0x4f,
        0x7c, 0x7c, 0x45, 0xf0, 0xa7, 0xd3, 0x1e, 0xf4, 0xb3, 0x71, 0x4c, 0xd6, 0x66, 0x88, 0xee,
        0x47, 0xec, 0xa7, 0x68, 0x52, 0xfd, 0x0, 0x92, 0xbe, 0xf1, 0x18, 0x97, 0x9c, 0x1e, 0x2,
        0x27,
    ]);
    check!(modulus: IntegerAsn1 in encoded[28..289]);

    let public_exponent: IntegerAsn1 = BigInt::from(65537).to_signed_bytes_be().into();
    check!(public_exponent: IntegerAsn1 in encoded[289..294]);

    // RSA public key

    let subject_public_key: EncapsulatedRSAPublicKey = RSAPublicKey {
        modulus,
        public_exponent,
    }
    .into();
    check!(subject_public_key: EncapsulatedRSAPublicKey in encoded[19..294]);

    // full encode / decode

    let info = SubjectPublicKeyInfoRsa {
        algorithm,
        subject_public_key,
    };
    check!(info: SubjectPublicKeyInfoRsa in encoded);
}