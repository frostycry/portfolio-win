use gloo_timers::callback::Interval;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Kind { About, Projects, Notepad, Calc }

const ALL: [Kind; 4] = [Kind::About, Kind::Projects, Kind::Notepad, Kind::Calc];

impl Kind {
    fn title(self) -> &'static str {
        match self { Kind::About => "About Me", Kind::Projects => "My Projects", Kind::Notepad => "Untitled - Notepad", Kind::Calc => "Calculator" }
    }
    fn icon(self) -> &'static str {
        match self { Kind::About => "👤", Kind::Projects => "📁", Kind::Notepad => "📝", Kind::Calc => "🧮" }
    }
}

#[derive(Clone, PartialEq)]
struct Win { id: u32, kind: Kind, x: i32, y: i32, z: u32, min: bool }

#[derive(Clone, PartialEq, Default)]
struct Desk { wins: Vec<Win>, next: u32, drag: Option<(u32, i32, i32)>, menu: bool }

impl Desk {
    fn open(&mut self, kind: Kind) {
        self.next += 1;
        let n = self.next;
        let o = (n % 6) as i32 * 28;
        self.wins.push(Win { id: n, kind, x: 60 + o, y: 30 + o, z: n, min: false });
        self.menu = false;
    }
    fn focus(&mut self, id: u32) {
        self.next += 1;
        let n = self.next;
        if let Some(w) = self.wins.iter_mut().find(|w| w.id == id) { w.z = n; w.min = false; }
    }
    fn toggle(&mut self, id: u32) {
        let top = self.wins.iter().filter(|w| !w.min).map(|w| w.z).max();
        let active = self.wins.iter().find(|w| w.id == id).map(|w| !w.min && Some(w.z) == top).unwrap_or(false);
        if active {
            if let Some(w) = self.wins.iter_mut().find(|w| w.id == id) { w.min = true; }
        } else { self.focus(id); }
    }
}

fn cb<E: 'static>(d: &UseStateHandle<Desk>, f: impl Fn(&mut Desk, E) + 'static) -> Callback<E> {
    let d = d.clone();
    Callback::from(move |e| { let mut s = (*d).clone(); f(&mut s, e); d.set(s); })
}

fn now() -> js_sys::Date { js_sys::Date::new_0() }
fn hms(d: &js_sys::Date) -> String { format!("{:02}:{:02}:{:02}", d.get_hours(), d.get_minutes(), d.get_seconds()) }

#[function_component(Clock)]
fn clock() -> Html {
    let t = use_state(|| hms(&now()));
    { let t = t.clone(); use_effect_with((), move |_| { let i = Interval::new(1000, move || t.set(hms(&now()))); move || drop(i) }); }
    html! { <span>{ (*t).clone() }</span> }
}

#[function_component(ClockWidget)]
fn clock_widget() -> Html {
    let t = use_state(|| hms(&now()));
    { let t = t.clone(); use_effect_with((), move |_| { let i = Interval::new(1000, move || t.set(hms(&now()))); move || drop(i) }); }
    let d = now();
    let date = format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date());
    html! { <div class="widget raised"><b>{"Clock"}</b><div class="big">{ (*t).clone() }</div><div>{ date }</div></div> }
}

#[function_component(Monitor)]
fn monitor() -> Html {
    let v = use_state(|| vec![30u32; 10]);
    { let v = v.clone(); use_effect_with((), move |_| { let i = Interval::new(700, move || v.set((0..10).map(|_| (js_sys::Math::random() * 100.0) as u32 + 5).collect())); move || drop(i) }); }
    html! { <div class="widget raised"><b>{"CPU Usage"}</b><div class="bars">{ for v.iter().map(|h| html!{ <i style={format!("height:{}%", h.min(&100))}></i> }) }</div></div> }
}

#[derive(Clone, PartialEq)]
struct Calc { disp: String, acc: f64, op: Option<char>, fresh: bool }

fn apply(o: char, a: f64, b: f64) -> f64 {
    match o { '+' => a + b, '-' => a - b, '*' => a * b, _ => if b == 0.0 { f64::NAN } else { a / b } }
}

fn press(c: &mut Calc, k: &str) {
    match k {
        "C" => *c = Calc { disp: "0".into(), acc: 0.0, op: None, fresh: false },
        "+" | "-" | "*" | "/" | "=" => {
            let v: f64 = c.disp.parse().unwrap_or(0.0);
            if let Some(o) = c.op { if !c.fresh { c.acc = apply(o, c.acc, v); } } else { c.acc = v; }
            c.disp = if c.acc.is_nan() { "Error".into() } else { format!("{}", (c.acc * 1e9).round() / 1e9) };
            c.op = if k == "=" { None } else { k.chars().next() };
            c.fresh = true;
        }
        _ => {
            if c.fresh || (c.disp == "0" && k != ".") {
                c.disp = if k == "." { "0.".into() } else { k.into() };
                c.fresh = false;
            } else if !(k == "." && c.disp.contains('.')) { c.disp.push_str(k); }
        }
    }
}

