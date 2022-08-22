use std::error::Error;

use bytes::Bytes;
use h2::{server::SendResponse, RecvStream};
use http::Request;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("127.0.0.1:5000").await?;
    loop {
        let (stream, _addr) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            match handle_h2(stream).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error: {}", e)
                }
            }
        });
    }
}

async fn handle_h2(stream: TcpStream) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = h2::server::Builder::new().handshake(stream).await?;
    match conn.accept().await {
        Some(acc) => {
            let acc = acc?;
            let _ = tokio::spawn(async {
                match handle_h2_stream(acc.0, acc.1).await {
                    Ok(_) => {}
                    Err(err) => eprintln!("{}", err),
                }
            });
        }
        None => todo!(),
    }
    Ok(())
}

async fn handle_h2_stream(
    _request: Request<RecvStream>,
    mut response: SendResponse<Bytes>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("start handle h2 stream");
    // send h2 response
    // let body = request.body_mut();
    // while let Some(data) = body.data().await {
    //     let data = data?;
    //     // release more flow control
    //     body.flow_control().release_capacity(data.len())?;
    // }
    println!("creating response");
    // create response
    let resp = http::Response::new(());
    let mut send_stream = response.send_response(resp, false)?;
    send_stream.send_data(Bytes::from_static(b"hello, "), false)?;
    send_stream.send_data(Bytes::from_static(b"this is kingtous h2 server, "), false)?;
    send_stream.send_data(Bytes::from_static(b"hope you enjoy!"), true)?;
    // sent
    println!("hello msg sent");
    Ok(())
}
