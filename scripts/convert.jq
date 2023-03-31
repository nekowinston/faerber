. | 
map({
  (.metadata.name): (
    [
      .colors.ansi[]?,
      .colors.brights[]?,
      .colors.indexed[]?,
      .colors.background,
      .colors.foreground,
      .colors.cursor_bg,
      .colors.selection_bg,
      .colors.selection_fg,
      .colors.cursor_bg,
      .colors.cursor_fg,
      .colors.cursor_border
    ] |
    map(
      select(. != null)
    ) |
    map(
      select(. | startswith("#"))
    ) |
    unique
  )
}) |
add
