mod github;

fn main() {
    let notifications = github::notifications();

    println!("{:?}", notifications);

    for notification in notifications {
        let notification = notification.read();
        println!("READ: {:?}", notification);
        let _ = notification.unsubscribe();
    }
}
