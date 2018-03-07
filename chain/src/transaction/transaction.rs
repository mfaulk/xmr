use keys::{Signature, SIGNATURE_LENGTH};
use rct::Signature as RctSignature;
use transaction::TransactionPrefix;
use primitives::H256;
use format::{
    Deserialize,
    DeserializerStream,
    Error,
    Serialize,
    SerializerStream,
    from_binary,
    to_binary,
};

/// A transaction.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub prefix: TransactionPrefix,
    pub signature_type: SignatureType,
}

impl Transaction {
    pub fn from_bytes<T: AsRef<[u8]>>(v: T) -> Result<Self, Error> {
        from_binary::<Self>(v.as_ref())
    }

    pub fn id(&self) -> H256 {
        match self.prefix.version {
            1 => H256::fast_hash(to_binary(self)),
            2 => unimplemented!(),
            _ => panic!("invalid tx version"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SignatureType {
    Normal(Vec<Vec<Signature>>),
    RingCt(RctSignature),
}

impl Deserialize for Transaction {
    fn deserialize(deserializer: &mut DeserializerStream) -> Result<Self, Error> {
        let prefix: TransactionPrefix = deserializer.get_deserializable()?;

        let signature_type = match prefix.version {
             1 => {
                let mut signatures = Vec::with_capacity(prefix.vin.len());
                for txin in prefix.vin.iter() {
                    if txin.signature_size() != 0 {
                        let mut txin_sigs = Vec::with_capacity(txin.signature_size());

                        for _ in 0..txin.signature_size() {
                            let blob = deserializer.get_blob(SIGNATURE_LENGTH)?;
                            txin_sigs.push(Signature::from_bytes(blob));
                        }

                        signatures.push(txin_sigs);
                    } else {
                        signatures.push(Vec::new())
                    }
                }

                SignatureType::Normal(signatures)
            },
            2 => unimplemented!(),
            n => return Err(Error::custom(format!("invalid transaction version ({})", n))),
        };

        Ok(Transaction {
            prefix,
            signature_type,
        })
    }
}

impl Serialize for Transaction {
    fn serialize(&self, mut serializer: SerializerStream) {
        serializer.put_serializable(&self.prefix);

        match self.signature_type {
            SignatureType::Normal(ref signatures) => {
                assert_eq!(self.prefix.version, 1, "signature type doesn't match version");

                for sigv in signatures.iter() {
                    for sig in sigv.iter() {
                        serializer.put_blob(sig.as_bytes());
                    }
                }
            },
            SignatureType::RingCt(_) => {
                assert_eq!(self.prefix.version, 2, "signature type doesn't match version");
                unimplemented!()
            },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use format::to_binary;
    use transaction::{
        SignatureType,
        Transaction,
        TransactionPrefix,
        TxInGen,
        TxOut,
        TxOutToKey,
    };

    #[test]
    fn mainnet_genesis_tx_serialization() {
        const GENESIS_TX: &'static [u8] = &[
            0x01, 0x3c, 0x01, 0xff, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0x03, 0x02, 0x9b, 0x2e, 0x4c, 0x02, 0x81, 0xc0, 0xb0, 0x2e,
            0x7c, 0x53, 0x29, 0x1a, 0x94, 0xd1, 0xd0, 0xcb, 0xff, 0x88, 0x83,
            0xf8, 0x02, 0x4f, 0x51, 0x42, 0xee, 0x49, 0x4f, 0xfb, 0xbd, 0x08,
            0x80, 0x71, 0x21, 0x01, 0x77, 0x67, 0xaa, 0xfc, 0xde, 0x9b, 0xe0,
            0x0d, 0xcf, 0xd0, 0x98, 0x71, 0x5e, 0xbc, 0xf7, 0xf4, 0x10, 0xda,
            0xeb, 0xc5, 0x82, 0xfd, 0xa6, 0x9d, 0x24, 0xa2, 0x8e, 0x9d, 0x0b,
            0xc8, 0x90, 0xd1,
        ];

        let genesis_transaction = Transaction {
            prefix: TransactionPrefix {
                version: 1,
                unlock_time: 60,
                vin: vec![TxInGen { height: 0 }.into()],
                vout: vec![
                    TxOut {
                        amount: 17592186044415,
                        target: TxOutToKey {
                            key: [
                                0x9b, 0x2e, 0x4c, 0x02, 0x81, 0xc0, 0xb0, 0x2e, 0x7c,
                                0x53, 0x29, 0x1a, 0x94, 0xd1, 0xd0, 0xcb, 0xff, 0x88,
                                0x83, 0xf8, 0x02, 0x4f, 0x51, 0x42, 0xee, 0x49, 0x4f,
                                0xfb, 0xbd, 0x08, 0x80, 0x71,
                            ].into(),
                        }.into()
                    }
                ],
                extra: vec![1, 119, 103, 170, 252, 222, 155, 224, 13, 207, 208, 152, 113, 94, 188, 247, 244, 16, 218, 235, 197, 130, 253, 166, 157, 36, 162, 142, 157, 11, 200, 144, 209]
            },
            signature_type: SignatureType::Normal(vec![]),
        };

        let blob = to_binary(&genesis_transaction);
        assert_eq!(&*blob, GENESIS_TX);
    }
}
