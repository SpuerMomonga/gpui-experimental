use std::{cell::RefCell, rc::Rc};

use gpui::{
    Action, AnyEntity, AnyWindowHandle, App, Context, Entity, IntoElement, SharedString, Window,
    WindowOptions,
};

slotmap::new_key_type! {
    /// A logical identifier for a mounted view.
    pub struct ViewId;
}

pub struct ViewConfig {
    pub title: SharedString,
    pub default: bool,
}

pub struct DetachableConfig {
    pub title: SharedString,
    pub window: WindowOptions,
}

pub enum ViewOptions {
    View(ViewConfig),
    Detachable(DetachableConfig),
    Window(WindowOptions),
}

pub struct ViewAction {
    pub title: SharedString,
    pub icon: Option<SharedString>,
    action: Box<dyn Action>,
}

impl ViewAction {
    pub fn new<A>(title: impl Into<SharedString>, action: A) -> Self
    where
        A: Action,
    {
        Self {
            title: title.into(),
            icon: None,
            action: Box::new(action),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn action(&self) -> &dyn Action {
        self.action.as_ref()
    }
}

pub trait ViewDelegate: 'static + Sized {
    fn render_view(
        &mut self,
        view: &mut View,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement;

    fn placeholder_text(
        &self,
        _view: &mut View,
        _window: &Window,
        _cx: &App,
    ) -> Option<SharedString> {
        None
    }

    fn render_search_control(
        &mut self,
        _view: &mut View,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        None::<gpui::Div>
    }

    fn perform_search(
        &mut self,
        _view: &mut View,
        _query: &str,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn actions(
        &mut self,
        _view: &mut View,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Vec<ViewAction> {
        Vec::new()
    }

    fn primary_action(
        &mut self,
        _view: &mut View,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<ViewAction> {
        None
    }
}

struct ViewState {
    id: ViewId,
    remove_requested: bool,
    mount_hooks: Vec<Box<dyn FnOnce(&mut App)>>,
    unmount_hooks: Vec<Box<dyn FnOnce(&mut App)>>,
}

/// Controller for one logical view mount.
///
/// Clones refer to the same lifecycle state, so a view can keep this handle in
/// its entity while the store keeps another clone for lifecycle dispatch.
#[derive(Clone)]
pub struct View(Rc<RefCell<ViewState>>);

impl View {
    pub(crate) fn new(id: ViewId) -> Self {
        Self(Rc::new(RefCell::new(ViewState {
            id,
            remove_requested: false,
            mount_hooks: Vec::new(),
            unmount_hooks: Vec::new(),
        })))
    }

    pub fn id(&self) -> ViewId {
        self.0.borrow().id
    }

    /// Register work to run when this mount becomes visible.
    ///
    /// Register this during the `open_view` build closure. The callback receives the
    /// application context that performs the mount.
    pub fn on_mount(&self, callback: impl FnOnce(&mut App) + 'static) {
        self.0.borrow_mut().mount_hooks.push(Box::new(callback));
    }

    /// Register work to run when this mount is removed from the store.
    ///
    /// The callback receives the application context that performs the unmount.
    pub fn on_unmount(&self, callback: impl FnOnce(&mut App) + 'static) {
        self.0.borrow_mut().unmount_hooks.push(Box::new(callback));
    }

    /// Request removal after the current GPUI operation returns.
    pub fn remove_view(&self) {
        self.0.borrow_mut().remove_requested = true;
    }

    pub(crate) fn take_remove_request(&self) -> bool {
        let mut state = self.0.borrow_mut();
        std::mem::take(&mut state.remove_requested)
    }

    pub(crate) fn run_mount(&self, cx: &mut App) {
        let callbacks = {
            let mut state = self.0.borrow_mut();
            std::mem::take(&mut state.mount_hooks)
        };
        for callback in callbacks {
            callback(cx);
        }
    }

    pub(crate) fn run_unmount(&self, cx: &mut App) {
        let callbacks = {
            let mut state = self.0.borrow_mut();
            std::mem::take(&mut state.unmount_hooks)
        };
        for callback in callbacks {
            callback(cx);
        }
    }
}

pub struct ViewHandle<V> {
    pub id: ViewId,
    pub view: View,
    pub entity: Entity<V>,
    pub window: Option<AnyWindowHandle>,
}

pub struct ViewEntry {
    pub view: View,
    pub entity: AnyEntity,
    pub window: Option<AnyWindowHandle>,
    pub options: ViewOptions,
}

pub enum ViewEvent {
    Mounted { id: ViewId },
    Changed { id: ViewId },
    Unmounted { id: ViewId },
}

#[derive(Clone)]
pub struct AnyViewHandle {
    pub id: ViewId,
    pub view: View,
    pub entity: AnyEntity,
    pub window: Option<AnyWindowHandle>,
}

impl<V: 'static> From<&ViewHandle<V>> for AnyViewHandle {
    fn from(handle: &ViewHandle<V>) -> Self {
        Self {
            id: handle.id,
            view: handle.view.clone(),
            entity: handle.entity.clone().into(),
            window: handle.window.clone(),
        }
    }
}
