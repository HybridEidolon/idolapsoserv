// use psocrypto::pc::PcCipher;
//
// pub struct Server {
//     config: Config,
//     server_cipher: PcCipher
// }
//
// #[derive(Clone, RustcSerializable, RustcDeserializable)]
// pub struct Config {
//     motd: String
// }
//
// impl Server {
//     pub fn new(server_seed: u32, config: Config) -> Server {
//         Server {
//             config: Config,
//             server_cipher: PcCipher::new(server_seed)
//         }
//     }
// }
