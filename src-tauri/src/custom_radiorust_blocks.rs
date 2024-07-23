pub mod custom_radiorust_blocks {
    use radiorust::{
        flow::{new_receiver, new_sender, Message, ReceiverConnector, SenderConnector},
        impl_block_trait,
    };
    use tokio::spawn;

    pub struct AmDemod<T> {
        receiver_connector: ReceiverConnector<T>,
        sender_connector: SenderConnector<T>,
    }

    impl_block_trait! { <T> Consumer<T> for AmDemod<T> }
    impl_block_trait! { <T> Producer<T> for AmDemod<T> }

    // this is just a NOP block from the docs for now
    impl<T> AmDemod<T>
    where
        T: Message + Send + 'static,
    {
        /// Creates a block which does nothing but pass data through
        pub fn new() -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<T>();
            let (sender, sender_connector) = new_sender::<T>();
            spawn(async move {
                loop {
                    let Ok(msg) = receiver.recv().await else {
                        return;
                    };
                    let Ok(()) = sender.send(msg).await else {
                        return;
                    };
                }
            });
            Self {
                receiver_connector,
                sender_connector,
            }
        }
    }
}
