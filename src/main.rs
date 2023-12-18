use blockchain_rust::Blockchain;
use blockchain_rust::Cli;

fn main() {
    let blockchain = Blockchain::new_blockchain();
    let mut cli = Cli::new_cli(blockchain);
    cli.run();
}
