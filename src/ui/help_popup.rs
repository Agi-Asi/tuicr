use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use unicode_width::UnicodeWidthStr;

use crate::app::App;
use crate::ui::styles;

/// Width, in terminal cells, reserved for the key column of a help row.
/// This is the width of the longest key (including its two-space gutter), so
/// every description in the popup begins in the same column.
const HELP_KEY_COL_WIDTH: usize = 32;

/// Build one aligned help row: the key is padded to a fixed display width so
/// every description in the popup starts in the same column.
fn help_row(key: &str, description: &str) -> Line<'static> {
    let key_width = key.width();
    let padded_key = if key_width < HELP_KEY_COL_WIDTH {
        format!("{key}{}", " ".repeat(HELP_KEY_COL_WIDTH - key_width))
    } else {
        // Long keys already fill the column; a single space keeps the
        // description visually separated without shifting the rest.
        format!("{key} ")
    };
    Line::from(vec![
        Span::styled(
            padded_key,
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(description.trim_start().to_string()),
    ])
}

pub fn render_message_details(frame: &mut Frame, app: &mut App) {
    let Some(message) = app.message.as_ref() else {
        return;
    };
    let content = message.content.clone();
    let theme = &app.theme;
    let area = frame.area();

    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(styles::panel_style(theme))
        .border_style(styles::border_style(theme, true));
    let inner = block.inner(area);

    let paragraph = Paragraph::new(content.as_str())
        .style(
            Style::default()
                .fg(theme.message_error_fg)
                .bg(theme.panel_bg),
        )
        .wrap(Wrap { trim: false });
    app.help_state.total_lines = paragraph.line_count(inner.width);
    app.help_state.viewport_height = inner.height as usize;
    let max_offset = app
        .help_state
        .total_lines
        .saturating_sub(app.help_state.viewport_height);
    app.help_state.scroll_offset = app.help_state.scroll_offset.min(max_offset);
    let scroll_indicator = match (
        app.help_state.scroll_offset > 0,
        app.help_state.scroll_offset < max_offset,
    ) {
        (true, true) => " ↑↓",
        (true, false) => " ↑",
        (false, true) => " ↓",
        (false, false) => "",
    };
    frame.render_widget(
        block.title(format!(
            " Messages (j/k scroll, q/Esc close){scroll_indicator} "
        )),
        area,
    );
    frame.render_widget(
        paragraph.scroll((
            app.help_state.scroll_offset.min(u16::MAX as usize) as u16,
            0,
        )),
        inner,
    );
}

