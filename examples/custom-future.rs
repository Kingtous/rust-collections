use std::{future::Future, task::Poll, time::Duration};

use pin_project_lite::pin_project;
use tokio::time::{sleep_until, Instant, Sleep};

pin_project! {
    pub struct Timeout{
        #[pin]
        sleep: Sleep
    }
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            sleep: sleep_until(Instant::now() + Duration::from_millis(000)),
        }
    }
}

impl Future for Timeout {
    type Output = String;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        println!("poll!");
        let me = self.project();
        match me.sleep.poll(cx) {
            Poll::Ready(_) => Poll::Ready("reached timeout!".to_string()),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[tokio::main]
pub async fn main() {
    let timeout = Timeout::default();
    let resp = timeout.await;
    println!("{}", &resp);
}
