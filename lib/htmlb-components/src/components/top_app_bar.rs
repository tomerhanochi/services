use crate::prelude::Unset;
use htmlb::prelude::*;

const TOP_APP_BAR: &str = "md-top-app-bar";
const TOP_APP_BAR_NAVIGATION: &str = "md-top-app-bar__navigation";
const TOP_APP_BAR_TITLES: &str = "md-top-app-bar__titles";
const TOP_APP_BAR_HEADLINE: &str = "md-top-app-bar__headline";
const TOP_APP_BAR_SUBTITLE: &str = "md-top-app-bar__subtitle";
const TOP_APP_BAR_ACTIONS: &str = "md-top-app-bar__actions";

/// The bar at the top of a screen: navigation, a headline, and actions.
pub fn top_app_bar() -> TopAppBar<Unset, (), (), ()> {
    TopAppBar {
        headline: Unset,
        navigation: None,
        subtitle: None,
        actions: None,
    }
}

pub struct TopAppBar<H, N, S, A> {
    headline: H,
    navigation: Option<N>,
    subtitle: Option<S>,
    actions: Option<A>,
}

impl<H, N, S, A> TopAppBar<H, N, S, A> {
    pub fn headline<H2: IntoHtml>(self, headline: H2) -> TopAppBar<H2, N, S, A> {
        TopAppBar {
            headline,
            navigation: self.navigation,
            subtitle: self.subtitle,
            actions: self.actions,
        }
    }

    /// Usually an [`icon_button`](crate::icon_button) going back or up.
    pub fn navigation<N2: IntoHtml>(self, navigation: N2) -> TopAppBar<H, N2, S, A> {
        TopAppBar {
            headline: self.headline,
            navigation: Some(navigation),
            subtitle: self.subtitle,
            actions: self.actions,
        }
    }

    /// A line below the headline, e.g. where the screen is.
    pub fn subtitle<S2: IntoHtml>(self, subtitle: S2) -> TopAppBar<H, N, S2, A> {
        TopAppBar {
            headline: self.headline,
            navigation: self.navigation,
            subtitle: Some(subtitle),
            actions: self.actions,
        }
    }

    /// Icon buttons at the end of the bar.
    pub fn actions<A2: IntoHtml>(self, actions: A2) -> TopAppBar<H, N, S, A2> {
        TopAppBar {
            headline: self.headline,
            navigation: self.navigation,
            subtitle: self.subtitle,
            actions: Some(actions),
        }
    }
}

impl<H: IntoHtml, N: IntoHtml, S: IntoHtml, A: IntoHtml> Component for TopAppBar<H, N, S, A> {
    fn render(self) -> impl IntoHtml {
        header().class(TOP_APP_BAR).child((
            self.navigation
                .map(|n| div().class(TOP_APP_BAR_NAVIGATION).child(n)),
            div().class(TOP_APP_BAR_TITLES).child((
                h1().class(TOP_APP_BAR_HEADLINE).child(self.headline),
                self.subtitle
                    .map(|s| p().class(TOP_APP_BAR_SUBTITLE).child(s)),
            )),
            self.actions
                .map(|a| div().class(TOP_APP_BAR_ACTIONS).child(a)),
        ))
    }
}
impl<H: IntoHtml, N: IntoHtml, S: IntoHtml, A: IntoHtml> RunTimeHtmlSizeHint
    for TopAppBar<H, N, S, A>
{
    fn size_hint(&self) -> usize {
        self.headline.size_hint()
            + self.navigation.size_hint()
            + self.subtitle.size_hint()
            + self.actions.size_hint()
    }
}
