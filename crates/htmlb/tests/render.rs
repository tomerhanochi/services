use htmlb::prelude::*;

/// Size estimates must be lower bounds: MIN_LEN <= len_hint <= output length.
/// Takes an expression (not a value) so the view can be built twice; elements aren't Clone.
macro_rules! check {
    ($view:expr, $expected:expr) => {{
        let make = || $view;
        let v = make();
        let hint = <_ as IntoHtml>::len_hint(&v);
        let min = min_len_of(&v);
        let out = make().to_html();
        assert_eq!(out, $expected);
        assert!(min <= hint, "MIN_LEN {min} > len_hint {hint}");
        assert!(hint <= out.len(), "len_hint {hint} > output {}", out.len());
    }};
}

fn min_len_of<V: IntoHtml>(_: &V) -> usize {
    V::MIN_LEN
}

#[test]
fn elements_and_attributes() {
    check!(div(), "<div></div>");
    check!(br(), "<br>");
    check!(
        a().href("/x?a=1&b=2").class("link").child("go"),
        r#"<a href="/x?a=1&#38;b=2" class="link">go</a>"#
    );
    check!(
        input().r#type(InputType::Email).name("e").required(true).disabled(false).maxlength(64u32),
        r#"<input type="email" name="e" required maxlength="64">"#
    );
    check!(
        table().child(tr().child(td().colspan(2).child(3))),
        r#"<table><tr><td colspan="2">3</td></tr></table>"#
    );
    check!(div().attr("data-id", 42).child(()), r#"<div data-id="42"></div>"#);
    check!(button().draggable(true).child("b"), r#"<button draggable="true">b</button>"#);
    check!(ol().child(li().value(Some(-3)).child("x")), r#"<ol><li value="-3">x</li></ol>"#);
    check!(ol().child(li().value(None::<i32>).child("x")), "<ol><li>x</li></ol>");
}

#[test]
fn escaping() {
    check!(p().child("a < b & c > d \"q\""), "<p>a &#60; b &#38; c &#62; d &#34;q&#34;</p>");
    check!(p().title("say \"hi\" <now>"), r#"<p title="say &#34;hi&#34; &#60;now&#62;"></p>"#);
    check!(p().child('<'), "<p>&#60;</p>");
    check!(script().child("if (a < b && c) {}"), "<script>if (a < b && c) {}</script>");
    check!(div().child(raw("<b>bold</b>")), "<div><b>bold</b></div>");
}

#[test]
fn optional_attributes_and_concatenation() {
    check!(a().href(("/files/", "a b")), r#"<a href="/files/a b"></a>"#);
    check!(li().class(None::<&str>), "<li></li>");
    check!(li().class(Some("champion")), r#"<li class="champion"></li>"#);
}

#[test]
fn class_lists_are_space_separated() {
    for (active, expected) in [(true, r#"<a class="btn active"></a>"#), (false, r#"<a class="btn"></a>"#)] {
        check!(a().class(("btn", active.then_some("active"))), expected);
    }
    // Leading `None`s and empty strings write no stray spaces.
    check!(a().class((None::<&str>, "", "x", Some("y"), ("z", String::from("w")))), r#"<a class="x y z w"></a>"#);
    check!(a().class((None::<&str>, None::<&str>)), "<a></a>");
    check!(a().class(("", "")), r#"<a class=""></a>"#);
    // Escaped, like any attribute value.
    check!(a().class(("a\"", "b")), r#"<a class="a&#34; b"></a>"#);
    check!(ul().child(li().class(None::<&str>)), "<ul><li></li></ul>");
    check!(ul().child(li().class(Some("champion"))), r#"<ul><li class="champion"></li></ul>"#);
}

#[test]
fn children_and_containers() {
    check!(h1().child(("CSL ", 2015u16)), "<h1>CSL 2015</h1>");
    check!(div().child("a").child(1).child('c'), "<div>a1c</div>");
    check!(div().child(Some(span())).child(None::<&str>), "<div><span></span></div>");
    check!(ul().child(vec![li().child("a"), li().child("b")]), "<ul><li>a</li><li>b</li></ul>");
    check!(ul().child([li(), li()]), "<ul><li></li><li></li></ul>");
    let pick = |left: bool| if left { Either::Left(b().child("l")) } else { Either::Right(i().child("r")) };
    check!(div().child(pick(true)), "<div><b>l</b></div>");
    check!(div().child(pick(false)), "<div><i>r</i></div>");
    check!((doctype(), html().child((head().child(title().child("t")), body()))),
        "<!DOCTYPE html><html><head><title>t</title></head><body></body></html>");
    check!(f64::NAN, "NaN");
    check!(-1.5f32, "-1.5");
}

#[test]
fn iterators() {
    let items = ["a", "b<", "c"];
    // single pass: cheap lower bound
    let v = ul().child(items.iter().map(|s| li().child(*s)).into_html());
    assert_eq!(<_ as IntoHtml>::len_hint(&v), "<ul></ul>".len() + 3 * "<li></li>".len());
    assert_eq!(v.to_html(), "<ul><li>a</li><li>b&#60;</li><li>c</li></ul>");
    // cloned: precise
    check!(
        ul().child(items.iter().map(|s| li().child(*s)).into_html_cloned()),
        "<ul><li>a</li><li>b&#60;</li><li>c</li></ul>"
    );
    // From<Iterator>
    let iter: htmlb::IterHtml<_> = items.iter().map(|s| li().child(*s)).into();
    assert_eq!(ul().child(iter).to_html(), "<ul><li>a</li><li>b&#60;</li><li>c</li></ul>");
}

#[test]
fn helpers_return_plain_impl_into_html() {
    fn item(name: &str) -> impl IntoHtml + '_ {
        li().child(name)
    }
    fn cell(n: u32) -> impl IntoHtml {
        td().child(n)
    }
    fn page(body_content: impl IntoHtml) -> impl IntoHtml {
        (doctype(), html().child(body().child(body_content)))
    }
    check!(ul().child((item("a"), item("b"))), "<ul><li>a</li><li>b</li></ul>");
    check!(tr().child([cell(1), cell(2)]), "<tr><td>1</td><td>2</td></tr>");
    check!(
        page(ul().child(["x", "y"].iter().map(|s| item(s)).into_html_cloned())),
        "<!DOCTYPE html><html><body><ul><li>x</li><li>y</li></ul></body></html>"
    );
    // standalone fragments (e.g. htmx partials) render directly
    assert_eq!(item("z").to_html(), "<li>z</li>");
}

#[test]
fn tables_selects_lists() {
    check!(
        table().child((
            caption().child("c"),
            thead().child(tr().child(th().scope(Scope::Col).child("h"))),
            tbody().child(tr().child((td().child(1), td().child(2)))),
        )),
        r#"<table><caption>c</caption><thead><tr><th scope="col">h</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table>"#
    );
    check!(
        select().child((option().value("a").child("A"), optgroup().label("g").child(option().child("B")))),
        r#"<select><option value="a">A</option><optgroup label="g"><option>B</option></optgroup></select>"#
    );
    check!(dl().child((dt().child("k"), dd().child("v"))), "<dl><dt>k</dt><dd>v</dd></dl>");
    check!(ul().child(raw("<li>pre-rendered</li>")), "<ul><li>pre-rendered</li></ul>");
}

#[test]
fn min_len_is_computed_from_the_type() {
    // <a class="…"><b>…</b></a> with unknown string contents
    type View = htmlb::El<
        htmlb::tag::a,
        ((), htmlb::Attr<htmlb::attr::class, htmlb::Classes<&'static str>>),
        ((), htmlb::El<htmlb::tag::b, (), ((), &'static str)>),
    >;
    assert_eq!(<View as IntoHtml>::MIN_LEN, r#"<a class=""><b></b></a>"#.len());
    let _: fn() -> View = || a().class("x").child(b().child("y"));
    // Option attributes contribute nothing to MIN_LEN, required ones do.
    assert_eq!(min_len_of(&li().class(Some("x"))), "<li></li>".len());
    assert_eq!(min_len_of(&li().class("x")), r#"<li class=""></li>"#.len());
}

#[test]
fn display_is_escaped_in_text_and_attributes() {
    use htmlb::prelude::*;
    struct Size(u64);
    impl std::fmt::Display for Size {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} <KiB> & \"more\"", self.0)
        }
    }
    let html = span().title(display(Size(3))).child(display(Size(4))).to_html();
    assert_eq!(
        html,
        r#"<span title="3 &#60;KiB&#62; &#38; &#34;more&#34;">4 &#60;KiB&#62; &#38; &#34;more&#34;</span>"#
    );
}

#[test]
fn popover_attributes_are_typed() {
    use htmlb::prelude::*;
    let html = (
        button().popovertarget("m").popovertargetaction(PopoverTargetAction::Show).child("Open"),
        div().id("m").popover(Popover::Auto),
    )
        .to_html();
    assert_eq!(html, r#"<button popovertarget="m" popovertargetaction="show">Open</button><div id="m" popover="auto"></div>"#);
}
