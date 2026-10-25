use crate::prelude::Unset;
use htmlb::prelude::*;

macro_rules! text_component {
    ($name:ident, $constructor:ident, $tag:ident, $class:literal, $description:literal) => {
        #[doc = $description]
        pub fn $constructor() -> $name<Unset> {
            $name { content: Unset }
        }

        pub struct $name<C> {
            content: C,
        }

        impl<C> $name<C> {
            pub fn child<N: IntoHtml>(self, child: N) -> $name<N> {
                $name { content: child }
            }
        }

        impl<C: IntoHtml> Component for $name<C> {
            fn render(self) -> impl IntoHtml {
                $tag().class($class).child(self.content)
            }
        }

        impl<C: IntoHtml> RunTimeHtmlSizeHint for $name<C> {
            fn size_hint(&self) -> usize {
                $class.len() + self.content.size_hint()
            }
        }
    };
}

text_component!(
    DisplayText,
    display,
    h1,
    "md-type-display-large",
    "The primary page heading, using Material's display-large style."
);
text_component!(
    HeadlineText,
    headline,
    h2,
    "md-type-headline-large",
    "A section heading using Material's headline-large style."
);
text_component!(
    TitleText,
    title,
    h3,
    "md-type-title-large",
    "A subsection heading using Material's title-large style."
);
text_component!(
    SubtitleText,
    subtitle,
    p,
    "md-type-body-medium md-color-on-surface-variant",
    "Supporting text below a heading, using body-medium and the on-surface-variant color."
);
text_component!(
    ParagraphText,
    paragraph,
    p,
    "md-type-body-large",
    "Prose using Material's body-large style."
);
text_component!(
    SupportingText,
    supporting_text,
    p,
    "md-type-body-small md-color-on-surface-variant",
    "Secondary explanatory text using body-small and the on-surface-variant color."
);
text_component!(
    LabelText,
    label,
    span,
    "md-type-label-large",
    "Concise label text using Material's label-large style."
);
