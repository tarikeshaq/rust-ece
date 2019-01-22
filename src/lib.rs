/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod aes128gcm;
mod aesgcm;
mod common;
mod crypto_backend;
mod crypto_backends;
mod error;

pub use crate::{
    aes128gcm::Aes128GcmEceWebPush,
    aesgcm::{AesGcmEceWebPush, AesGcmEncryptedBlock},
    common::WebPushParams,
    crypto_backend::{LocalKeyPair, RemotePublicKey},
    error::*,
};

pub type Aes128GcmEceWebPushImpl = aes128gcm::Aes128GcmEceWebPush<crypto_backends::CryptoImpl>;
pub type AesGcmEceWebPushImpl = aesgcm::AesGcmEceWebPush<crypto_backends::CryptoImpl>;
pub use crate::crypto_backends::{LocalKeyPairImpl, RemoteKeyPairImpl};

#[cfg(test)]
mod aes128gcm_tests {
    extern crate hex;
    use super::crypto_backend::Crypto;
    use super::crypto_backends::CryptoImpl;
    use super::*;

    fn generate_keys() -> Result<(LocalKeyPairImpl, LocalKeyPairImpl)> {
        let local_key = LocalKeyPairImpl::generate_random()?;
        let remote_key = LocalKeyPairImpl::generate_random()?;
        Ok((local_key, remote_key))
    }

    fn try_encrypt(
        priv_key: &str,
        pub_key: &str,
        auth_secret: &str,
        salt: &str,
        pad_length: usize,
        rs: u32,
        plaintext: &str,
    ) -> Result<String> {
        let priv_key = hex::decode(priv_key).unwrap();
        let priv_key = LocalKeyPairImpl::new(&priv_key)?;
        let pub_key = hex::decode(pub_key).unwrap();
        let pub_key = CryptoImpl::public_key_from_raw(&pub_key)?;
        let auth_secret = hex::decode(auth_secret).unwrap();
        let salt = hex::decode(salt).unwrap();
        let plaintext = plaintext.as_bytes();
        let params = WebPushParams::new(rs, pad_length, salt);
        let ciphertext = Aes128GcmEceWebPushImpl::encrypt_with_keys(
            &priv_key,
            &pub_key,
            &auth_secret,
            &plaintext,
            params,
        )?;
        Ok(hex::encode(ciphertext))
    }

    fn try_decrypt(priv_key: &str, auth_secret: &str, payload: &str) -> Result<String> {
        let priv_key = hex::decode(priv_key).unwrap();
        let priv_key = LocalKeyPairImpl::new(&priv_key)?;
        let auth_secret = hex::decode(auth_secret).unwrap();
        let payload = hex::decode(payload).unwrap();
        let plaintext = Aes128GcmEceWebPushImpl::decrypt(&priv_key, &auth_secret, &payload)?;
        Ok(String::from_utf8(plaintext).unwrap())
    }

