use crate::blog::metadata::{BlogEntry, Locale, Tag, date};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct ChatControlReplyV;

impl BlogEntry for ChatControlReplyV {
    const UID: u32 = 6;

    const PUBLISH_DATE: DateTime<Utc> = date(2025, 9, 4);

    const LOCALE: Option<Locale> = Some(Locale::Swedish);

    const TITLE: &'static str = "RE: Chat Control";
    const AUTHOR: &'static str = "Hanna Gedin & Jonas Sjöstedt (Charlotta Tjärdahl)";
    const HIDDEN: bool = true;

    const TAGS: &'static [Tag] = &[];

    const PIN: Option<usize> = None;
}

#[lazy_route]
impl LazyRoute for ChatControlReplyV {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! {
            <p>"Hej,"</p>

            <p>"Tack för ditt mejl och ditt engagemang i frågan om Chat Control."</p>

            <p>
                "Vänsterpartiet är emot förslaget om Chat Control, som innebär massövervakning av privat kommunikation på nätet. Vi anser att förslaget hotar grundläggande rättigheter som rätten till privatliv och kommunikationsfrihet. Att skanna alla medborgares meddelanden innebär en oacceptabel form av övervakning som riskerar att urholka demokratin och rättsstaten."
            </p>

            <p>
                "Vi arbetar aktivt i Europaparlamentet för att försvara rätten till privat kommunikation och skydda medborgares integritet, och kommer fortsätta kämpa emot förslag som underminerar detta."
            </p>

            <p>"Vänliga hälsningar,"<br /> "Hanna och Jonas"</p>
        }.into_any()
    }
}
