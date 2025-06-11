macro_rules! define_tag_sets {
    (
        unique: [ $( $unique_tag:literal ),* $(,)? ],
        $(
            $set_name:ident : [ $( $tag:literal ),* $(,)? ]
        ),* $(,)?
    ) => {
        $(
            #[inline(always)]
            pub fn $set_name(tag: &str) -> bool {
                matches!(tag, $( $tag )|*)
            }
        )*

        #[inline(always)]
        pub fn is_unique_tag(tag: &str) -> bool {
            matches!(tag, $( $unique_tag )|*)
        }
    };
}

define_tag_sets! {
    unique: [ "html", "head", "body", "title" ],
    html_default_scope: [ "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template" ],
    list_item_scope:    [ "ol", "ul" ],
    button_scope:       [ "button" ],
    table_scope:        [ "html", "table", "template" ],
    select_scope:       [ "optgroup", "option" ],
    table_body_context: [ "tbody", "tfoot", "thead", "template", "html" ],
    table_row_context:  [ "tr", "template", "html" ],
    td_th:              [ "td", "th" ],
    cursory_implied_end:[ "dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc" ],
    thorough_implied_end:[ "caption", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr", "dd", "dt", "li", "option", "optgroup", "p", "rb", "rp", "rt", "rtc" ],
    heading_tag:        [ "h1", "h2", "h3", "h4", "h5", "h6" ],
    special_tag:        [ "address", "applet", "area", "article", "aside", "base", "basefont", "bgsound", "blockquote", "body",
                          "br", "button", "caption", "center", "col", "colgroup", "dd", "details", "dir", "div", "dl", "dt", "embed",
                          "fieldset", "figcaption", "figure", "footer", "form", "frame", "frameset", "h1", "h2", "h3", "h4", "h5",
                          "h6", "head", "header", "hgroup", "hr", "html", "iframe", "img", "input", "isindex", "li", "link",
                          "listing", "main", "marquee", "menu", "meta", "nav", "noembed", "noframes", "noscript",
                          "object", "ol", "p", "param", "plaintext", "pre", "script", "section", "select", "source", "style",
                          "summary", "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead", "title", "tr", "track",
                          "ul", "wbr", "xmp" ],
}
