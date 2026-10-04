pub trait Server {
    type ConnState: Default;
    type AppState;
}
