use crate::keys::KeysResponse;

#[derive(Clone, Copy)]
pub struct TickContext {
    pub tick_count: u32,
    pub keys: KeysResponse,
}
