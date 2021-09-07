mod github;

#[tokio::main]
async fn main() {
    let notifications = github::notifications().await;

    println!("{:?}", notifications);

    for notification in notifications {
        let notification = notification.read().await;
        println!("READ: {:?}", notification);
        let _ = notification.unsubscribe().await;
    }
}