#[function_component(Calculator)]
fn calculator() -> Html {
    let c = use_state(|| Calc { disp: "0".into(), acc: 0.0, op: None, fresh: false });
    let keys = ["7","8","9","/","4","5","6","*","1","2","3","-","0",".","=","+"];
    let btn = |k: &'static str| {
        let c = c.clone();
        let onclick = Callback::from(move |_: MouseEvent| { let mut s = (*c).clone(); press(&mut s, k); c.set(s); });
        html! { <button {onclick}>{ k }</button> }
    };
    html! { <div><div class="disp">{ c.disp.clone() }</div><div class="keys">{ for keys.iter().map(|k| btn(k)) }{ btn("C") }</div></div> }
}

fn body(k: Kind) -> Html {
    match k {
        Kind::Notepad => html! { <textarea class="pad" placeholder="Type something..."></textarea> },
        Kind::Calc => html! { <Calculator /> },
        Kind::About => html! { <div>
            <h3>{"Hi, I'm YOUR NAME 👋"}</h3>
            <p>{"Replace this text with your bio. Double-click icons on the desktop to open programs."}</p>
            <p>{"📧 you@example.com"}</p></div> },
        Kind::Projects => html! { <ul>
            <li>{"📄 Project One - short description"}</li>
            <li>{"📄 Project Two - short description"}</li>
            <li>{"📄 Project Three - short description"}</li></ul> },
    }
}

#[function_component(App)]
fn app() -> Html {
    let d = use_state(Desk::default);
    let top = d.wins.iter().filter(|w| !w.min).map(|w| w.z).max();
    let stop = Callback::from(|e: MouseEvent| e.stop_propagation());

    html! {
      <div id="root"
        onmousemove={cb(&d, |s, e: MouseEvent| {
            if let Some((id, ox, oy)) = s.drag {
                if let Some(w) = s.wins.iter_mut().find(|w| w.id == id) { w.x = (e.client_x() - ox).max(0); w.y = (e.client_y() - oy).max(0); }
            }
        })}
        onmouseup={cb(&d, |s, _e: MouseEvent| s.drag = None)}>
        <div class="desk" onclick={cb(&d, |s, _e: MouseEvent| s.menu = false)}>
          { for ALL.iter().enumerate().map(|(i, k)| { let k = *k; html! {
              <div class="icon" style={format!("left:12px;top:{}px", 12 + i * 84)}
                   ondblclick={cb(&d, move |s, _e: MouseEvent| s.open(k))}>
                <span>{ k.icon() }</span>{ k.title().split(" - ").next().unwrap_or("") }
              </div> } }) }
          <div class="widgets"><ClockWidget /><Monitor /></div>
          { for d.wins.iter().filter(|w| !w.min).map(|w| {
              let (id, x, y, k) = (w.id, w.x, w.y, w.kind);
              html! {
                <div class="win" style={format!("left:{}px;top:{}px;z-index:{}", w.x, w.y, w.z)}>
                  <div class="title" onmousedown={cb(&d, move |s, e: MouseEvent| { s.focus(id); s.drag = Some((id, e.client_x() - x, e.client_y() - y)); })}>
                    <span>{ format!("{} {}", k.icon(), k.title()) }</span>
                    <span>
                      <button onmousedown={stop.clone()} onclick={cb(&d, move |s, _e: MouseEvent| s.toggle(id))}>{"_"}</button>
                      <button onmousedown={stop.clone()} onclick={cb(&d, move |s, _e: MouseEvent| s.wins.retain(|w| w.id != id))}>{"x"}</button>
                    </span>
                  </div>
                  <div class="body" onmousedown={cb(&d, move |s, _e: MouseEvent| s.focus(id))}>{ body(k) }</div>
                </div> } }) }
        </div>
        <div class="bar">
          <button class="start-btn" onclick={cb(&d, |s, _e: MouseEvent| s.menu = !s.menu)}>{"🪟 Start"}</button>
          <div class="tasks">
            { for d.wins.iter().map(|w| {
                let id = w.id;
                let on = !w.min && Some(w.z) == top;
                html! { <button class={classes!("task", on.then_some("on"))} onclick={cb(&d, move |s, _e: MouseEvent| s.toggle(id))}>{ format!("{} {}", w.kind.icon(), w.kind.title()) }</button> } }) }
          </div>
          <div class="tray"><Clock /></div>
          { if d.menu { html! {
              <div class="start">
                { for ALL.iter().map(|k| { let k = *k; html! { <div onclick={cb(&d, move |s, _e: MouseEvent| s.open(k))}>{ format!("{} {}", k.icon(), k.title()) }</div> } }) }
                <hr />
                <div onclick={cb(&d, |s, _e: MouseEvent| { s.wins.clear(); s.menu = false; })}>{"🔌 Shut Down..."}</div>
              </div> } } else { html! {} } }
        </div>
      </div>
    }
}

fn main() { yew::Renderer::<App>::new().render(); }
