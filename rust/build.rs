use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

pub fn main() {
    cc::Build::new()
        .file("netcode.c")
        .include("c")
        .include("windows")
        .define("NETCODE_ENABLE_TESTS", Some("0"))
        .define("NDEBUG", Some("0"))
        .compile("libnetcode.a");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let private_path = out_path.join("private_bindings.rs");

    //Do some basic dependecy management
    let targets = vec![&private_path];
    let source = vec!["rust/build.rs", "netcode.c", "netcode.h"]
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let newest_source = source
        .iter()
        .map(|v| {
            File::open(v)
                .and_then(|f| f.metadata())
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| panic!("Source file {:?} not found", v))
        })
        .max()
        .unwrap();

    let oldest_target = targets
        .iter()
        .filter_map(|v| {
            File::open(v)
                .and_then(|f| f.metadata())
                .and_then(|m| m.modified())
                .ok()
        })
        .min()
        .unwrap_or(newest_source - Duration::from_secs(1));

    if newest_source > oldest_target {
        let include = env::var("INCLUDE").unwrap_or_else(|_| "".to_string());
        let sodium_include = env::var("SODIUM_LIB_DIR").unwrap_or_else(|_| "windows".to_string());

        let private_bindings = bindgen::Builder::default()
            .header("netcode.c")
            .clang_arg("-Ic")
            .clang_arg(format!("-I{}", sodium_include))
            .clang_arg(format!("-I{}", include))
            .whitelist_function("netcode_log_level")
            .whitelist_function("netcode_write_packet")
            .whitelist_function("netcode_read_packet")
            .whitelist_function("netcode_read_connect_token")
            .whitelist_function("netcode_decrypt_challenge_token")
            .whitelist_function("netcode_read_challenge_token")
            .whitelist_function("netcode_replay_protection_reset")
            .whitelist_function("free")
            .whitelist_function("netcode_generate_connect_token")
            .whitelist_function("netcode_init")
            //.whitelist_function("netcode_client_create_internal")
            .whitelist_function("netcode_client_create")
            .whitelist_function("netcode_client_connect")
            .whitelist_function("netcode_client_update")
            .whitelist_function("netcode_client_state")
            .whitelist_function("netcode_client_receive_packet")
            .whitelist_function("netcode_client_free_packet")
            .whitelist_function("netcode_client_destroy")
            .whitelist_function("netcode_client_free_packet")
            .whitelist_function("netcode_client_send_packet")
            .whitelist_function("netcode_term")
            .whitelist_type("netcode_network_simulator_t")
            .whitelist_type("netcode_address_t")
            .whitelist_function("netcode_network_simulator_create")
            .whitelist_function("netcode_network_simulator_destroy")
            .whitelist_function("netcode_network_simulator_update")
            .whitelist_function("netcode_network_simulator_send_packet")
            .whitelist_function("netcode_network_simulator_receive_packets")
            .whitelist_function("netcode_parse_address")
            .whitelist_function("netcode_address_to_string")
            .whitelist_var("NETCODE_MAX_ADDRESS_STRING_LENGTH")
            .whitelist_var("NETCODE_CONNECTION_NUM_PACKETS")
            .whitelist_var("NETCODE_CLIENT_STATE_CONNECTED")
            .whitelist_var("NETCODE_CLIENT_STATE_DISCONNECTED")
            .whitelist_var("NETCODE_CLIENT_STATE_SENDING_CONNECTION_RESPONSE")
            .whitelist_var("NETCODE_CLIENT_STATE_SENDING_CONNECTION_REQUEST")
            .whitelist_var("NETCODE_LOG_LEVEL_DEBUG")
            .whitelist_var("NETCODE_PACKET_SEND_RATE")
            .whitelist_var("NETCODE_CONNECT_TOKEN_BYTES")
            .generate()
            .expect("Unable to generate bindings");

        private_bindings
            .write_to_file(&private_path)
            .expect("Couldn't write bindings!");
    }
}
