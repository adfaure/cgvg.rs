use rgvg::common::load;

#[tokio::main]
async fn main() {
    env_logger::init();

    let data_file = "file.bin";
    let index_file = "index.bin";

    let result = load(1, data_file, index_file);
    println!("retrieved tuple: {result:?}");

    // Ensure the command completes
    let status = cmd.wait().await.expect(""); //cmd.await.expect("Command wasn't running");
    debug!("Command finished with status: {}", status);
}
