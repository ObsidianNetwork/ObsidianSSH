//! Dashboard grid: empty state. Split out of views/dashboard/grid/mod.rs.

use super::*;
use iced::widget::column;

/// Width of the whole centered block (input, Continue, the "or"
/// divider and every secondary action), so the column reads as one
/// object instead of a stack of differently sized parts.
const BLOCK_WIDTH: f32 = 380.0;

impl ObsidianSsh {
    /// Centered empty state shown when no hosts/groups/session groups exist.
    ///
    /// No toolbar here: with an empty vault there is nothing to search,
    /// sort, filter or re-layout, and the add menu's entries render as
    /// real buttons below instead (same catalog, `add_host_actions`).
    /// Matches the other empty vault views (see `view_history`).
    pub(crate) fn dashboard_empty_state(&self) -> Element<'_, Message> {
        // Termius-style empty state, centered "Create host" with input.
        // Two named ways out of the field instead of one "Continue" that
        // quietly connected whenever the target looked explicit: Add host
        // always opens the editor (pre-filled), Connect dials without
        // saving and only lights up once there is something to dial.
        let typed = self.cur_nav().quick_host_input.trim();
        let can_connect = !typed.is_empty() && self.quick_connect_target(typed).is_some();

        // The toolbar's recording is from a previous frame (a host was
        // just deleted); the toolbar isn't on screen, so drop it and
        // blank the anchor cells its dropdowns would point at. The
        // content zone was cleared by the caller and is re-recorded
        // below, in display order.
        self.keynav_toolbar_reset();
        self.keynav_toolbar_zero_trigger_bounds();

        let mut items: Vec<Element<'_, Message>> = vec![
            // Icon
            container(iced_fonts::lucide::server().size(32).color(ObsidianSshColors::t().text_muted))
                .padding(16)
                .style(|_| container::Style {
                    background: Some(Background::Color(ObsidianSshColors::t().bg_surface)),
                    border: Border { radius: Radius::from(12.0), ..Default::default() },
                    ..Default::default()
                })
                .boxed(),
            Space::new().height(20).boxed(),
            text(crate::i18n::t("create_host_title"))
                .size(20)
                .color(ObsidianSshColors::t().text_primary)
                .boxed(),
            Space::new().height(8).boxed(),
            text(crate::i18n::t("create_host_desc")).size(13).color(ObsidianSshColors::t().text_muted).boxed(),
            Space::new().height(24).boxed(),
            // Hostname input. Enter on its keyboard row focuses it (the
            // id), typing then submits with the same Enter.
            self.content_action_slot(
                crate::keynav::RowAction::input(iced::widget::Id::new(QUICK_HOST_INPUT_ID)),
                8.0,
                text_input(t("type_ip_or_hostname"), &self.cur_nav().quick_host_input)
                    .id(QUICK_HOST_INPUT_ID)
                    .on_input(|v| Message::Navigation(NavigationMessage::QuickHostInput(v)))
                    // No submit binding while a side panel is open: the
                    // fork's `text_input` fires on_submit on ANY Enter,
                    // focused or not, and the empty state stays mounted
                    // BEHIND the host editor, so an Enter meant for the
                    // editor also landed here and rebuilt its form (the
                    // dispatcher's modal guard cannot help, the message
                    // itself must not exist). The click path is
                    // unaffected: the Continue button below keeps its
                    // own on_press.
                    .on_submit_maybe(
                        (!self.side_panel_open())
                            .then(|| Message::Navigation(NavigationMessage::QuickHostContinue)),
                    )
                    .padding(14)
                    .width(BLOCK_WIDTH)
                    .style(crate::widgets::rounded_input_style)
                    .align_x(dir_align_x())
                    .boxed(),
            ),
            Space::new().height(12).boxed(),
            crate::widgets::dir_row(vec![
                self.content_action_slot(
                    crate::keynav::RowAction::activate(Message::Navigation(NavigationMessage::QuickHostAdd)),
                    8.0,
                    first_run_button(
                        iced_fonts::lucide::plus(),
                        crate::i18n::t("add_host_btn"),
                        Some(Message::Navigation(NavigationMessage::QuickHostAdd)),
                        true,
                    ),
                ),
                Space::new().width(BUTTON_GAP).boxed(),
                if can_connect {
                    self.content_action_slot(
                        crate::keynav::RowAction::activate(Message::Navigation(
                            NavigationMessage::QuickHostConnect,
                        )),
                        8.0,
                        first_run_button(
                            iced_fonts::lucide::plug(),
                            crate::i18n::t("connect"),
                            Some(Message::Navigation(NavigationMessage::QuickHostConnect)),
                            false,
                        ),
                    )
                } else {
                    first_run_button(iced_fonts::lucide::plug(), crate::i18n::t("connect"), None, false)
                },
            ])
            .boxed(),
        ];

        // Secondary paths: the "+ Host ▾" menu's own entries, as
        // buttons. A dropdown on an otherwise blank screen hides the
        // only other ways in (import) behind a chevron
        // the first-run user has no reason to click.
        let actions = self.add_host_actions();
        if !actions.is_empty() {
            items.push(Space::new().height(24).boxed());
            items.push(crate::views::add_actions::or_divider(BLOCK_WIDTH));
            items.push(Space::new().height(16).boxed());
            for action in actions {
                items.push(self.content_action_slot(
                    crate::keynav::RowAction::activate(action.msg.clone()),
                    8.0,
                    crate::views::add_actions::secondary_action_button(action, BLOCK_WIDTH),
                ));
                items.push(Space::new().height(8).boxed());
            }
        }

        let empty_state = container(column(items).align_x(iced::Alignment::Center)).center(Length::Fill);

        column![empty_state].width(Length::Fill).height(Length::Fill).boxed()
    }
}

/// Text-input id of the quick-host field, shared by the widget and its
/// keyboard row (`RowAction::input` focuses by id).
const QUICK_HOST_INPUT_ID: &str = "empty-quick-host";

/// Gap between the Add host and Connect buttons; each takes half of
/// what is left of the block.
const BUTTON_GAP: f32 = 8.0;

/// One of the two buttons under the first-run host field. `primary` is
/// the accent fill (Add host); `msg: None` draws it disabled (Connect
/// with nothing to dial).
fn first_run_button<'a>(
    icon: iced::widget::Text<'a>,
    label: &'a str,
    msg: Option<Message>,
    primary: bool,
) -> Element<'a, Message> {
    let width = (BLOCK_WIDTH - BUTTON_GAP) / 2.0;
    let c = ObsidianSshColors::t();
    let enabled = msg.is_some();
    let fg = match (primary, enabled) {
        (true, _) => c.button_text,
        (false, true) => c.text_primary,
        (false, false) => c.text_muted,
    };
    button(
        container(
            crate::widgets::dir_row(vec![
                icon.size(14).color(fg).boxed(),
                Space::new().width(6).boxed(),
                text(label).size(14).color(fg).boxed(),
            ])
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding { top: 12.0, right: 0.0, bottom: 12.0, left: 0.0 })
        .center_x(width),
    )
    .on_press_maybe(msg)
    .width(width)
    .style(move |_, status| {
        let c = ObsidianSshColors::t();
        let lifted = matches!(status, BtnStatus::Hovered | BtnStatus::Pressed);
        let bg = match (primary, lifted) {
            (true, true) => c.button_bg_hover,
            (true, false) => c.button_bg,
            (false, true) => c.bg_hover,
            (false, false) => c.bg_surface,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            border: Border {
                radius: Radius::from(8.0),
                color: if primary { Color::TRANSPARENT } else { c.border },
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .boxed()
}


