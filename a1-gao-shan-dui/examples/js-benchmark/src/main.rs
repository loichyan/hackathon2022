use akun::*;
use rand::prelude::*;
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};
use wasm_bindgen::JsValue;
use web_sys::console;

static ADJECTIVES: &[&str] = &[
    "pretty",
    "large",
    "big",
    "small",
    "tall",
    "short",
    "long",
    "handsome",
    "plain",
    "quaint",
    "clean",
    "elegant",
    "easy",
    "angry",
    "crazy",
    "helpful",
    "mushy",
    "odd",
    "unsightly",
    "adorable",
    "important",
    "inexpensive",
    "cheap",
    "expensive",
    "fancy",
];

static COLOURS: &[&str] = &[
    "red", "yellow", "blue", "green", "pink", "brown", "purple", "brown", "white", "black",
    "orange",
];

static NOUNS: &[&str] = &[
    "table", "chair", "house", "bbq", "desk", "car", "pony", "cookie", "sandwich", "burger",
    "pizza", "mouse", "keyboard",
];

struct Button<N> {
    cx: Scope,
    id: Option<&'static str>,
    text: Option<&'static str>,
    on_click: Option<Box<dyn FnMut()>>,
    children: PhantomData<N>,
}

impl<N: GenericNode> Button<N> {
    pub fn build(self) -> impl GenericComponent<N> {
        let cx = self.cx;
        let id = self.id.unwrap();
        let text = self.text.unwrap();
        let mut on_click = self.on_click.unwrap();
        view! { cx,
            div {
                :className("col-sm-6 smallpad")
                button {
                    :className("btn btn-primary btn-block")
                    :type("button")
                    :id(id)
                    @click(move |_| on_click())
                    (text)
                }
            }
        }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn text(mut self, text: &'static str) -> Self {
        self.text = Some(text);
        self
    }

    pub fn on_click(mut self, on_click: impl 'static + FnMut()) -> Self {
        self.on_click = Some(Box::new(on_click));
        self
    }
}

#[allow(non_snake_case)]
fn Button<N: GenericNode>(cx: Scope) -> Button<N> {
    Button {
        cx,
        id: None,
        text: None,
        on_click: None,
        children: PhantomData,
    }
}

#[derive(Clone)]
struct RowData {
    id: usize,
    label: Signal<String>,
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn build_data(cx: Scope, count: usize) -> Vec<RowData> {
    let mut thread_rng = thread_rng();

    let mut data = Vec::new();
    data.reserve_exact(count);

    for _i in 0..count {
        let adjective = ADJECTIVES.choose(&mut thread_rng).unwrap();
        let colour = COLOURS.choose(&mut thread_rng).unwrap();
        let noun = NOUNS.choose(&mut thread_rng).unwrap();
        let capacity = adjective.len() + colour.len() + noun.len() + 2;
        let mut label = String::with_capacity(capacity);
        label.push_str(adjective);
        label.push(' ');
        label.push_str(colour);
        label.push(' ');
        label.push_str(noun);

        data.push(RowData {
            id: ID_COUNTER.load(Ordering::Relaxed),
            label: cx.create_signal(label),
        });

        ID_COUNTER.store(ID_COUNTER.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    }

    data
}

struct App<N> {
    cx: Scope,
    children: PhantomData<N>,
}

impl<N: GenericNode> App<N> {
    pub fn build(self) -> impl GenericComponent<N> {
        let Self { cx, .. } = self;

        let data = cx.create_signal(Vec::<RowData>::new());
        let selected = cx.create_signal(None::<usize>);

        cx.create_effect(move || {
            let arr = data.read(|data| {
                data.iter()
                    .map(|t| JsValue::from_f64(t.id as f64))
                    .collect::<js_sys::Array>()
            });
            console::log_1(&arr);
        });

        let remove = move |id| {
            data.write(move |data| data.retain(|row| row.id != id));
        };

        let run = move || {
            data.set(build_data(cx, 1000));
            selected.set(None);
        };

        let runlots = move || {
            data.set(build_data(cx, 10000));
            selected.set(None);
        };

        let add = move || {
            data.write(move |data| data.append(&mut build_data(cx, 1000)));
        };

        let update = move || {
            data.write(|data| {
                for row in data.iter().step_by(10) {
                    row.label.write(|n| n.push_str(" !!!"));
                }
            });
        };

        let clear = move || {
            data.set(Vec::new());
            selected.set(None);
        };

        let swaprows = move || {
            data.write(|data| {
                if data.len() > 998 {
                    data.swap(1, 998);
                }
            });
        };

        view! { cx,
            div {
                :className("container")
                div {
                    :className("jumbotron")
                    div {
                        :className("row")
                        div { :className("col-md-6") h1 { "aKun Keyed" } }
                        div {
                            :className("col-md-6")
                            div {
                                :className("row")
                                *Button { .id("run") .text("Create 1,000 rows") .on_click(run) }
                                *Button { .id("runlots") .text("Create 10,000 rows") .on_click(runlots) }
                                *Button { .id("add") .text("Append 1,000 rows") .on_click(add) }
                                *Button { .id("update") .text("Update every 10th row") .on_click(update) }
                                *Button { .id("clear") .text("Clear") .on_click(clear) }
                                *Button { .id("swaprows") .text("Swap Rows") .on_click(swaprows) }
                            }
                        }
                    }
                }
                table {
                    :className("table table-hover table-striped test-data")
                    tbody {
                        *For {
                            .each(data)
                            .key(|data| data.id)
                            {move |cx, row| {
                                let row_id = row.id;
                                let label = row.label;
                                let is_selected = cx.create_selector(move || selected.get() == Some(row_id));
                                let set_selected = move |_| selected.set(Some(row_id));
                                view! { cx,
                                    tr {
                                        .toggle_class("danger", is_selected)
                                        td { :className("col-md-1") (row_id) }
                                        td { :className("col-md-4") a { @click(set_selected) (label) } }
                                        td {
                                            :className("col-md-1")
                                            a {
                                                @click(move |_| remove(row_id))
                                                span {
                                                    :className("glyphicon glyphicon-remove")
                                                    :ariaHidden(true)
                                                }
                                            }
                                        }
                                        td { :className("col-md-6") }
                                    }
                                }
                            }}
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn App<N: GenericNode>(cx: Scope) -> App<N> {
    App {
        cx,
        children: PhantomData,
    }
}

fn main() {
    console_error_panic_hook::set_once();

    akun::mount_to_body(|cx| view! { cx, *App { }})
}