    #[test]
    fn test_e2e() {
        let (local_key, remote_key) = generate_keys().unwrap();
        let plaintext = "When I grow up, I want to be a watermelon".as_bytes();
        let mut auth_secret = vec![0u8; 16];
        CryptoImpl::random(&mut auth_secret).unwrap();
        let remote_public =
            CryptoImpl::public_key_from_raw(&remote_key.pub_as_raw().unwrap()).unwrap();
        let params = WebPushParams::default();
        let ciphertext = Aes128GcmEceWebPushImpl::encrypt_with_keys(
            &local_key,
            &remote_public,
            &auth_secret,
            &plaintext,
            params,
        )
        .unwrap();
        let decrypted =
            Aes128GcmEceWebPushImpl::decrypt(&remote_key, &auth_secret, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn try_encrypt_ietf_rfc() {
        let ciphertext = try_encrypt(
            "c9f58f89813e9f8e872e71f42aa64e1757c9254dcc62b72ddc010bb4043ea11c",
            "042571b2becdfde360551aaf1ed0f4cd366c11cebe555f89bcb7b186a53339173168ece2ebe018597bd30479b86e3c8f8eced577ca59187e9246990db682008b0e",
            "05305932a1c7eabe13b6cec9fda48882",
            "0c6bfaadad67958803092d454676f397",
            0,
            4096,
            "When I grow up, I want to be a watermelon",
        ).unwrap();
        assert_eq!(ciphertext, "0c6bfaadad67958803092d454676f397000010004104fe33f4ab0dea71914db55823f73b54948f41306d920732dbb9a59a53286482200e597a7b7bc260ba1c227998580992e93973002f3012a28ae8f06bbb78e5ec0ff297de5b429bba7153d3a4ae0caa091fd425f3b4b5414add8ab37a19c1bbb05cf5cb5b2a2e0562d558635641ec52812c6c8ff42e95ccb86be7cd");
    }

    #[test]
    fn try_encrypt_rs_24_pad_6() {
        let ciphertext = try_encrypt(
            "0f28beaf7e27793c03638dc2973a15b0016e1b367cbffda8861ab175f31bce02",
            "04c0d1a812b291291dd7beee358713c126c589f3633c26d1a201311de036dc10931e4ee142f61921a3ea5864e872a93841a52944e5b3f6accecce8c828fb04a4cd",
            "9d7735d8de1962b98394b07ffe287e20",
            "ff805030a108e114e6c17fad6186a1a6",
            6,
            24,
            "I am the very model of a modern Major-General, I've information vegetable, animal, and mineral",
        ).unwrap();
        assert_eq!(ciphertext, "ff805030a108e114e6c17fad6186a1a600000018410430efcb1eb043b805e4e44bab35f82513c33fedb28700f7e568ac8b61e8d835665a51eb6679b2db228a10c0c3fe5077062848d9bb3d60279f93ce35484728aa1fd2c1713949aec98f05096c7298fd3f51c4f818fafa1fe615d8447b3a05406031f6401ac24f2a775ca52456a921b83b9e0042c3a63e1afa1ae012774d9d775be8d19419451d37ff59ff592e84f07440a63fc17f5cabcb9a50eddaf75370db647f94447d3f166269d8711df0f57e56049576e1130a5a5e1f94ba8a5d0b0007c6c0fd2998429e7d63d4ef919798f46ecf5f0b28fb80f5b2439de26b8a52200bc7d6af7a4840721fe8be8524a691b6ef0edae90bb6f5927894819b831b45b53f8401fe022dbb64ed7565350904ac0b517135d7f8abbc98127fb163864d4d4a307425b2cd43db22af267d71c37146994a8c4805adc341bfba27af09fd80bd5eff51d877282a2fbfbfeb10199e7879e4b9d13a46d57fb7d786824853e1cc89cafbaf14de1e924c944feb8b626ce0207d6f9fa9d849eecac69b42d6e7a23bd5124d49622b44b35c5b15fb0e6a7781a503f1a4e062e015d557d95d44d9d8b0799b3aafce83d5d4");
    }

    #[test]
    fn try_encrypt_rs_18_pad_31() {
        // This test is also interesting because the data length (54) is a
        // multiple of rs (18). We'll allocate memory to hold 4 records, but only
        // write 3.
        let ciphertext = try_encrypt(
            "7830577bafcfc45828da0c40aab09fb227bfeae068aab8c064222acbe6effd34",
            "04c3d714cb42e2b0a1d6f98599e2f186b8c2ba6f6fab5e09a2abca865c0805892b2c3729330ef83dc9df4b44362b039a0609d36beb9321a431ec123506ddd90f24",
            "e4d7b79decdede12c3e9d90d3e05730f",
            "e49888d2b28f277f847bc5de96f0f81b",
            31,
            18,
            "Push the button, Frank!",
        ).unwrap();
        assert_eq!(ciphertext, "e49888d2b28f277f847bc5de96f0f81b00000012410400b833e481a99aa330dcb277922d5f84af2e9ce611ad2ad3ed0f5b431912d35ea72fc5bf76b769d9526778f5abfa058650988da5e531ff82d1a7043794c717063aeb958bf116bccf50742fd4d69bd0ea7e3f611c709bf2cdf5cd47c6426cb8323b5398c43c0d0b92cc982da1c24ce5fee2b203f7ad78ca44f0490f3407f5fee883266ee47035195de0fe6d8a75e487df256db597a75e45ae4fb55b8259cb0b2d19e7b05714267eb560ae072b7a665951917a068732df309be256f90f2adda32f05feaa5e9b0695bca2ccf22aaefc7da9ceebc5d40c12d32adb5c84cb320af944016095362febba4ffa4a99830e4958ea2bba508cb683a58d2027d4b74726a853b24b47ccba751abe9d9ab2da9ec2ba9c7ccf0cf17305bae314d38a687618b0772fcb71d4419027a4bf435cb721aad74efc179981b7169604bf97ecac41e73884456933734818132923b56c152d6c9e59aef995aca59de0bf2c803a07180889670a08e64a20d2bfa853e0112872947baaaffb510cc9e75d6310ed6aacbd2e0ba3a29be42c6532ea4e3346e1f0571646371c71665e3fac9d76faee1f122e64d490dd2a3e31816eab583f172841a075d205f318714a8c70ce0f327f4d92b8c9dcb813e6d24fe85633f1a9c7c1e4a1fb314dd5fe3e280e3908f36c8cbfb80b7d9243abaffa65c216cf1aa8b8d626a630dfe8186ce977a5b8f3649d3753b9176c367e4e07f220a175806138e88825a2f3498420582b96209658bbfa8f2ba6933a83c25edb269187796542e2ac49b8078636bddc268e11625e8bff9f0a343d3a4c06080ef0803b8dcd8e841d0e2759e483ea19b903324d9ec4d52f491acef3eeff441c37881c7593eac31621337a5e8659f93e20079b0e26ebfe56c10455d10971130bd2a2c159c74f48b2e526530a76f64cca2efb246e793d11fb75a668018e70c3107100f81ba3b16ae40a838f18d4c47f1d7132f174688ec5382394e0119921731a16879b858ff38f72851ea3d9f5263fec5a606d1271a89b84cca53ed73c5254e245bf8f2f27c2c1c87f39eea78c7017c8c6b5ab01663032b58da31057285e56c203f4e48d6789c66b2695a900e00482bd846559ecddd40264b38e279647d1ec0fccdc1881838bbe0c835e2690ef058b8f6a03e29cd9eb9584e97fbc309773c3688e5e03f9d38e3e4548738a5f569c59147d3e823cccac71d5e8825d5134ce9813cd0b8f9627a3dbfa45b83a59c83d2b4d3ad437778a3cb1bc77ba16c92306f4261a2a1f0d5c7edaecf926f92d7c9dfcae87513a68b8c7ef7c63264b858767c11aaa41d27c636f52e28551e93a969cdc96d43867b7cbd68fe0357bd33415faf22aaeebc957f4b5737a04ab7277b4ed4008f09edaff5a6db69f6cb06f3d0b76688906b2f53b27e63f3728ba2eda505fb1b32f81dddc6d305fd5949edd05490cb1618f0ce1430e9f5edf50012dc3");
    }

    #[test]
    fn test_decrypt_rs_24_pad_0() {
        let plaintext = try_decrypt(
            "c899d11d32e2b7e6fe7498786f50f23b98ace5397ad261de39ba6449ecc12cad",
            "996fad8b50aa2d02b83f26412b2e2aee",
            "495ce6c8de93a4539e862e8634993cbb0000001841043c3378a2c0ab954e1498718e85f08bb723fb7d25e135a663fe385884eb8192336bf90a54ed720f1c045c0b405e9bbc3a2142b16c89086734c374ebaf7099e6427e2d32c8ada5018703c54b10b481e1027d7209d8c6b43553fa133afa597f2ddc45a5ba8140944e6490bb8d6d99ba1d02e60d95f48ce644477c17231d95b97a4f95dd"
        ).unwrap();
        assert_eq!(plaintext, "I am the walrus");
    }

    #[test]
    fn test_decrypt_rs_49_pad_84_ciphertext_len_falls_on_record_boundary() {
        let plaintext = try_decrypt(
            "67004a4ea820deed8e49db5e9480e63d3ea3cce1ae8e1a60609713d527d001ef",
            "95f17570e508ef6a2b2ad1b4f5cade33",
            "fb2883cec1c4fcadd6d1371f6ea491e00000003141042d441ee7f9ff6a0329a64927d0524fdbe7b22c6fb65e10ab4fdc038f94420a0ca3fa28dad36c84ec91a162eae078faad2c1ced78de8113e19602b20e894f4976b973e2fcf682fa0c8ccd9af3d5bff1ede16fad5a31ce19d38b5e1fe1f78a4fad842bbc10254c2c6cdd96a2b55284d972c53cad8c3bacb10f5f57eb0d4a4333b604102ba117cae29108fbd9f629a8ba6960dd01945b39ed37ba706c434a10fd2bd2094ff9249bcdad45135f5fe45fcd38071f8b2d3941afda439810d77aacaf7ce50b54325bf58c9503337d073785a323dfa343"
        ).unwrap();
        assert_eq!(plaintext, "Hello, world");
    }

    #[test]
    fn test_decrypt_ietf_rfc() {
        let plaintext = try_decrypt(
            "ab5757a70dd4a53e553a6bbf71ffefea2874ec07a6b379e3c48f895a02dc33de",
            "05305932a1c7eabe13b6cec9fda48882",
            "0c6bfaadad67958803092d454676f397000010004104fe33f4ab0dea71914db55823f73b54948f41306d920732dbb9a59a53286482200e597a7b7bc260ba1c227998580992e93973002f3012a28ae8f06bbb78e5ec0ff297de5b429bba7153d3a4ae0caa091fd425f3b4b5414add8ab37a19c1bbb05cf5cb5b2a2e0562d558635641ec52812c6c8ff42e95ccb86be7cd"
        ).unwrap();
        assert_eq!(plaintext, "When I grow up, I want to be a watermelon");
    }

    #[test]
    fn test_decrypt_rs_18_pad_0() {
        let plaintext = try_decrypt(
            "27433fab8970b3cb5284b61183efb46286562cd2a7330d8cae960911a5571d0c",
            "d65a04df95f2db5e604839f717dcde79",
            "7caebdbc20938ee340a946f1bd4f68f100000012410437cfdb5223d9f95eaa02f6ed940ff22eaf05b3622e949dc3ce9f335e6ef9b26aeaacca0f74080a8b364592f2ccc6d5eddd43004b70b91887d144d9fa93f16c3bc7ea68f4fd547a94eca84b16e138a6080177"
        ).unwrap();
        assert_eq!(plaintext, "1");
    }

    #[test]
    fn test_decrypt_missing_header_block() {
        let err = try_decrypt(
            "1be83f38332ef09681faf3f307b1ff2e10cab78cc7cdab683ac0ee92ac3f6ee1",
            "3471bb98481e02533bf39542bcf3dba4",
            "45b74d2b69be9b074de3b35aa87e7c15611d",
        )
        .unwrap_err();
        match err.kind() {
            ErrorKind::HeaderTooShort => {}
            _ => assert!(false),
        };
    }

    #[test]
    fn test_decrypt_truncated_sender_key() {
        let err = try_decrypt(
            "ce88e8e0b3057a4752eb4c8fa931eb621c302da5ad03b81af459cf6735560cae",
            "5c31e0d96d9a139899ac0969d359f740",
            "de5b696b87f1a15cb6adebdd79d6f99e000000120100b6bc1826c37c9f73dd6b4859c2b505181952",
        )
        .unwrap_err();
        match err.kind() {
            ErrorKind::InvalidKeyLength => {}
            _ => assert!(false),
        };
    }

    #[test]
    fn test_decrypt_truncated_auth_secret() {
        let err = try_decrypt(
            "60c7636a517de7039a0ac2d0e3064400794c78e7e049398129a227cee0f9a801",
            "355a38cd6d9bef15990e2d3308dbd600",
            "8115f4988b8c392a7bacb43c8f1ac5650000001241041994483c541e9bc39a6af03ff713aa7745c284e138a42a2435b797b20c4b698cf5118b4f8555317c190eabebfab749c164d3f6bdebe0d441719131a357d8890a13c4dbd4b16ff3dd5a83f7c91ad6e040ac42730a7f0b3cd3245e9f8d6ff31c751d410cfd"
        ).unwrap_err();
        match err.kind() {
            ErrorKind::OpenSSLError(_) => {}
            _ => assert!(false),
        };
    }

    #[test]
    fn test_decrypt_early_final_record() {
        let err = try_decrypt(
            "5dda1d918bc407ba3cda12cb8014d49aa7e0269002820304466bc80034ca9240",
            "40c241fde4269ee1e6d725592d982718",
            "dbe215507d1ad3d2eaeabeae6e874d8f0000001241047bc4343f34a8348cdc4e462ffc7c40aa6a8c61a739c4c41d45125505f70e9fc5f9efa86852dd488dcf8e8ea2cafb75e07abd5ee7c9d5c038bafef079571b0bda294411ce98c76dd031c0e580577a4980a375e45ed30429be0e2ee9da7e6df8696d01b8ec"
        ).unwrap_err();
        match err.kind() {
            ErrorKind::DecryptPadding => {}
            _ => assert!(false),
        };
    }
}

// =====================
#[cfg(test)]
mod aesgcm_tests {
    extern crate base64;
    extern crate hex;

    use super::crypto_backend::Crypto;
    use super::crypto_backends::CryptoImpl;
    use super::*;

    fn generate_keys() -> Result<(LocalKeyPairImpl, LocalKeyPairImpl)> {
        let local_key = LocalKeyPairImpl::generate_random()?;
        let remote_key = LocalKeyPairImpl::generate_random()?;
        Ok((local_key, remote_key))
    }

    fn try_decrypt(
        priv_key: &str,
        auth_secret: &str,
        block: &AesGcmEncryptedBlock,
    ) -> Result<String> {
        // The AesGcmEncryptedBlock is composed from the `Crypto-Key` & `Encryption` headers, and post body
        // The Block will attempt to decode the base64 strings for dh & salt, so no additional action needed.
        // Since the body is most likely not encoded, it is expected to be a raw buffer of [u8]
        let priv_key_raw = base64::decode_config(priv_key, base64::URL_SAFE_NO_PAD)?;
        let priv_key = LocalKeyPairImpl::new(&priv_key_raw)?;
        let auth_secret = base64::decode_config(auth_secret, base64::URL_SAFE_NO_PAD)?;
        let plaintext = AesGcmEceWebPushImpl::decrypt(&priv_key, &auth_secret, &block)?;
        Ok(String::from_utf8(plaintext).unwrap())
    }

    #[test]
    fn test_decode() {
        // generated the content using pywebpush, which verified against the client.
        let auth_raw = "LsuUOBKVQRY6-l7_Ajo-Ag";
        let priv_key_raw = "yerDmA9uNFoaUnSt2TkWWLwPseG1qtzS2zdjUl8Z7tc";
        //let pub_key_raw = "BLBlTYure2QVhJCiDt4gRL0JNmUBMxtNB5B6Z1hDg5h-Epw6mVFV4whoYGBlWNY-ENR1FObkGFyMf7-6ZMHMAxw";

        // Incoming Crypto-Key: dh=
        let dh = "BJvcyzf8ocm6F7lbFePebtXU7OHkmylXN9FL2g-yBHwUKqo6cD-FP1h5SHEQQ-xEgJl-F0xEEmSaEx2-qeJHYmk";
        // Incoming Encryption-Key: salt=
        let salt = "8qX1ZgkLD50LHgocZdPKZQ";
        // Incoming Body (this is normally raw bytes. It's encoded here for presentation)
        let ciphertext = base64::decode_config("8Vyes671P_VDf3G2e6MgY6IaaydgR-vODZZ7L0ZHbpCJNVaf_2omEms2tiPJiU22L3BoECKJixiOxihcsxWMjTgAcplbvfu1g6LWeP4j8dMAzJionWs7OOLif6jBKN6LGm4EUw9e26EBv9hNhi87-HaEGbfBMGcLvm1bql1F",
            base64::URL_SAFE_NO_PAD).unwrap();
        let plaintext = "Amidst the mists and coldest frosts I thrust my fists against the\nposts and still demand to see the ghosts.\n";

        let block = AesGcmEncryptedBlock::new(
            &base64::decode_config(dh, base64::URL_SAFE_NO_PAD).unwrap(),
            &base64::decode_config(salt, base64::URL_SAFE_NO_PAD).unwrap(),
            4096,
            ciphertext,
        )
        .unwrap();

        let result = try_decrypt(priv_key_raw, auth_raw, &block).unwrap();

        assert!(result == plaintext)
    }

    #[test]
    fn test_e2e() {
        let (local_key, remote_key) = generate_keys().unwrap();
        let plaintext = "When I grow up, I want to be a watermelon".as_bytes();
        let mut auth_secret = vec![0u8; 16];
        CryptoImpl::random(&mut auth_secret).unwrap();
        let remote_public =
            CryptoImpl::public_key_from_raw(&remote_key.pub_as_raw().unwrap()).unwrap();
        let params = WebPushParams::default();
        let ciphertext = AesGcmEceWebPushImpl::encrypt_with_keys(
            &local_key,
            &remote_public,
            &auth_secret,
            &plaintext,
            params,
        )
        .unwrap();
        let decrypted =
            AesGcmEceWebPushImpl::decrypt(&remote_key, &auth_secret, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    // If decode using externally validated data works, and e2e using the same decoder work, things
    // should encode/decode.
    // Other tests to be included if required, but skipping for now because of time constraints.
}
