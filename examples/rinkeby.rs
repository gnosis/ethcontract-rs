use ethcontract::transaction::Account;
use ethsign::SecretKey;
use std::env;
use std::time::Duration;
use web3::api::Web3;
use web3::transports::Http;
use web3::types::H256;

ethcontract::contract!("examples/truffle/build/contracts/DeployedContract.json");

const RINKEBY_CHAIN_ID: u64 = 4;

fn main() {
    futures::executor::block_on(run());
}

async fn run() {
    let account = {
        let pk = env::var("PK").expect("PK is not set");
        let raw_key: H256 = pk.parse().expect("invalid PK");
        let key = SecretKey::from_raw(&raw_key[..]).expect("invalid PK");
        Account::Offline(key, Some(RINKEBY_CHAIN_ID))
    };
    let infura_url = {
        let project_id = env::var("INFURA_PROJECT_ID").expect("INFURA_PROJECT_ID is not set");
        format!("https://rinkeby.infura.io/v3/{}", project_id)
    };

    let (eloop, http) = Http::new(&infura_url).expect("transport");
    eloop.into_remote();
    let web3 = Web3::new(http);

    let instance = DeployedContract::deployed(&web3).await.expect("deployed");

    println!(
        "value before: {}",
        instance
            .value()
            .from(account.address())
            .call()
            .await
            .expect("value")
    );
    instance
        .increment()
        .from(account.clone())
        //.send_and_confirm(Duration::new(5, 0), 1)
        .send()
        .await
        .expect("increment");
    println!(
        "value after: {}",
        instance
            .value()
            .from(account.address())
            .call()
            .await
            .expect("value")
    );
}
