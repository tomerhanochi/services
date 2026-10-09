use htmlb::Component;
use htmlb::prelude::*;

use crate::class::{
    TOP_APP_BAR, TOP_APP_BAR_ACTIONS, TOP_APP_BAR_HEADLINE, TOP_APP_BAR_NAVIGATION,
    TOP_APP_BAR_SUBTITLE, TOP_APP_BAR_TITLES,
};

/// The bar at the top of a screen: navigation, a headline, and actions.
pub fn top_app_bar<H: IntoHtml>(headline: H) -> TopAppBar<H, (), (), ()> {
    TopAppBar {
        headline,
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
    fn dynamic_len_hint(&self) -> usize {
        self.headline.len_hint()
            + self.navigation.as_ref().map_or(0, N::len_hint)
            + self.subtitle.as_ref().map_or(0, S::len_hint)
            + self.actions.as_ref().map_or(0, A::len_hint)
    }
}
