use fvm_integration_tests::tester::{Account, Tester};
use fvm_integration_tests::dummy::DummyExterns;
use fvm_integration_tests::bundle;
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::state::StateTreeVersion;
use fvm_shared::version::NetworkVersion;
use fvm_ipld_blockstore::MemoryBlockstore;
use std::env;
use fvm_shared::address::Address;
use fvm_shared::message::Message;
use fvm::executor::{ApplyKind, Executor};
use fil_actor_eam::Return;
use fvm_ipld_encoding::RawBytes;
use fil_actors_runtime::{EAM_ACTOR_ADDR, DATACAP_TOKEN_ACTOR_ADDR};
use fvm_shared::ActorID;
use fvm_shared::econ::TokenAmount;

const WASM_COMPILED_PATH: &str =
   "../../build/v0.8/DataCapAPI.bin";

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct Create2Params {
    #[serde(with = "strict_bytes")]
    pub initcode: Vec<u8>,
    #[serde(with = "strict_bytes")]
    pub salt: [u8; 32],
}

fn main() {
    println!("Testing solidity API");

    let bs = MemoryBlockstore::default();
    let actors = std::fs::read("../builtin-actors/output/builtin-actors-devnet-wasm.car").expect("Unable to read actor devnet file file");
    let bundle_root = bundle::import_bundle(&bs, &actors).unwrap();

    let mut tester =
        Tester::new(NetworkVersion::V18, StateTreeVersion::V5, bundle_root, bs).unwrap();

    // As the governor address for datacap is 200, we create many many address in order to initialize the ID 200 with some tokens
    // and make it a valid address to use.
    let sender: [Account; 300] = tester.create_accounts().unwrap();

    // Create embryo address to deploy the contract on it (assign some FILs to it)
    let tmp = hex::decode("DAFEA492D9c6733ae3d56b7Ed1ADB60692c98Bc5").unwrap();
    let embryo_eth_address = tmp.as_slice();
    let embryo_delegated_address = Address::new_delegated(10, embryo_eth_address).unwrap();
    let embryo_actor_id: ActorID = tester.create_embryo(&embryo_delegated_address,TokenAmount::from_whole(100)).unwrap();


    println!("Embryo address delegated type [{}]", embryo_delegated_address);
    println!("Embryo address delegated type on hex [{}]",hex::encode(embryo_delegated_address.to_bytes()));
    println!("Embryo address ID type on decimal [{}]",embryo_actor_id);
    println!("Embryo address ID type on hex [{}]",hex::encode(Address::new_id(embryo_actor_id).to_bytes()));

    println!("{}", format!("Sender address id [{}] and bytes [{}]", &sender[0].0, hex::encode(&sender[0].1.to_bytes())));
    println!("{}", format!("Sender address id [{}] and bytes [{}]", &sender[1].0, hex::encode(&sender[1].1.to_bytes())));
    println!("{}", format!("Sender address id [{}] and bytes [{}]", &sender[2].0, hex::encode(&sender[2].1.to_bytes())));
    println!("{}", format!("Sender address id [{}] and bytes [{}]", &sender[3].0, hex::encode(&sender[3].1.to_bytes())));

    // Governor address
    // https://github.com/Zondax/ref-fvm/blob/14fdd638fe29beaf4259a02a65a141b736fff17d/testing/integration/src/tester.rs#L84
    println!("Governor address ID type on hex [{}]",hex::encode(Address::new_id(200).to_bytes()));

    // Instantiate machine
    tester.instantiate_machine(DummyExterns).unwrap();

    let executor = tester.executor.as_mut().unwrap();


    // First we deploy the contract in order to actually have an actor running on the embryo address
    println!("Calling init actor (EVM)");

    let wasm_path = env::current_dir()
        .unwrap()
        .join(WASM_COMPILED_PATH)
        .canonicalize()
        .unwrap();
    let evm_hex = std::fs::read(wasm_path).expect("Unable to read file");
    let evm_bin = hex::decode(evm_hex).unwrap();

    let constructor_params = Create2Params {
        initcode: evm_bin,
        salt: [0; 32],
    };

    let message = Message {
        from: Address::new_id(embryo_actor_id),
        to: EAM_ACTOR_ADDR,
        gas_limit: 1000000000,
        method_num: 3,
        sequence: 0,
        params: RawBytes::serialize(constructor_params).unwrap(),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);

    let exec_return : Return = RawBytes::deserialize(&res.msg_receipt.return_data).unwrap();

    println!("Contract address ID type on decimal [{}]",exec_return.actor_id);
    println!("Contract address ID type on hex [{}]", hex::encode(Address::new_id(exec_return.actor_id).to_bytes()));
    println!("Contract address robust type [{}]",exec_return.robust_address);
    println!("Contract address eth address type [{}]",hex::encode(exec_return.eth_address.0));

    let contract_actor_id = exec_return.actor_id;

    // We need to mint tokens for the contract actor address in order to be able to execute methods like transfer, etc
    // NOTICE: The only address that can mint tokens is the governor, which is defined on the ref-fvm repo (on integration module)
    // NOTICE: We firt deploy the contract because the embryo address by its own cannot receive minted tokens.
    println!("Minting some tokens on datacap actor");

    let mint_params_1 = fil_actor_datacap::MintParams{
        to: Address::new_id(contract_actor_id),
        amount: TokenAmount::from_whole(1000),
        operators: vec![Address::new_id(sender[0].0),Address::new_id(sender[1].0)]
    };

    let message = Message {
        from: Address::new_id(200),
        to: DATACAP_TOKEN_ACTOR_ADDR,
        gas_limit: 1000000000,
        method_num: 116935346, // Coming from get_method_nums command
        sequence: 0,
        params: RawBytes::serialize(mint_params_1).unwrap(),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);


    println!("Minting more tokens on datacap actor");

    let mint_params_2 = fil_actor_datacap::MintParams{
        to: Address::new_id(sender[0].0),
        amount: TokenAmount::from_whole(1000),
        operators: vec![Address::new_id(contract_actor_id)]
    };

    let message = Message {
        from: Address::new_id(200),
        to: DATACAP_TOKEN_ACTOR_ADDR,
        gas_limit: 1000000000,
        method_num: 116935346, // Coming from get_method_nums command
        sequence: 1,
        params: RawBytes::serialize(mint_params_2).unwrap(),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);

    println!("Calling `name`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 0,
        params: RawBytes::new(hex::decode("4406FDDE03").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "5860000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000074461746143617000000000000000000000000000000000000000000000000000");


    println!("Calling `symbol`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 1,
        params: RawBytes::new(hex::decode("4495D89B41").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    //dbg!(&res);
    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "5860000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000044443415000000000000000000000000000000000000000000000000000000000");


    println!("Calling `total_supply`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 2,
        params: RawBytes::new(hex::decode("443940E9EE").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "582000000000000000000000000000000000000000000000006c6b935b8bbd400000");


    println!("Calling `balance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 3,
        params: RawBytes::new(hex::decode("58645363301D000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020066000000000000000000000000000000000000000000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58200000000000000000000000000000000000000000000000000000000000000000");


    println!("Calling `allowance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 4,
        params: RawBytes::new(hex::decode("58E4CE0A0B350000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000000000000000000000000000000000000000000000000000000000000000001501DCE5B7F69E73494891556A350F8CC357614916D50000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58200000000000000000000000000000000000000000000000000000000000000000");

    println!("Calling `transfer`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 5,
        params: RawBytes::new(hex::decode("58E4003B119F000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000001BC16D674EC8000000000000000000000000000000000000000000000000000000000000000000A0000000000000000000000000000000000000000000000000000000000000000300C80100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000361a08405e8fd800000000000000000000000000000000000000000000000000001bc16d674ec8000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000");

    println!("Calling `transfer_from`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 6,
        params: RawBytes::new(hex::decode("5901443EB577B10000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000C00000000000000000000000000000000000000000000000003782DACE9D90000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000000000000000000000000000000000000000000000000000000000000000000300C80100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58c00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000035fe46d2f74110000000000000000000000000000000000000000000000000000053444835ec58000000000000000000000000000000000002f050fe938943acc427e27bb16270000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000000");


    println!("Calling `burn`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 7,
        params: RawBytes::new(hex::decode("5824958FD4A00000000000000000000000000000000000000000000000000DE0B6B3A7640000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58200000000000000000000000000000000000000000000000360c2789aae8740000");


    println!("Calling `burn_from`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 8,
        params: RawBytes::new(hex::decode("58A45CF757EF000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000DE0B6B3A76400000000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "5840000000000000000000000000000000000000000000000035f0661c4399ac000000000000000000000000000000000002f050fe938943acc41a01c4fdbb0c0000");

    println!("Calling `allowance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 9,
        params: RawBytes::new(hex::decode("58E4CE0A0B350000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C000000000000000000000000000000000000000000000000000000000000000000000000000000000000030091030000000000000000000000000000000000000000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "582000000000000000000000000000000002f050fe938943acc41a01c4fdbb0c0000");


    println!("Calling `increase_allowance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 10,
        params: RawBytes::new(hex::decode("58A46BE03C810000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000003635C9ADC5DEA000000000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "582000000000000000000000000000000002f050fe938943acfa952f0445dea00000");



    println!("Calling `decrease_allowance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 11,
        params: RawBytes::new(hex::decode("58A46E7E2C520000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000003635C9ADC5DEA000000000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "582000000000000000000000000000000002f050fe938943acc45f65568000000000");



    println!("Calling `revoke_allowance`");

    let message = Message {
        from: sender[0].1,
        to: Address::new_id(contract_actor_id),
        gas_limit: 1000000000,
        method_num: 2,
        sequence: 12,
        params: RawBytes::new(hex::decode("588455E1C7A3000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000015011EDA43D05CA6D7D637E7065EF6B8C5DB89E5FB0C0000000000000000000000").unwrap()),
        ..Message::default()
    };

    let res = executor
        .execute_message(message, ApplyKind::Explicit, 100)
        .unwrap();

    assert_eq!(res.msg_receipt.exit_code.value(), 0);
    assert_eq!(hex::encode(res.msg_receipt.return_data.bytes()), "58200000000000000000000000000000000000000000000000000000000000000000");
}
