use gpui::{App, AppContext as _, Entity, EventEmitter, Global};
use slotmap::SlotMap;

mod view;

pub use view::*;

pub fn init(cx: &mut App) {
    let register = cx.new(|_cx| ViewStore::new());
    cx.set_global(GlobalViewStore(register));
}

pub struct ViewStore {
    views: SlotMap<ViewId, ViewEntry>,
    main_view: Option<ViewId>,
    default_view: Option<ViewId>,
}

impl ViewStore {
    fn new() -> Self {
        Self {
            views: Default::default(),
            main_view: Default::default(),
            default_view: Default::default(),
        }
    }

    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalViewStore>().0.clone()
    }
}

struct GlobalViewStore(Entity<ViewStore>);

impl Global for GlobalViewStore {}

impl EventEmitter<ViewEvent> for ViewStore {}

/// Opens a logical view and gives its builder the lifecycle controller for that mount.
///
/// The same controller is threaded through ordinary `ViewDelegate` callbacks as `&mut View`;
/// the delegate does not need separate lifecycle methods.
pub trait ViewContext {
    fn open_view<V: ViewDelegate>(
        &mut self,
        options: ViewOptions,
        build: impl FnOnce(&mut App, View) -> Entity<V>,
    ) -> anyhow::Result<ViewHandle<V>>;
}
