//! Dashboard main content, the responsive grid of folder cards and host
//! cards plus the early-return path (zero connections). The biggest chunk of
//! `view_dashboard`, lifted here so the orchestrator stays thin.
//!
//! Returns the full `main_content` (toolbar + search + status + body).
//! The mod-level `view_dashboard` only wraps it with the right-side
//! panel slot.

pub(crate) use iced::border::Radius;
pub(crate) use iced::widget::button::Status as BtnStatus;
pub(crate) use iced::widget::{button, container, scrollable, text, text_input, MouseArea, Space};
use iced::Widget as _;
use iced::widget::column;
pub(crate) use uuid::Uuid;
pub(crate) use iced::{Background, Border, Color, Element, Length, Padding};

pub(crate) use crate::app::{TabsMessage, SshMessage, NavigationMessage, DashNavItem, Message, ObsidianSsh, SessionGroupMessage, CARD_WIDTH};
pub(crate) use crate::i18n::t;
pub(crate) use crate::os_icon::BrandIcon;
pub(crate) use crate::theme::ObsidianSshColors;
pub(crate) use crate::widgets::{card_grid_columns, dir_align_x, dir_row, distribute_card_grid};

/// Count the leaf panes in a saved session-group layout (for the card
/// subtitle).
pub(crate) fn count_leaves(layout: &obsidianssh_core::models::PaneLayout) -> usize {
    match layout {
        obsidianssh_core::models::PaneLayout::Split { a, b, .. } => {
            count_leaves(a) + count_leaves(b)
        }
        obsidianssh_core::models::PaneLayout::Leaf(_) => 1,
    }
}

/// Map (card, accent-colour, nav-item) tuples to renderable cards: apply
/// the shared `widgets::card_accent_wash` when `glass` is on, then draw
/// the keyboard-selection ring on the item matching `selected`. A free fn
/// (not a closure) so the input/output lifetimes stay tied.
pub(crate) fn apply_card_wash<'a>(
    cards: Vec<(Element<'a, Message>, Color, DashNavItem)>,
    glass: bool,
    selected: Option<DashNavItem>,
    ring_bounds: crate::widgets::BoundsCell,
) -> Vec<Element<'a, Message>> {
    cards
        .into_iter()
        .map(|(el, c, nav)| {
            let el = if glass {
                crate::widgets::card_accent_wash(el, c)
            } else {
                el
            };
            // Ring wrapper on EVERY card (transparent when unringed,
            // see select_ring_opt); the bounds_reporter, invisible to
            // the widget tree, wraps only the ringed card so the Menu
            // key can anchor the context menu at its kebab corner.
            let ringed = selected == Some(nav);
            let el = crate::widgets::select_ring_opt(
                el,
                10.0,
                ringed.then(|| crate::theme::ObsidianSshColors::t().accent),
            );
            if ringed {
                crate::widgets::bounds_reporter(el, ring_bounds.clone())
            } else {
                el
            }
        })
        .collect()
}

/// The pre-pass both dashboard host surfaces share (the folder-card
/// grid and the tree view): one scan over connections + groups per
/// view call, so the per-card lookups (host / nested counts, filter
/// chips) all hit maps in O(1).
pub(crate) struct DashGridPrePass {
    /// Direct connections per group.
    pub(crate) direct_host_count: std::collections::HashMap<Uuid, usize>,
    /// Child groups per parent.
    pub(crate) nested_group_count: std::collections::HashMap<Uuid, usize>,
    /// Subtree-match set for the tag filter (None when the filter is off).
    pub(crate) tag_filter_groups: Option<std::collections::HashSet<Uuid>>,
}

