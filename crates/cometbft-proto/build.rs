use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf();

    let proto_root = repo_root.join("third_party/cometbft/proto");
    let third_party = repo_root.join("third_party/cometbft/third_party/proto");
    let vendor_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor");

    // Keep builds reproducible when the proto sources change.
    println!("cargo:rerun-if-changed={}", proto_root.display());
    println!("cargo:rerun-if-changed={}", third_party.display());
    println!("cargo:rerun-if-changed={}", vendor_root.display());

    let proto_files = [
        "tendermint/abci/types.proto",
        "tendermint/blocksync/types.proto",
        "tendermint/consensus/types.proto",
        "tendermint/consensus/wal.proto",
        "tendermint/crypto/keys.proto",
        "tendermint/crypto/proof.proto",
        "tendermint/libs/bits/types.proto",
        "tendermint/mempool/types.proto",
        "tendermint/p2p/conn.proto",
        "tendermint/p2p/pex.proto",
        "tendermint/p2p/types.proto",
        "tendermint/privval/types.proto",
        "tendermint/rpc/grpc/types.proto",
        "tendermint/state/types.proto",
        "tendermint/statesync/types.proto",
        "tendermint/store/types.proto",
        "tendermint/types/block.proto",
        "tendermint/types/canonical.proto",
        "tendermint/types/evidence.proto",
        "tendermint/types/events.proto",
        "tendermint/types/params.proto",
        "tendermint/types/types.proto",
        "tendermint/types/validator.proto",
        "tendermint/version/types.proto",
    ]
    .into_iter()
    .map(|p| proto_root.join(p))
    .collect::<Vec<_>>();

    tonic_build::configure()
        .build_server(false)
        .compile_well_known_types(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &proto_files,
            &[proto_root.clone(), third_party.clone(), vendor_root.clone()],
        )?;

    Ok(())
}
