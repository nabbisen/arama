use arama_i18n::t;
use iced::{
    Element,
    widget::{button, container, row, text},
};
use lucide_icons::iced::icon_group;

use super::{
    Header,
    message::{Event, Internal, Message},
};

impl Header {
    pub fn view(&self) -> Element<'_, Message> {
        // T077 (RFC 052 §4): the button's disabled state is itself the
        // message (RFC 047 §3.4/§7.1 - disabled means "the folder has
        // not finished indexing"), so it needs a name and, while
        // disabled, a reason in words - a fade alone said nothing.
        // `primary_readable_when_disabled` handles both states: its
        // non-`Disabled` arms delegate to `arama_theme::primary`
        // unchanged, so which one iced actually applies follows
        // `on_press_maybe` the same way `primary` alone always did.
        let similar_pairs_button =
            button(row![icon_group(), text(t("header.similarity_pairs"))].spacing(6))
                .style(arama_theme::primary_readable_when_disabled)
                .on_press_maybe(if self.embedding_cached {
                    Some(Message::Event(Event::SimilarPairsDialogOpen))
                } else {
                    None
                });

        // Not chosen: a tooltip (hover is the gate T064 removed) or
        // hiding the button until it is ready (the user could then
        // never learn the feature exists, or what unlocks it).
        let similar_pairs_group: Element<'_, Message> = if self.embedding_cached {
            similar_pairs_button.into()
        } else {
            row![
                similar_pairs_button,
                text(t("header.similarity_pairs.unavailable")),
            ]
            .spacing(8)
            .into()
        };

        // T077: a plain `row!` here let the Folder button's own label
        // starve to zero width once the new name+reason made this row
        // wider than before, at `ja`/Larger/760x520 - the same shape
        // T063/T064 both fixed elsewhere. `Row::wrap()` moves the
        // similar-pairs group to its own line instead; the container's
        // former fixed `.height(60)` would have clipped exactly that
        // second line, so it is removed in favour of the row's own
        // natural height - unchanged (still ~60px) whenever nothing
        // wraps.
        let inner = row![
            self.dir_nav
                .view()
                .map(|x| Message::Internal(Internal::DirNavMessage(x))),
            similar_pairs_group,
        ]
        .spacing(20)
        .padding(10)
        .wrap();

        container(inner).into()
    }
}
