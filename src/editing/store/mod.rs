//! # Intra-buffer communication
//!
//! ## Overview
//!
//! These components support sharing information between multiple buffers, such as
//! [registers](RegisterStore). Most of the time, you will only need to create a [SharedStore]
//! which contains all of the common components.
//!
//! ## Example
//!
//! ```
//! use modalkit::{
//!     editing::store::{SharedStore, Store},
//!     env::vim::VimContext,
//! };
//!
//! fn main() {
//!     let store: SharedStore<VimContext, ()> = Store::new();
//!     let locked = store.try_read().unwrap();
//!
//!     assert_eq!(locked.digraphs.get(('>', '>')), Some('\u{00BB}'));
//! }
//! ```
use std::sync::{Arc, RwLock};

use crate::editing::{
    base::{Application, EditContext},
    history::HistoryList,
    rope::EditRope,
};

mod buffer;
mod cursor;
mod digraph;
mod register;

pub use self::buffer::{BufferId, BufferStore, SharedBuffer};
pub use self::cursor::{CursorStore, MarkStore};
pub use self::digraph::DigraphStore;
pub use self::register::{RegisterCell, RegisterStore};

const SEARCH_HISTORY_LEN: usize = 50;

/// Global editing context
pub struct Store<C: EditContext, P: Application> {
    /// Tracks what [buffers](crate::editing::buffer::EditBuffer) have been created.
    pub buffers: BufferStore<C, P>,

    /// Tracks mapped digraphs.
    pub digraphs: DigraphStore,

    /// Tracks the current value of each [Register](crate::editing::base::Register).
    pub registers: RegisterStore,

    /// Tracks the buffer- and global-specific information of each
    /// [Mark](crate::editing::base::Mark).
    pub marks: MarkStore,

    /// Tracks previous search expressions.
    pub searches: HistoryList<EditRope>,

    /// Application-specific storage.
    pub application: P::Store,
}

/// Shared reference to the global context.
pub type SharedStore<C, P> = Arc<RwLock<Store<C, P>>>;

impl<C, P> Store<C, P>
where
    C: EditContext,
    P: Application,
{
    /// Create a new global store for an application to use.
    pub fn new() -> SharedStore<C, P> {
        let store = Store {
            buffers: BufferStore::new(),
            digraphs: DigraphStore::default(),
            registers: RegisterStore::default(),
            marks: MarkStore::default(),
            searches: HistoryList::new("".into(), SEARCH_HISTORY_LEN),

            application: P::Store::default(),
        };

        return Arc::new(RwLock::new(store));
    }

    /// Create a new shared buffer.
    pub fn new_buffer(store: &SharedStore<C, P>) -> SharedBuffer<C, P> {
        let clone = store.clone();

        store.write().unwrap().buffers.new_buffer(clone)
    }

    /// Get a buffer via its identifier.
    pub fn get_buffer(id: BufferId, store: &SharedStore<C, P>) -> SharedBuffer<C, P> {
        store.read().unwrap().buffers.get_buffer(&id)
    }

    /// Add a search query to the search history, and set [Register::LastSearch].
    ///
    /// [Register::LastSearch]: super::base::Register::LastSearch
    pub fn set_last_search<T: Into<EditRope>>(text: T, store: &SharedStore<C, P>) {
        let rope = text.into();

        if rope.len() == 0 {
            // Disallow empty searches.
            return;
        }

        let mut locked = store.write().unwrap();
        locked.searches.select(rope.clone());
        locked.registers.set_last_search(rope);
    }
}