pub fn render_help(frame: &mut Frame, app: &mut App) {
    let theme = &app.theme;
    // Center over the diff pane (matches the submit-modal anchoring) so the
    // file list doesn't tug the popup's visual centre off to one side. Fall
    // back to the full frame when no diff area is laid out yet.
    let anchor = app.diff_area.unwrap_or(frame.area());
    let area = centered_rect(80, 90, anchor);

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help (j/k scroll, / search) - Press ? or Esc to close ")
        .borders(Borders::ALL)
        .style(styles::popup_style(theme))
        .border_style(styles::border_style(theme, true));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  j/k       ", "Scroll down/up"),
        help_row("  Ctrl-e/y  ", "Scroll view down/up"),
        help_row("  Ctrl-d/u  ", "Half page down/up"),
        help_row("  Ctrl-f/b  ", "Full page down/up"),
        help_row("  g/G       ", "Go to first/last file"),
        help_row("  {N}G      ", "Go to source line N in current file"),
        help_row("  {/}       ", "Jump to prev/next file"),
        help_row("  [/]       ", "Jump to prev/next hunk"),
        help_row("  m/M       ", "Jump to next/previous comment"),
        help_row("  /         ", "Search within diff (case-insensitive)"),
        help_row("  n/N       ", "Next/prev search match (wraps)"),
        help_row("  Esc       ", "Clear search highlighting"),
        help_row("  Enter     ", "Expand/collapse context (20 lines)"),
        help_row("  S-Enter   ", "Expand/collapse all hidden context"),
        help_row("  Tab/S-Tab ", "Toggle focus next/previous panel"),
        help_row(
            &format!("  {}h/{}l     ", app.leader_key, app.leader_key),
            "Focus file list/diff",
        ),
        help_row(
            &format!("  {}k/{}j     ", app.leader_key, app.leader_key),
            "Move focus up/down between panes",
        ),
        help_row(
            &format!("  {}e        ", app.leader_key),
            "Toggle file list visibility",
        ),
        help_row(
            &format!("  {}s        ", app.leader_key),
            "Toggle commit selector visibility (also `:set commits!`)",
        ),
        help_row(
            &format!("  {}f        ", app.leader_key),
            "Toggle single-file view (also `:focus` / `:f`)",
        ),
        help_row("  h/l       ", "Scroll diff left/right (or ←/→)"),
        help_row("  h/← at 0 ", "Reveal + focus file list"),
        help_row("  l/→ in list", "Hide list + focus diff"),
        Line::from(""),
        Line::from(Span::styled(
            format!("Single-file view (`:focus`, `:f`, {}f)", app.leader_key),
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(Span::raw(
            "  One file at a time. j/k walks across files, ]/[ walks across hunks. Default in `--all-files`.",
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Commit Selector (multi-commit reviews)",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        Line::from(Span::raw(
            "  ✓ marks commits covered by your latest submitted review.",
        )),
        help_row("  j/k       ", "Navigate commits"),
        help_row("  Space/Enter", "  Toggle commit selection (updates diff)"),
        help_row("  (/)       ", "Cycle through individual commits"),
        help_row("  Esc       ", "Return focus to diff"),
        Line::from(""),
        Line::from(Span::styled(
            "Review Target Selector",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  Tab/S-Tab ", "Switch Local / Pull Requests tab"),
        help_row("  j/k       ", "Move row"),
        help_row("  Space     ", "Toggle local commit selection (no-op on PR tab)"),
        help_row("  Enter     ", "Open selected target or load more"),
        help_row("  /         ", "Local filter for current tab"),
        help_row("  r         ", "Toggle PRs requesting your review"),
        help_row("  Esc       ", "Return to the diff"),
        Line::from(""),
        Line::from(Span::styled(
            "Submit Action Picker (PR mode)",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  j/k       ", "Navigate review events"),
        help_row("  Enter     ", "Submit the selected event"),
        help_row("  Esc       ", "Cancel"),
        Line::from(""),
        Line::from(Span::styled(
            "File Tree",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  Space     ", "Toggle expand directory"),
        help_row("  Enter     ", "Expand dir / Jump to file"),
        help_row("  o         ", "Expand all directories"),
        help_row("  O         ", "Collapse all directories"),
        help_row(
            "  i         ",
            "Filter to files matching a regex (hides others from tree + diff)",
        ),
        help_row("  e         ", "Filter out files matching a regex"),
        help_row("  I/E       ", "Clear the include/exclude filter"),
        help_row("  /         ", "Search file paths; n/N step matches (selection only)"),
        Line::from(Span::raw(
            "  Patterns are case-insensitive and match the whole relative path.",
        )),
        Line::from(Span::raw(
            "  :set noreviewed hides files already marked reviewed.",
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Comment Navigator",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  j/k       ", "Navigate comments"),
        help_row("  h/l       ", "Scroll comment rows left/right"),
        help_row("  Enter     ", "Jump to selected comment"),
        Line::from(""),
        Line::from(Span::styled(
            "Review Actions",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  r         ", "Toggle file reviewed"),
        help_row("  R         ", "Toggle hunk reviewed"),
        help_row("  c         ", "Add line comment"),
        help_row("  C         ", "Add file comment"),
        help_row(&format!("  {}c        ", app.leader_key), "Add review comment"),
        help_row("  i         ", "Edit comment at cursor"),
        help_row("  dd        ", "Delete comment at cursor"),
        help_row("  y         ", "Yank: mouse selection if any, else review to clipboard"),
        help_row("  Y         ", "Copy comment at cursor to clipboard"),
        help_row("  e         ", "Open focused file in $EDITOR"),
        help_row("  v/V       ", "Enter visual mode for range comments"),
        Line::from(""),
        Line::from(Span::styled(
            "Visual Mode",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  j/k       ", "Extend selection down/up"),
        help_row("  c/Enter   ", "Create comment for selected range"),
        help_row("  Esc/v/V   ", "Cancel visual selection"),
        Line::from(""),
        Line::from(Span::styled(
            "Comment Mode",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  Tab/S-Tab ", "Cycle comment type next/previous"),
        help_row("  Enter     ", "Save comment"),
        help_row("  Ctrl-S    ", "Save comment"),
        help_row("  Shift-Enter/Alt-Enter/Ctrl-J", "Insert newline"),
        help_row("  Ctrl-A/E  ", "Line start/end"),
        help_row("  Ctrl/Alt-Left/Right", "Word left/right"),
        help_row("  Cmd-Left/Right", "Line start/end (macOS)"),
        help_row("  Esc/Ctrl-C", "Cancel"),
        help_row("  comment_vim", if app.comment_vim_enabled {
                "Vim ON (i/a:insert Esc:normal hjkl dd/ciw/x u; S-Enter:save S-Esc:discard :w/:q)"
            } else {
                "Set comment_vim=true (or :vim) for vim modal editing"
            }),
        Line::from(""),
        Line::from(Span::styled(
            "Commands",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )),
        Line::from(""),
        help_row("  :{N}      ", "Jump to new-side line N in current file"),
        help_row("  :o{N}     ", "Jump to old-side line N in current file (matches deletions)"),
        help_row("  Tab       ", "Complete or cycle command names in the : prompt"),
        help_row("  :w        ", "Save review session"),
        help_row(
            "  :e        ",
            "Reload diff files and comments (in PR mode: refetch PR; may switch session)",
        ),
        help_row("  :edit     ", "Open focused file in $EDITOR"),
        help_row("  :clip     ", "Copy review to clipboard"),
        help_row("  :copy-url ", "Copy the open PR URL to clipboard (PR mode)"),
        help_row("  :wrap     ", "Toggle line wrap in diff view"),
        help_row("  :help     ", "Open this help screen"),
        help_row("  :messages ", "Open full details for the current error"),
        help_row("  :summary  ", "Pending comments; j/k select, Enter jumps"),
        help_row("  :stage    ", "Stage reviewed files"),
        help_row("  :diff     ", "Toggle unified/side-by-side diff view"),
        help_row("  :focus    ", &format!(
                "Toggle single-file view (alias `:f`, {}f)",
                app.leader_key
            )),
        help_row("  :targets  ", "Open the review target selector"),
        help_row("  :commits  ", "Open the selector on Local (commits, staged/unstaged)"),
        help_row("  :prs      ", "Open the selector on Pull Requests"),
        help_row("  :comments unresolved", "  Show unresolved remote comments (PR mode, default)"),
        help_row("  :comments all", "  Show all remote comments incl. resolved/outdated"),
        help_row("  :comments hide", "  Hide remote comments in PR mode"),
        help_row(
            "  :submit       ",
            "Pick a review event from a menu (Comment / Approve / Request changes / Draft)",
        ),
        help_row(
            "  :submit comment",
            "Submit a COMMENT review (skips the picker, shows confirm modal)",
        ),
        help_row("  :submit approve", "  Submit an APPROVE review to the forge"),
        help_row("  :submit request-changes", "  Submit a REQUEST_CHANGES review to the forge"),
        help_row("  :submit draft", "  Push a pending (draft) review to the forge"),
        help_row("  :set relativenumber[!]", "  Enable/toggle relative rendered-row numbers"),
        help_row("  :set norelativenumber", "  Disable relative rendered-row numbers"),
        help_row("  :set commits", "  Show inline commit selector"),
        help_row("  :set nocommits", "  Hide inline commit selector"),
        help_row("  :set commits!", "  Toggle inline commit selector"),
        help_row(
            "  :set reviewed",
            "Show files marked reviewed (noreviewed hides, reviewed! toggles)",
        ),
        help_row("  :clear    ", "Clear all comments and reviewed marks"),
        help_row("  :clearc   ", "Clear comments only"),
        help_row("  :q        ", "Quit"),
        help_row("  :wq       ", "Save and quit"),
        help_row("  :version  ", "Show tuicr version"),
        help_row("  :update   ", "Check for updates"),
        Line::from(""),
        help_row("  /         ", "Search within this help (case-insensitive)"),
        help_row("  n/N       ", "Next/previous help search match"),
        help_row("  ?         ", "Toggle this help"),
    ];

    // Update help state with total lines and viewport height
    let total_lines = help_text.len();
    let viewport_height = inner.height as usize;
    app.help_state.total_lines = total_lines;
    app.help_state.viewport_height = viewport_height;
    app.help_state.searchable_lines = help_text
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect()
        })
        .collect();

    // Calculate if we can scroll
    let can_scroll_up = app.help_state.scroll_offset > 0;
    let can_scroll_down = app.help_state.scroll_offset + viewport_height < total_lines;

    // Apply scroll offset
    let visible_lines: Vec<Line> = help_text
        .into_iter()
        .enumerate()
        .skip(app.help_state.scroll_offset)
        .take(viewport_height)
        .map(|(line_idx, line)| {
            if app.help_state.current_match_line == Some(line_idx) {
                line.style(styles::selected_style(theme))
            } else {
                line
            }
        })
        .collect();

    let paragraph = Paragraph::new(visible_lines).style(styles::popup_style(theme));
    frame.render_widget(paragraph, inner);

    // Render scroll indicators
    let indicator_style = styles::help_indicator_style(theme);

    if can_scroll_up {
        let up_indicator = Paragraph::new(Line::from(Span::styled("▲ more", indicator_style)));
        let up_area = Rect {
            x: inner.x + inner.width.saturating_sub(8),
            y: inner.y,
            width: 7,
            height: 1,
        };
        frame.render_widget(up_indicator, up_area);
    }

    if can_scroll_down {
        let down_indicator = Paragraph::new(Line::from(Span::styled("▼ more", indicator_style)));
        let down_area = Rect {
            x: inner.x + inner.width.saturating_sub(8),
            y: inner.y + inner.height.saturating_sub(1),
            width: 7,
            height: 1,
        };
        frame.render_widget(down_indicator, down_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
