// `tag: Kind { attribute: ValueKind }`
//
// Kind: `Normal` (any children, escaped), `Void` (no children), `RawText` (unescaped
// string children: `<script>`, `<style>`). ValueKind: `Text` (any AttrValue), `Bool`
// (boolean attribute), `Class` (a ClassList), or a concrete value type.
elements! {
    a: Normal { download: Text, href: Text, hreflang: Text, ping: Text, referrerpolicy: ReferrerPolicy, rel: Text, target: Text, r#type: Text }
    abbr: Normal {}
    address: Normal {}
    area: Void { alt: Text, coords: Text, download: Text, href: Text, hreflang: Text, ping: Text, rel: Text, shape: Text, target: Text }
    article: Normal {}
    aside: Normal {}
    audio: Normal { autoplay: Bool, controls: Bool, crossorigin: CrossOrigin, r#loop: Bool, muted: Bool, preload: Preload, src: Text }
    b: Normal {}
    base: Void { href: Text, target: Text }
    bdi: Normal {}
    bdo: Normal {}
    blockquote: Normal { cite: Text }
    body: Normal {}
    br: Void {}
    button: Normal { command: Command, commandfor: Text, disabled: Bool, form: Text, formaction: Text, formenctype: FormEnctype, formmethod: FormMethod, formnovalidate: Bool, formtarget: Text, name: Text, r#type: ButtonType, value: Text, popovertarget: Text, popovertargetaction: PopoverTargetAction }
    canvas: Normal { height: u32, width: u32 }
    caption: Normal {}
    cite: Normal {}
    code: Normal {}
    col: Void { span: u32 }
    colgroup: Normal { span: u32 }
    data: Normal { value: Text }
    datalist: Normal {}
    dd: Normal {}
    del: Normal { cite: Text, datetime: Text }
    details: Normal { name: Text, open: Bool }
    dfn: Normal {}
    dialog: Normal { closedby: Text, open: Bool }
    div: Normal {}
    dl: Normal {}
    dt: Normal {}
    em: Normal {}
    embed: Void { height: u32, src: Text, r#type: Text, width: u32 }
    fieldset: Normal { disabled: Bool, form: Text, name: Text }
    figcaption: Normal {}
    figure: Normal {}
    footer: Normal {}
    form: Normal { accept_charset: Text, action: Text, autocomplete: Text, enctype: FormEnctype, method: FormMethod, name: Text, novalidate: Bool, target: Text }
    h1: Normal {}
    h2: Normal {}
    h3: Normal {}
    h4: Normal {}
    h5: Normal {}
    h6: Normal {}
    head: Normal {}
    header: Normal {}
    hgroup: Normal {}
    hr: Void {}
    html: Normal {}
    i: Normal {}
    iframe: Normal { allow: Text, allowfullscreen: Bool, allowpaymentrequest: Bool, height: u32, name: Text, referrerpolicy: ReferrerPolicy, sandbox: Text, src: Text, srcdoc: Text, width: u32 }
    img: Void { alt: Text, attributionsrc: Text, crossorigin: CrossOrigin, decoding: Decoding, elementtiming: Text, fetchpriority: FetchPriority, height: u32, ismap: Bool, loading: Loading, referrerpolicy: ReferrerPolicy, sizes: Text, src: Text, srcset: Text, usemap: Text, width: u32 }
    input: Void { accept: Text, alt: Text, autocomplete: Text, capture: Text, checked: Bool, dirname: Text, disabled: Bool, form: Text, formaction: Text, formenctype: FormEnctype, formmethod: FormMethod, formnovalidate: Bool, formtarget: Text, height: u32, list: Text, max: Text, maxlength: u32, min: Text, minlength: u32, multiple: Bool, name: Text, pattern: Text, placeholder: Text, popovertarget: Text, popovertargetaction: PopoverTargetAction, readonly: Bool, required: Bool, size: u32, src: Text, step: Text, r#type: InputType, value: Text, width: u32 }
    ins: Normal { cite: Text, datetime: Text }
    kbd: Normal {}
    label: Normal { r#for: Text, form: Text }
    legend: Normal {}
    li: Normal { value: i32 }
    link: Void { r#as: Text, blocking: Text, crossorigin: CrossOrigin, fetchpriority: FetchPriority, href: Text, hreflang: Text, imagesizes: Text, imagesrcset: Text, integrity: Text, media: Text, rel: Text, referrerpolicy: ReferrerPolicy, sizes: Text, r#type: Text }
    main: Normal {}
    map: Normal { name: Text }
    mark: Normal {}
    menu: Normal {}
    meta: Void { charset: Text, content: Text, http_equiv: Text, name: Text }
    meter: Normal { value: f64, min: f64, max: f64, low: f64, high: f64, optimum: f64, form: Text }
    nav: Normal {}
    noscript: Normal {}
    object: Normal { data: Text, form: Text, height: u32, name: Text, r#type: Text, usemap: Text, width: u32 }
    ol: Normal { reversed: Bool, start: i32, r#type: OlType }
    optgroup: Normal { disabled: Bool, label: Text }
    option: Normal { disabled: Bool, label: Text, selected: Bool, value: Text }
    output: Normal { r#for: Text, form: Text, name: Text }
    p: Normal {}
    picture: Normal {}
    pre: Normal {}
    progress: Normal { min: f64, max: f64, value: f64 }
    q: Normal { cite: Text }
    rp: Normal {}
    rt: Normal {}
    ruby: Normal {}
    s: Normal {}
    samp: Normal {}
    script: RawText { r#async: Bool, crossorigin: CrossOrigin, defer: Bool, fetchpriority: FetchPriority, integrity: Text, nomodule: Bool, referrerpolicy: ReferrerPolicy, src: Text, r#type: Text, blocking: Text }
    search: Normal {}
    section: Normal {}
    select: Normal { autocomplete: Text, disabled: Bool, form: Text, multiple: Bool, name: Text, required: Bool, size: u32 }
    slot: Normal { name: Text }
    small: Normal {}
    source: Void { src: Text, r#type: Text, srcset: Text, sizes: Text, media: Text, height: u32, width: u32 }
    span: Normal {}
    strong: Normal {}
    style: RawText { media: Text, blocking: Text }
    sub: Normal {}
    summary: Normal {}
    sup: Normal {}
    table: Normal {}
    tbody: Normal {}
    td: Normal { colspan: u32, headers: Text, rowspan: u32 }
    template: Normal {}
    textarea: Normal { autocomplete: Text, cols: u32, dirname: Text, disabled: Bool, form: Text, maxlength: u32, minlength: u32, name: Text, placeholder: Text, readonly: Bool, required: Bool, rows: u32, wrap: Wrap }
    tfoot: Normal {}
    th: Normal { abbr: Text, colspan: u32, headers: Text, rowspan: u32, scope: Scope }
    thead: Normal {}
    time: Normal { datetime: Text }
    title: Normal {}
    tr: Normal {}
    track: Void { default: Bool, kind: TrackKind, label: Text, src: Text, srclang: Text }
    u: Normal {}
    ul: Normal {}
    var: Normal {}
    video: Normal { autoplay: Bool, controls: Bool, controlslist: Text, crossorigin: CrossOrigin, disablepictureinpicture: Bool, disableremoteplayback: Bool, height: u32, r#loop: Bool, muted: Bool, playsinline: Bool, poster: Text, preload: Preload, src: Text, width: u32 }
    wbr: Void {}
}

global_attributes! {
        accesskey: Text,
        autocapitalize: AutoCapitalize,
        autofocus: Bool,
        class: Class,
        contenteditable: ContentEditable,
        dir: Dir,
        draggable: bool,
        enterkeyhint: EnterKeyHint,
        exportparts: Text,
        hidden: Bool,
        id: Text,
        inert: Bool,
        inputmode: InputMode,
        is: Text,
        itemid: Text,
        itemprop: Text,
        itemref: Text,
        itemscope: Bool,
        itemtype: Text,
        lang: Text,
        nonce: Text,
        part: Text,
        popover: Popover,
        role: Text,
        slot: Text,
        spellcheck: bool,
        style: Text,
        tabindex: i32,
        title: Text,
        translate: Translate,
        virtualkeyboardpolicy: Text,
        aria_activedescendant: Text,
        aria_atomic: Text,
        aria_autocomplete: Text,
        aria_busy: Text,
        aria_checked: Text,
        aria_colcount: Text,
        aria_colindex: Text,
        aria_colspan: Text,
        aria_controls: Text,
        aria_current: Text,
        aria_describedby: Text,
        aria_description: Text,
        aria_details: Text,
        aria_disabled: Text,
        aria_dropeffect: Text,
        aria_errormessage: Text,
        aria_expanded: Text,
        aria_flowto: Text,
        aria_grabbed: Text,
        aria_haspopup: Text,
        aria_hidden: Text,
        aria_invalid: Text,
        aria_keyshortcuts: Text,
        aria_label: Text,
        aria_labelledby: Text,
        aria_live: Text,
        aria_modal: Text,
        aria_multiline: Text,
        aria_multiselectable: Text,
        aria_orientation: Text,
        aria_owns: Text,
        aria_placeholder: Text,
        aria_posinset: Text,
        aria_pressed: Text,
        aria_readonly: Text,
        aria_relevant: Text,
        aria_required: Text,
        aria_roledescription: Text,
        aria_rowcount: Text,
        aria_rowindex: Text,
        aria_rowspan: Text,
        aria_selected: Text,
        aria_setsize: Text,
        aria_sort: Text,
        aria_valuemax: Text,
        aria_valuemin: Text,
        aria_valuenow: Text,
        aria_valuetext: Text,
}
