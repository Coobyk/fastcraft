fn fastcraft_placed(
    game: &mut crate::game::Game<'_>,
    entry: usize,
    all: bool,
    placed: bool,
) -> bool {
    // Only a Ctrl click crafts; every other click is the vanilla placement.
    if !placed || !game.has_control_down() || game.active_book() != Some(BookType::Crafting) {
        return placed;
    }
    let Some((expected, display)) =
        crate::game::recipe_book::fastcraft_book::expected(game, entry)
    else {
        return placed;
    };
    if expected.is_empty() {
        return placed;
    }
    // The drop key (Q) held at the click drops the results instead of
    // moving them to the inventory.
    let drop = game
        .physical_keys
        .contains(&winit::keyboard::KeyCode::KeyQ);
    crate::game::fastcraft::arm(expected, display, all, drop);
    // A local world has the grid filled already; a server's answer lands on
    // a later tick (`tick`).
    crate::game::fastcraft::fire(game);
    placed
}
