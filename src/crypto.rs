use blst::min_sig::*;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

#[allow(dead_code)]
pub struct KeyMaterial {
 secret_key: [u8; 32],
 public_key: [u8; 96],
 proof_of_possession: [u8; 48],
}

impl KeyMaterial {
 pub fn generate_new() -> Self {
  let mut seed = [0u8; 32];
  rand::rngs::OsRng.fill_bytes(&mut seed);

  let mut rng = ChaCha20Rng::from_seed(seed);
  let mut ikm = [0u8; 32];
  rng.fill_bytes(&mut ikm);

  let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
  let pk = sk.sk_to_pk();

  dbg!(hex::encode(sk.to_bytes()));
  dbg!(hex::encode(pk.compress()));

  let dst = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_POP_";
  let msg = b"blst is such a blast";
  let sig = sk.sign(msg, dst, &[]);
  let pop = sk.sign(&pk.compress(), b"BLS_POP_BLS12381G1_XMD:SHA-256_SSWU_RO_POP_", &[]);

  dbg!(hex::encode(pop.compress()));
  dbg!(hex::encode(sig.compress()));

  let err = sig.verify(true, msg, dst, &[], &pk, true);
  assert_eq!(err, blst::BLST_ERROR::BLST_SUCCESS);

  KeyMaterial {
   secret_key: sk.to_bytes(),
   public_key: pk.compress(),
   proof_of_possession: pop.compress(),
  }
 }
}