impl ObsidianSsh {
    pub(crate) fn dash_grid_pre_pass(&self) -> DashGridPrePass {
        let mut direct_host_count: std::collections::HashMap<Uuid, usize> =
            std::collections::HashMap::new();
        for conn in &self.vaults.cur().connections {
            if let Some(cgid) = conn.group_id {
                *direct_host_count.entry(cgid).or_insert(0) += 1;
            }
        }
        let mut nested_group_count: std::collections::HashMap<Uuid, usize> =
            std::collections::HashMap::new();
        for g in &self.vaults.cur().groups {
            if let Some(pgid) = g.parent_id {
                *nested_group_count.entry(pgid).or_insert(0) += 1;
            }
        }
        let tag_filter_groups: Option<std::collections::HashSet<Uuid>> =
            self.groups_containing_filtered_tags();
        DashGridPrePass {
            direct_host_count,
            nested_group_count,
            tag_filter_groups,
        }
    }
}

// Card/section view methods, split into sibling files.
mod empty;
mod group;
mod host;
mod session;
mod tree;

impl ObsidianSsh {
    /// Subtree-match set for the dashboard tag filter: every group
    /// whose descendants include a host carrying at least one selected
    /// tag (ancestors marked in one upward walk). `None` while the filter is
    /// off so callers can skip the check entirely.
    pub(crate) fn groups_containing_filtered_tags(
        &self,
    ) -> Option<std::collections::HashSet<Uuid>> {
        if self.cur_nav().host_filter_tags.is_empty() {
            return None;
        }
        let parent_of: std::collections::HashMap<Uuid, Option<Uuid>> =
            self.vaults.cur().groups.iter().map(|g| (g.id, g.parent_id)).collect();
        let mut set = std::collections::HashSet::new();
        for conn in &self.vaults.cur().connections {
            let matches = conn.tags.iter().any(|tg| {
                self.cur_nav().host_filter_tags.iter().any(|f| f.eq_ignore_ascii_case(tg))
            });
            if !matches {
                continue;
            }
            let mut cur = conn.group_id;
            while let Some(g) = cur {
                if !set.insert(g) {
                    break;
                }
                cur = parent_of.get(&g).copied().flatten();
            }
        }
        Some(set)
    }

    /// Group ids eligible for the parent-group pickers (host editor
    /// combo, Settings default-group). Every folder qualifies, including
    /// empty ones (a freshly created subgroup must be a pickable
    /// destination before anything lives in it).
    pub(crate) fn visible_group_ids(&self) -> std::collections::HashSet<Uuid> {
        self.vaults.cur().groups.iter().map(|g| g.id).collect()
    }

    /// The host cards currently shown on the dashboard, as absolute
    /// indices into `self.vaults.cur().connections`, in display order (group + search +
    /// tag filters applied, then the user's sort). Shared by the
    /// grid renderer and the keyboard-selection navigation so Tab / arrows
    /// move through exactly what's on screen.
    pub(crate) fn dashboard_host_order(&self) -> Vec<usize> {
        let at_root = self.cur_nav().active_group.is_none();
        let flatten = self.vaults.cur().flatten_hosts && at_root;
        let search_lower = self.cur_nav().host_search.to_lowercase();
        let mut host_order: Vec<usize> = (0..self.vaults.cur().connections.len())
            .filter(|&i| {
                let conn = &self.vaults.cur().connections[i];
                if let Some(gid) = self.cur_nav().active_group {
                    if conn.group_id != Some(gid) {
                        return false;
                    }
                } else if conn.group_id.is_some() && !flatten {
                    return false;
                }
                if !crate::util::host_matches_search(
                    conn,
                    self.effective_username(conn).as_deref(),
                    &search_lower,
                ) {
                    return false;
                }
                if !self.cur_nav().host_filter_tags.is_empty()
                    && !conn.tags.iter().any(|tg| {
                        self.cur_nav().host_filter_tags.iter().any(|f| f.eq_ignore_ascii_case(tg))
                    })
                {
                    return false;
                }
                true
            })
            .collect();
        self.vaults.cur().hosts_sort.sort_items(
            &mut host_order,
            |&i| self.vaults.cur().connections[i].label.clone(),
            |&i| self.vaults.cur().connections[i].created_at,
        );
        host_order
    }

