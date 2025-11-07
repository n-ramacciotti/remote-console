/// Remote console application entry point.
#[tokio::main]
async fn main() {
    if let Err(e) = remote_console::run().await {
        println!("Remote console failed: {e}");
    }
}
