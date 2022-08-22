use std::sync::mpsc::channel;

#[monoio::main]
async fn main() {
    let msg = "hello world!";
    let (tx, rx) = channel();
    let _ = tx.send(msg);
    monoio::spawn(async move {
        let msg = rx.recv();
        if let Ok(msg) = msg {
            println!("{}", msg);
        } else {
            eprintln!("error!")
        }
    })
    .await;
}
