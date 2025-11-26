//! Generated CometBFT protobuf types (built from `third_party/cometbft/proto`).

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod tendermint {
    pub mod abci {
        include!(concat!(env!("OUT_DIR"), "/tendermint.abci.rs"));
    }
    pub mod blocksync {
        include!(concat!(env!("OUT_DIR"), "/tendermint.blocksync.rs"));
    }
    pub mod statesync {
        include!(concat!(env!("OUT_DIR"), "/tendermint.statesync.rs"));
    }
    pub mod consensus {
        include!(concat!(env!("OUT_DIR"), "/tendermint.consensus.rs"));
    }
    pub mod crypto {
        include!(concat!(env!("OUT_DIR"), "/tendermint.crypto.rs"));
    }
    pub mod libs {
        pub mod bits {
            include!(concat!(env!("OUT_DIR"), "/tendermint.libs.bits.rs"));
        }
    }
    pub mod mempool {
        include!(concat!(env!("OUT_DIR"), "/tendermint.mempool.rs"));
    }
    pub mod p2p {
        include!(concat!(env!("OUT_DIR"), "/tendermint.p2p.rs"));
    }
    pub mod privval {
        include!(concat!(env!("OUT_DIR"), "/tendermint.privval.rs"));
    }
    pub mod rpc {
        pub mod grpc {
            include!(concat!(env!("OUT_DIR"), "/tendermint.rpc.grpc.rs"));
        }
    }
    pub mod state {
        include!(concat!(env!("OUT_DIR"), "/tendermint.state.rs"));
    }
    pub mod store {
        include!(concat!(env!("OUT_DIR"), "/tendermint.store.rs"));
    }
    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/tendermint.types.rs"));
    }
    pub mod version {
        include!(concat!(env!("OUT_DIR"), "/tendermint.version.rs"));
    }
}

pub mod google {
    pub mod protobuf {
        include!(concat!(env!("OUT_DIR"), "/google.protobuf.rs"));
    }
}
