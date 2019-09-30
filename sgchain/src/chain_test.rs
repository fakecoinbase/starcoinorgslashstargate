use types::account_address::AccountAddress;
use std::thread::sleep;
use std::time::Duration;
use crate::star_chain_client::{MockChainClient, ChainClient, faucet_sync, stop_mock_chain};

#[test]
fn test_mock_chain_client_faucet() {
    ::logger::init_for_e2e_testing();
    let (client, _handle) = MockChainClient::new();
    for _i in 1..2 {
        let addr = AccountAddress::random();
        faucet_sync(client.clone(), addr, 1000);
        faucet_sync(client.clone(), addr, 1000);
        faucet_sync(client.clone(), addr, 1000);
        assert_eq!(client.account_exist(&addr, None), true);
    }
    stop_mock_chain(&client);
    drop(client);
}