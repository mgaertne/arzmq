//! A [`Stream`] of the events a monitor socket receives.

use alloc::sync::Arc;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::stream::{FusedStream, Stream};

use super::{
    MonitorSocket, MonitorSocketEvent,
    monitor::Monitor,
    multipart_stream::{MultipartStream, OwnedMultipartStream},
};
use crate::{ZmqResult, message::MultipartMessage};

/// # Stream of the events received by a monitor socket
///
/// Created by [`MonitorSocket::monitor_event_stream()`] or
/// [`MonitorSocket::into_monitor_event_stream()`]. Events that cannot be decoded are yielded as
/// errors. The stream ends after the [`MonitorStopped`] event, which 0MQ sends last, or when the
/// context is terminated, so [`MonitorStopped`] has to be part of the monitored events to end the
/// stream before that.
///
/// [`MonitorStopped`]: MonitorSocketEvent::MonitorStopped
#[derive(Debug)]
pub struct MonitorEventStream<S> {
    messages: S,
    stopped: bool,
}

impl MonitorSocket {
    /// # Stream the events of this monitor socket
    ///
    /// See [`MonitorEventStream`].
    pub fn monitor_event_stream(&self) -> MonitorEventStream<MultipartStream<'_, Monitor>> {
        MonitorEventStream {
            messages: self.multipart_stream(),
            stopped: false,
        }
    }

    /// # Stream the events of a shared monitor socket
    ///
    /// Like [`monitor_event_stream()`], but keeps the socket alive itself.
    ///
    /// [`monitor_event_stream()`]: MonitorSocket::monitor_event_stream
    pub fn into_monitor_event_stream(
        self: Arc<Self>,
    ) -> MonitorEventStream<OwnedMultipartStream<Monitor>> {
        MonitorEventStream {
            messages: self.into_multipart_stream(),
            stopped: false,
        }
    }
}

impl<S> Stream for MonitorEventStream<S>
where
    S: Stream<Item = ZmqResult<MultipartMessage>> + Unpin,
{
    type Item = ZmqResult<MonitorSocketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.stopped {
            return Poll::Ready(None);
        }

        let item = match Pin::new(&mut self.messages).poll_next(ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(item) => item,
        };

        Poll::Ready(item.map(|message| {
            let event = message.and_then(MonitorSocketEvent::try_from);
            self.stopped = matches!(event, Ok(MonitorSocketEvent::MonitorStopped));
            event
        }))
    }
}

impl<S> FusedStream for MonitorEventStream<S>
where
    S: FusedStream + Stream<Item = ZmqResult<MultipartMessage>> + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.stopped || self.messages.is_terminated()
    }
}

#[cfg(test)]
mod monitor_stream_tests {
    use alloc::sync::Arc;

    use futures::{StreamExt, executor::block_on, stream::FusedStream};

    use crate::prelude::{Context, MonitorFlags, MonitorSocketEvent, PairSocket, ZmqResult};

    #[test]
    fn stream_yields_events_until_monitor_stops() -> ZmqResult<()> {
        let context = Context::new()?;
        let monitored = PairSocket::from_context(&context)?;
        let monitor =
            Arc::new(monitored.monitor(MonitorFlags::Listening | MonitorFlags::MonitorStopped)?);
        monitored.bind("tcp://127.0.0.1:*")?;
        drop(monitored);

        let mut stream = monitor.into_monitor_event_stream();
        let events = block_on(stream.by_ref().collect::<Vec<_>>());

        assert_eq!(
            events,
            vec![
                Ok(MonitorSocketEvent::Listening),
                Ok(MonitorSocketEvent::MonitorStopped)
            ]
        );
        assert!(stream.is_terminated());
        Ok(())
    }

    #[test]
    fn borrowed_stream_yields_event_of_monitored_socket() -> ZmqResult<()> {
        let context = Context::new()?;
        let monitored = PairSocket::from_context(&context)?;
        let monitor = monitored.monitor(MonitorFlags::Listening)?;
        monitored.bind("tcp://127.0.0.1:*")?;

        let event = block_on(monitor.monitor_event_stream().next());

        assert_eq!(event, Some(Ok(MonitorSocketEvent::Listening)));
        Ok(())
    }
}