    /// The host rows on screen in the ACTIVE view mode, in display
    /// order: the tree walk's host entries in Tree mode (nested folders
    /// included, collapsed ones not), `dashboard_host_order` otherwise.
    /// Every selection verb reads the hosts through this, so a batch
    /// acts on exactly the rows the user can see and pick.
    pub(crate) fn dashboard_visible_host_order(&self) -> Vec<usize> {
        if self.vaults.cur().prefs.host_view_mode == crate::state::HostViewMode::Tree {
            self.dashboard_tree_host_order()
        } else {
            self.dashboard_host_order()
        }
    }

    /// The bar over the grid while the selection is live: the count,
    /// "Connect", "Move to group" and "Delete" for the selection, then
    /// select every visible host, clear. Its buttons are toolbar items on
    /// the keyboard ring, recorded here in visual order after the toolbar
    /// proper.
    ///
    /// The order reads WHICH hosts first, then what to DO with them (the
    /// accent CTA among them - connecting is what a batch of hosts is
    /// for), then the one that cannot be undone: select all, clear,
    /// connect, move to group, delete. The gaps between all five are the
    /// same width; the colors carry the grouping - neutral for the two
    /// that build and drop the selection, accent for the CTA, error for
    /// the destructive one.
    ///
    /// It shows while anything is selected AND while the multi-select
    /// mode is on with nothing selected yet: the bar is how the mode
    /// spells out what it is for, so entering the mode must not leave
    /// the user with a grid of suddenly-inert cards and no explanation.
    /// Every verb that needs a selection (Connect, Move to group,
    /// Delete, Clear) is then disabled rather than absent, so the row
    /// never reflows under the cursor; Select all stays live, being the
    /// one thing to do with an empty selection.
    ///
    /// Chrome: a bare row and one hairline under it in the theme's border
    /// color - no surface, no rounded box. The bar is chrome, not content,
    /// and the same material as the toolbar it hangs from; drawn as a
    /// filled card it read as one more thing floating over the grid, and
    /// outlined in the accent it shouted over the very cards it was
    /// describing. The accent lives in the count's glyph and text and in
    /// Connect, where it points at something.
    fn dashboard_selection_bar(&self) -> Element<'_, Message> {
        // The count and the Move button read the same list every other
        // verb does (`selected_hosts_in_view_order`), so the bar never
        // counts a host a batch would skip.
        let (selected, hidden) = self.selected_hosts_split();
        let n = selected.len();
        let any = n > 0;
        // The count, led by the same glyph the toolbar's mode button
        // wears: the bar says what it is counting before the words are
        // read, and it costs no fill - the accent stays in the text and in
        // the one button that carries an action the user came here for.
        let count = dir_row(vec![
            iced_fonts::lucide::list_checks()
                .size(12)
                .color(ObsidianSshColors::t().accent)
                .boxed(),
            Space::new().width(6).boxed(),
            text(t("selection_count").replace("{n}", &n.to_string()))
                .size(13)
                .color(ObsidianSshColors::t().accent)
                .boxed(),
        ])
        .align_y(iced::Alignment::Center);
        // Selected hosts the view does not show (a search, or a folded
        // folder in Tree mode; the multi-select mode keeps them): said
        // next to the count, so a batch never acts on something the user
        // cannot see without being told.
        let count = if hidden > 0 {
            dir_row(vec![
                count.boxed(),
                Space::new().width(8).boxed(),
                text(t("selection_hidden").replace("{n}", &hidden.to_string()))
                    .size(12)
                    .color(ObsidianSshColors::t().text_muted)
                    .boxed(),
            ])
        } else {
            count
        }
        .align_y(iced::Alignment::Center);
        // The row reads left to right as: WHICH hosts (the two that build
        // the selection), then what to DO with them, then the one that
        // cannot be undone. The gaps are all the same width - the order
        // and the colors carry the grouping, not the spacing.
        let all_btn = self.keynav_toolbar_slot(
            crate::keynav::ToolbarItem::SelectionAll,
            crate::widgets::styled_button_owned(
                t("select_all").to_string(),
                Some(Message::Tabs(TabsMessage::SelectionSelectAll)),
                ObsidianSshColors::t().bg_hover,
            ),
        );
        let clear_btn = self.keynav_toolbar_slot(
            crate::keynav::ToolbarItem::SelectionClear,
            crate::widgets::styled_button_owned(
                t("deselect_all").to_string(),
                any.then_some(Message::Tabs(TabsMessage::SelectionClear)),
                ObsidianSshColors::t().bg_hover,
            ),
        );
        let connect_btn = self.keynav_toolbar_slot(
            crate::keynav::ToolbarItem::SelectionConnect,
            crate::widgets::styled_button_owned(
                t("connect").to_string(),
                any.then_some(Message::Tabs(TabsMessage::SelectionConnect)),
                ObsidianSshColors::t().accent,
            ),
        );
        let move_btn = self.keynav_toolbar_slot(
            crate::keynav::ToolbarItem::SelectionMove,
            crate::widgets::styled_button_owned(
                t("move_to_group").to_string(),
                any.then_some(Message::Tabs(TabsMessage::MoveHostsPick(selected))),
                ObsidianSshColors::t().bg_hover,
            ),
        );
        // Destructive, so it wears the app's error red and closes the
        // row. It asks before it deletes.
        let delete_btn = self.keynav_toolbar_slot(
            crate::keynav::ToolbarItem::SelectionDelete,
            crate::widgets::styled_button_owned(
                t("delete").to_string(),
                any.then_some(Message::Tabs(TabsMessage::SelectionDelete)),
                ObsidianSshColors::t().error,
            ),
        );
        let bar = container(
            dir_row(vec![
                count.boxed(),
                Space::new().width(Length::Fill).boxed(),
                all_btn,
                Space::new().width(8).boxed(),
                clear_btn,
                Space::new().width(8).boxed(),
                connect_btn,
                Space::new().width(8).boxed(),
                move_btn,
                Space::new().width(8).boxed(),
                delete_btn,
            ])
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        // No horizontal padding of its own: the wrapper in
        // `dashboard_main_content` puts the band on the grid's own 24px
        // gutter, so the count starts exactly under the card grid's
        // leading edge and the buttons end under the toolbar's cluster.
        .padding(Padding { top: 8.0, right: 0.0, bottom: 8.0, left: 0.0 });
        // Under it, one hairline in the theme's border color - the whole
        // of the bar's chrome. `Border` applies to all four sides, so a
        // dedicated separator is how a single edge is drawn (the status
        // bar's top edge is the same trick).
        let hairline = container(Space::new().height(1))
            .width(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(ObsidianSshColors::t().border)),
                ..Default::default()
            });
        column![bar, hairline].width(Length::Fill).boxed()
    }

    /// True on a first-run vault: nothing saved anywhere, so the
    /// dashboard renders `dashboard_empty_state` (no toolbar, no
    /// search field). Read outside the view too, by
    /// `active_view_search_id`, so the keyboard never tries to focus a
    /// search field this screen doesn't build.
    pub(crate) fn dashboard_is_empty(&self) -> bool {
        self.vaults.cur().connections.is_empty() && self.vaults.cur().groups.is_empty() && self.vaults.cur().session_groups.is_empty()
    }

    pub(super) fn dashboard_main_content(&self) -> Element<'_, Message> {
        if self.dashboard_is_empty() {
            // Nothing navigable; keep the keyboard order in sync. The
            // empty state builds no toolbar, so it resets that
            // recording itself.
            self.keynav_clear_content();
            return self.dashboard_empty_state();
        }

        let toolbar = self.dashboard_toolbar();

        // ── Search bar ──
        // The host search lives in the dashboard toolbar
        // (`vault_search_field`) now, so the legacy full-width bar here
        // collapses to a zero-height spacer.
        let search_bar: Element<'_, Message> = Space::new().boxed();

        // The host editor's validation error renders inside the
        // editor panel itself (`host_panel::view_host_panel`) right
        // above the Save button. Slot reserved for future list-level
        // statuses.
        let status: Element<'_, Message> = Space::new().boxed();
        let at_root = self.cur_nav().active_group.is_none();
        let flatten = self.vaults.cur().flatten_hosts && at_root;

        // Tree mode builds its own depth-aware walk (grid/tree.rs);
        // the flat collectors stay empty so nothing below double
        // renders.
        let tree_mode = self.vaults.cur().prefs.host_view_mode == crate::state::HostViewMode::Tree;
        let (group_cards, host_cards) = if tree_mode {
            (Vec::new(), Vec::new())
        } else {
            (self.dashboard_group_cards(), self.dashboard_host_cards())
        };

        // Column count adapts to current window width minus the visible
        // chrome (left nav + optional right panel + horizontal padding).
        // Re-derived on every view() so resizing the window or toggling
        // the side panel reflows the cards into the new column count.
        let nav_width = self.vault_rail_width();
        let panel_open = self.vaults.cur().panels.host_panel;
        let panel_width = if panel_open { self.vaults.cur().panel_width } else { 0.0 };
        // A side-docked tab strip (issue #87) narrows the content band
        // like the other grids; without it the math yields one column
        // too many and the card row clips at the edge.
        let available = (self.cur_window_size().width
            - nav_width
            - panel_width
            - self.side_strip_reserve()
            - 48.0)
            .max(0.0);
        // List and tree modes force a single column; otherwise the
        // grid reflows responsively to the available width.
        let cols = if self.vaults.cur().prefs.host_view_mode == crate::state::HostViewMode::Grid {
            card_grid_columns(available, CARD_WIDTH, 12.0)
        } else {
            1
        };

        // Section header (Termius-style "Groups" / "Hosts" labels).
        // Only rendered in flatten mode at root, where the user can
        // see both lists side-by-side.
        // Wrap the label in a width-fill container so it lines up
        // with the card grid's leading edge. The plain `text` widget
        // shrinks to content and the column's `align_x` pushes the
        // shrunk box around in a way that doesn't always coincide
        // with the card border; making the container Fill anchors it
        // explicitly to the leading edge of the row. Also mirrors
        // Keychain's section_title vertical padding (4 px top, 8 px
        // bottom) so the section labels sit at the same offset
        // relative to the search bar as they do in the Keychain.
        let section_header = |label_key: &'static str| -> Element<'_, Message> {
            container(
                container(
                    text(t(label_key))
                        .size(14)
                        .color(ObsidianSshColors::t().text_muted),
                )
                .width(Length::Fill)
                .align_x(crate::widgets::dir_align_x()),
            )
            .padding(Padding { top: 4.0, right: 0.0, bottom: 8.0, left: 0.0 })
            .boxed()
        };

        // Saved session groups that live in the current folder. The
        // enumerate index is absolute (into `self.vaults.cur().session_groups`), which is
        // what Open/Edit/Delete expect. Tree mode emits them inside
        // its own walk, at their folder's level.
        let session_group_cards: Vec<(Element<'_, Message>, Color, DashNavItem)> = if tree_mode
        {
            Vec::new()
        } else {
            self.vaults.cur().session_groups
                .iter()
                .enumerate()
                .filter(|(_, g)| g.group_id == self.cur_nav().active_group)
                .map(|(i, g)| {
                    let (el, color) = self.session_group_card(i, g);
                    (el, color, DashNavItem::SessionGroup(i))
                })
                .collect()
        };

        // Per the `card_accent_glass` setting: glass on → each card gets
        // the soft per-colour wash; off → cards stay pure (just the
        // element, no overlay).
        let glass = self.vaults.cur().prefs.card_accent_glass;
        let selected = match self.kn().selected_in(crate::keynav::FocusZone::Content) {
            Some(crate::keynav::NavItem::Dash(d)) => Some(d),
            _ => None,
        };

        // List mode (cols == 1) renders History-style rows: full-width
        // rounded cards with a small gap. Grid mode keeps the roomier
        // 12px gutters. Tree mode is dense on purpose - a hairline gap
        // keeps its indent guide lines reading as near-continuous.
        let gap = match self.vaults.cur().prefs.host_view_mode {
            crate::state::HostViewMode::Grid => 12.0,
            crate::state::HostViewMode::List => 8.0,
            crate::state::HostViewMode::Tree => 2.0,
        };

        let mut content_rows: Vec<Element<'_, Message>> = Vec::new();
        let tree_cards = if tree_mode { self.dashboard_tree_cards() } else { Vec::new() };
        // Record the keyboard-navigation order as visual rows (groups rows
        // then hosts rows, each chunked to the column count) so the key
        // handler can move the selection in 2-D without re-deriving the
        // group order. Groups + session groups share the groups section.
        // Tree mode is one linear section instead: one item per row,
        // in exactly the walk's display order.
        if tree_mode {
            self.keynav_set_content_sections(vec![tree_cards
                .iter()
                .map(|(_, _, n)| vec![crate::keynav::NavItem::Dash(*n)])
                .collect()]);
        } else {
            let cw = cols.max(1);
            let group_nav: Vec<DashNavItem> = group_cards
                .iter()
                .chain(session_group_cards.iter())
                .map(|(_, _, n)| *n)
                .collect();
            let host_nav: Vec<DashNavItem> =
                host_cards.iter().map(|(_, _, n)| *n).collect();
            let dash_row = |c: &[DashNavItem]| {
                c.iter().map(|&n| crate::keynav::NavItem::Dash(n)).collect()
            };
            // Two Tab sections (Groups, then Hosts); arrows still flow
            // continuously across both.
            self.keynav_set_content_sections(vec![
                group_nav.chunks(cw).map(dash_row).collect(),
                host_nav.chunks(cw).map(dash_row).collect(),
            ]);
        }

        // Search filtered everything out but the query parses as an
        // ad-hoc `user@host[:port]` target: offer a centered quick-connect
        // card instead of a bare no-results gap (mirrors the picker row
        // and the toolbar's "Enter to connect" hint).
        if group_cards.is_empty()
            && session_group_cards.is_empty()
            && host_cards.is_empty()
            && tree_cards.is_empty()
            && !self.cur_nav().host_search.trim().is_empty()
            && let Some(conn) = self.dashboard_quick_connect_target(&self.cur_nav().host_search)
        {
            content_rows.push(self.quick_connect_card(conn));
        }
        // Selection bar (issue #230): count, connect, move, delete,
        // select all, clear. Built HERE - so its buttons land on the
        // keyboard ring right after the toolbar's own - but placed
        // OUTSIDE the scroller, pinned under the toolbar: the actions
        // belong to the selection, and a selection outlives scrolling. A
        // host picked at the top of a long list must still have its
        // Connect in reach from the bottom of it. It is also the
        // multi-select mode's own surface: entering the mode shows it
        // with nothing selected yet.
        let selection_bar: Element<'_, Message> = if !self.cur_nav().dash_selection.is_empty()
            || self.cur_nav().dash_multi_select
        {
            container(self.dashboard_selection_bar())
                .width(Length::Fill)
                // The 24px band the grid content rides in, plus the gap
                // that used to sit between the bar and the first card
                // row when the bar scrolled with it.
                .padding(Padding { top: 0.0, right: 24.0, bottom: 12.0, left: 24.0 })
                // The bar's buttons are fixed-width, so a window narrow
                // enough (the minimum is 800px wide, and an expanded rail
                // leaves the band under 600) leaves the row wider than
                // the surface it rides on. Inside the scroller the
                // overflow was clipped for free; pinned out here it would
                // be drawn over whatever sits to the right of the band,
                // so clip it at the same edge the scroller did.
                .clip(true)
                .boxed()
        } else {
            Space::new().height(0).boxed()
        };
        if tree_mode {
            let washed =
                apply_card_wash(tree_cards, glass, selected, self.kn().ring_bounds.clone());
            content_rows.push(distribute_card_grid(washed, 1, gap, gap));
        } else if flatten {
            // Session groups live under the same "Groups" section as host
            // groups (they're both group-shaped entries), instead of a
            // separate "Session Groups" section. Host groups come first.
            if !group_cards.is_empty() || !session_group_cards.is_empty() {
                // `section_header` already carries its own 4/8 vertical
                // padding (mirroring Keychain), so no extra Space below.
                content_rows.push(section_header("groups_section"));
                let mut grouped = group_cards;
                grouped.extend(session_group_cards);
                let grouped = apply_card_wash(grouped, glass, selected, self.kn().ring_bounds.clone());
                content_rows.push(distribute_card_grid(grouped, cols, gap, gap));
                content_rows.push(Space::new().height(20).boxed());
            }
            if !host_cards.is_empty() {
                content_rows.push(section_header("hosts_section"));
                let host_cards = apply_card_wash(host_cards, glass, selected, self.kn().ring_bounds.clone());
                content_rows.push(distribute_card_grid(host_cards, cols, gap, gap));
            }
        } else {
            // Legacy: groups, then session groups, then hosts, in one grid.
            let mut combined = group_cards;
            combined.extend(session_group_cards);
            combined.extend(host_cards);
            let combined = apply_card_wash(combined, glass, selected, self.kn().ring_bounds.clone());
            content_rows.push(distribute_card_grid(combined, cols, gap, gap));
        }

        // Each grid row holds up to 3 fixed-width cards; once the row
        // is narrower than the available column width, the column's
        // cross-axis alignment decides whether the row sticks to the
        // leading or trailing edge. Use `dir_align_x()` so cards begin
        // from the trailing edge of the LTR layout (= leading edge of
        // the RTL layout), keeping them aligned with the toolbar title
        // / actions on the same side.
        // The column needs `Length::Fill` for `align_x` to have any
        // slack to align inside, without it the column shrinks to
        // content and the rows still hug the leading edge.
        let grid = scrollable(
            column(content_rows)
                .width(Length::Fill)
                .padding(Padding { top: 0.0, right: 24.0, bottom: 24.0, left: 24.0 })
                .align_x(crate::widgets::dir_align_x()),
        )
        .id(iced::widget::Id::new("dashboard-grid-scroll"))
        .height(Length::Fill);

        let main_content = column![toolbar, search_bar, status, selection_bar, grid]
            .width(Length::Fill)
            .height(Length::Fill);
        main_content.boxed()
    }

    /// Centered ad-hoc quick-connect card, shown when the host search
    /// matches nothing but parses as a `user@host[:port]` target. The
    /// whole card is a button (hover + press feedback per convention)
    /// dispatching `QuickConnect`; nothing is saved to the vault.
    fn quick_connect_card(
        &self,
        conn: obsidianssh_core::models::Connection,
    ) -> Element<'_, Message> {
        let label = conn.label.clone();
        // Protocol badges, for a line that named no `scheme://`. They
        // live OUTSIDE the card button (a button inside a button never
        // gets its own press), directly under it.
        let badges: Option<Element<'_, Message>> = self
            .quick_connect_badges(&self.cur_nav().host_search)
            .map(|(options, selected)| {
                let chips: Vec<Element<'_, Message>> = options
                    .into_iter()
                    .map(|p| {
                        let on = p == selected;
                        let bg = if on {
                            ObsidianSshColors::t().accent
                        } else {
                            ObsidianSshColors::t().bg_hover
                        };
                        let fg = if on {
                            crate::theme::contrast_text_for(bg)
                        } else {
                            ObsidianSshColors::t().text_secondary
                        };
                        button(text(p.to_string()).size(11).color(fg))
                            .on_press(Message::Ssh(SshMessage::QuickConnectProtocolPicked(p)))
                            .padding(Padding { top: 3.0, right: 10.0, bottom: 3.0, left: 10.0 })
                            .style(move |_, status| button::Style {
                                background: Some(Background::Color(match status {
                                    BtnStatus::Hovered | BtnStatus::Pressed => {
                                        ObsidianSshColors::t().bg_selected
                                    }
                                    _ => bg,
                                })),
                                border: Border {
                                    radius: Radius::from(4.0),
                                    ..Default::default()
                                },
                                text_color: fg,
                                ..Default::default()
                            })
                            .boxed()
                    })
                    .collect();
                container(crate::widgets::dir_row(chips).spacing(6))
                    .padding(Padding { top: 10.0, right: 0.0, bottom: 0.0, left: 0.0 })
                    .center_x(Length::Fill)
                    .boxed()
            });
        let card = button(
            iced::widget::Column::<iced::Element<'_, _>>::with_children(vec![
                iced_fonts::lucide::zap()
                    .size(28)
                    .color(ObsidianSshColors::t().accent)
                    .boxed(),
                Space::new().height(10).boxed(),
                // The protocol is named whenever it is not the default
                // one, so a Telnet or Serial dial says so before Enter
                // rather than after it opens.
                text(
                    match conn.protocol
                        == obsidianssh_core::models::connection::ConnectionProtocol::Ssh
                    {
                        true => format!("{}: {}", t("quick_connect"), label),
                        false => format!(
                            "{} ({}): {}",
                            t("quick_connect"),
                            conn.protocol,
                            label
                        ),
                    },
                )
                .size(15)
                .color(ObsidianSshColors::t().text_primary)
                .boxed(),
                Space::new().height(4).boxed(),
                text(t("quick_connect_not_saved"))
                    .size(12)
                    .color(ObsidianSshColors::t().text_muted)
                    .boxed(),
                Space::new().height(8).boxed(),
                container(
                    text(t("quick_connect_hint"))
                        .size(11)
                        .color(ObsidianSshColors::t().accent),
                )
                .padding(Padding { top: 2.0, right: 8.0, bottom: 2.0, left: 8.0 })
                .style(|_| container::Style {
                    background: Some(Background::Color(ObsidianSshColors::t().bg_hover)),
                    border: Border {
                        radius: Radius::from(4.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .boxed(),
            ])
            .align_x(iced::alignment::Horizontal::Center),
        )
        .on_press(Message::Ssh(SshMessage::QuickConnect(Box::new(
            crate::state::QuickConnectEntry::bare(conn),
        ))))
        .padding(Padding { top: 24.0, right: 32.0, bottom: 24.0, left: 32.0 })
        .style(|_: &iced::Theme, status| {
            let (bg, bc) = match status {
                BtnStatus::Hovered => (ObsidianSshColors::t().bg_hover, ObsidianSshColors::t().accent),
                BtnStatus::Pressed => {
                    (ObsidianSshColors::t().bg_selected, ObsidianSshColors::t().accent)
                }
                _ => (ObsidianSshColors::t().bg_surface, ObsidianSshColors::t().border),
            };
            button::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    radius: Radius::from(10.0),
                    color: bc,
                    width: 1.0,
                },
                ..Default::default()
            }
        });
        let mut stack = iced::widget::Column::<iced::Element<'_, _>>::new().align_x(iced::alignment::Horizontal::Center);
        stack = stack.push(card.boxed());
        if let Some(badges) = badges {
            stack = stack.push(badges);
        }
        container(stack)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding { top: 32.0, right: 0.0, bottom: 0.0, left: 0.0 })
            .boxed()
    }
}

