pub mod custom_radiorust_blocks {
    use radiorust::{
        flow::{new_receiver, new_sender, Message, ReceiverConnector, SenderConnector},
        impl_block_trait,
        numbers::Float,
        prelude::Complex,
        signal::Signal,
    };
    use tokio::spawn;

    pub struct AmDemod<Flt> {
        receiver_connector: ReceiverConnector<Signal<Complex<Flt>>>,
        sender_connector: SenderConnector<Signal<Complex<Flt>>>,
    }

    impl_block_trait! { <Flt> Consumer<Signal<Complex<Flt>>> for AmDemod<Flt> }
    impl_block_trait! { <Flt> Producer<Signal<Complex<Flt>>> for AmDemod<Flt> }

    impl<Flt> AmDemod<Flt>
    where
        Flt: Float,
    {
        pub fn new() -> Self {
            let (mut receiver, receiver_connector) = new_receiver::<Signal<Complex<Flt>>>();
            let (sender, sender_connector) = new_sender::<Signal<Complex<Flt>>>();
            spawn(async move {
                loop {
                    let Ok(signal) = receiver.recv().await else {
                        return;
                    };
                    match signal {
                        Signal::Samples {
                            sample_rate,
                            chunk: input_chunk,
                        } => {
                            // pass through signal
                            let Ok(()) = sender
                                .send(Signal::Samples {
                                    sample_rate,
                                    chunk: input_chunk,
                                })
                                .await
                            else {
                                return;
                            };
                        }
                        Signal::Event(event) => {
                            let Ok(()) = sender.send(Signal::Event(event)).await else {
                                return;
                            };
                        }
                    }
                }
            });
            Self {
                receiver_connector,
                sender_connector,
            }
        }
    }
}
