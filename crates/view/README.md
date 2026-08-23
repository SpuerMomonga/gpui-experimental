## API

```rust

pub trait ViewContext {
    fn open_view<V: 'static + ViewDelegate>(
        options: ViewOptions, 
        view: impl FnOnce(&mut Window, &mut App) -> Entity<V>
    ) -> anyhow::Result<ViewHandle<V>>;
}

impl ViewContext for App {
    fn open_view<V: 'static + ViewDelegate>(
        options: ViewOptions, 
        view: impl FnOnce(&mut Window, &mut App) -> Entity<V>
    ) -> anyhow::Result<ViewHandle<V>> {
        todo!()
    }
}

```

## Examples

```rust

pub fn init(cx: &mut App) {
    ViewStore::global(cx).update(cx, |view, cx| {
        view.open_view(|cx| cx.new(|_cx| ClipboardViewDelegate::new()), cx);
    })
}

pub struct ClipboardViewDelegate;

impl ViewDelegate for ClipboardViewDelegate {
    fn placeholder_text(&self, _window: &mut Window, _cx: &mut App) -> Arc<str> {
        "Search a clipboard...".into()
    }

    fn view_id(&self) -> Arc<str> {
        "clipboard.view".into()
    }

    fn actions(&mut self, _window: &mut Window, _cx: &mut App) -> Vec<Action> {
        todo!()
    }

    fn render_view(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

```
