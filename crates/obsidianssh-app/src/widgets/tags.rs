//! Bubble editor shared by host and macro forms.

use iced::Widget as _;
use crate::state::{TagInput, TagInputMessage};
use super::*;

pub(crate) fn tag_input<'a>(
    value: &'a TagInput,
    id: &'static str,
    on_edit: fn(TagInputMessage) -> Message,
) -> Element<'a, Message> {
    let c = ObsidianSshColors::t();
    let chips = value.tags.iter().map(|tag| {
        let remove = button(iced_fonts::lucide::x().size(12))
            .padding(4)
            .on_press(on_edit(TagInputMessage::Remove(tag.clone())))
            .style(|_, status| button::Style {
                background: Some(Background::Color(match status {
                    BtnStatus::Hovered | BtnStatus::Pressed => ObsidianSshColors::t().bg_selected,
                    _ => Color::TRANSPARENT,
                })),
                text_color: ObsidianSshColors::t().text_primary,
                border: Border { radius: Radius::from(12.0), ..Default::default() },
                ..Default::default()
            });
        container(
            dir_row(vec![
                text(tag).size(12).color(c.text_primary).boxed(),
                iced::widget::tooltip(
                    remove,
                    text(crate::i18n::t("remove")).size(12),
                    iced::widget::tooltip::Position::Top,
                ).boxed(),
            ])
            .spacing(4)
            .align_y(iced::Alignment::Center),
        )
        .padding([3, 8])
        .style(|_| container::Style {
            background: Some(Background::Color(ObsidianSshColors::t().bg_selected)),
            border: Border {
                radius: Radius::from(16.0),
                color: ObsidianSshColors::t().border,
                width: 1.0,
            },
            ..Default::default()
        })
        .boxed()
    });
    let input = text_input(crate::i18n::t("snippet_tag_suggestions"), &value.draft)
        .id(iced::widget::Id::new(id))
        .on_input(move |text| on_edit(TagInputMessage::Changed(text)))
        .on_submit(on_edit(TagInputMessage::Commit))
        .padding(8)
        .size(13)
        .style(rounded_input_style)
        .align_x(dir_align_x());
    let add = button(text(crate::i18n::t("add")).size(12))
        .padding([8, 12])
        .on_press_maybe(
            (!value.draft.trim().is_empty()).then(|| on_edit(TagInputMessage::Commit)),
        )
        .style(|_, status| {
            let c = ObsidianSshColors::t();
            button::Style {
                background: Some(Background::Color(match status {
                    BtnStatus::Hovered | BtnStatus::Pressed => c.button_bg_hover,
                    BtnStatus::Disabled => c.bg_surface,
                    _ => c.button_bg,
                })),
                text_color: if matches!(status, BtnStatus::Disabled) { c.text_muted } else { c.button_text },
                border: Border { radius: Radius::from(8.0), ..Default::default() },
                ..Default::default()
            }
        });
    // Keep both rows mounted so adding a bubble does not replace the
    // text input's tree state (focus, cursor and IME composition).
    container(
        iced::widget::Column::new()
            .push(Row::with_children(chips).spacing(6).wrap().vertical_spacing(6).boxed())
            .push(dir_row(vec![input.boxed(), add.boxed()]).spacing(6).align_y(iced::Alignment::Center).boxed())
            .spacing(if value.tags.is_empty() { 0 } else { 8 }),
    )
    .padding(8)
    .width(Length::Fill)
    .style(|_| container::Style {
        background: Some(Background::Color(ObsidianSshColors::t().bg_surface)),
        border: Border {
            radius: Radius::from(INPUT_RADIUS),
            color: ObsidianSshColors::t().border,
            width: 1.0,
        },
        ..Default::default()
    })
    .boxed()
}
