use std::io::{self, ErrorKind};
use std::os::unix::net::{UnixListener, UnixStream};

use smithay_client_toolkit::reexports::calloop::generic::Generic;
use smithay_client_toolkit::reexports::calloop::{
    self, EventSource, Interest, Mode, Poll, PostAction, Readiness, Token, TokenFactory,
};

#[derive(Debug)]
pub struct SocketSource {
    socket: Generic<UnixListener>,
}

impl SocketSource {
    pub fn new(socket: UnixListener) -> calloop::Result<Self> {
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket: Generic::new(socket, Interest::READ, Mode::Level),
        })
    }
}

impl EventSource for SocketSource {
    type Error = io::Error;
    type Event = UnixStream;
    type Metadata = ();
    type Ret = ();

    fn process_events<F>(
        &mut self,
        readiness: Readiness,
        token: Token,
        mut callback: F,
    ) -> io::Result<PostAction>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        self.socket.process_events(readiness, token, |_, socket| {
            let accept_next = || match socket.accept() {
                Ok((stream, _)) => Ok(Some(stream)),
                Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
                Err(err) => Err(err),
            };

            while let Some(stream) = accept_next()? {
                callback(stream, &mut ());
            }

            Ok(PostAction::Continue)
        })
    }

    fn register(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        self.socket.register(poll, token_factory)
    }

    fn reregister(
        &mut self,
        poll: &mut Poll,
        token_factory: &mut TokenFactory,
    ) -> calloop::Result<()> {
        self.socket.reregister(poll, token_factory)
    }

    fn unregister(&mut self, poll: &mut Poll) -> calloop::Result<()> {
        self.socket.unregister(poll)
    }
}
