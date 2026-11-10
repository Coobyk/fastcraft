fn fastcraft_placed(
    game: &mut crate::game::Game<'_>,
    entry: usize,
    all: bool,
    placed: bool,
) -> bool {
    // Alt is the bypass: an Alt click places the recipe, nothing more.
    if !placed || game.has_alt_down() || game.active_book() != Some(BookType::Crafting) {
        return placed;
    }
    let expected: Vec<std::sync::Arc<str>> = game
        .recipe_ui
        .book
        .as_ref()
        .and_then(|book| book.entries.get(entry))
        .map(|entry| {
            entry
                .results()
                .iter()
                .map(|result| result.item.clone())
                .collect()
        })
        .unwrap_or_default();
    if expected.is_empty() {
        return placed;
    }
    // The drop key (Q) held at the click drops the results instead of
    // moving them to the inventory.
    let drop = game
        .physical_keys
        .contains(&winit::keyboard::KeyCode::KeyQ);
    crate::game::fastcraft::arm(expected, drop, all);
    // A local world has the grid filled already; a server's answer lands on
    // a later tick (`tick`).
    crate::game::fastcraft::fire(game);
    placed
}
