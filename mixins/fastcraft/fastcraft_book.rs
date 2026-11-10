// Fastcraft's door into the recipe book: the craft loop refills the grid
// with the armed recipe, which the sibling `fastcraft` module cannot reach
// (`place_recipe` and the book are private to this one).
use std::sync::Arc;

use super::{BookType, Game};

/// The result items a book entry's display cycles through, with its display
/// id for `re_place`.
pub(super) fn expected(game: &Game<'_>, entry_index: usize) -> Option<(Vec<Arc<str>>, i32)> {
    let entry = game.recipe_ui.book.as_ref()?.entries.get(entry_index)?;
    Some((
        entry
            .results()
            .iter()
            .map(|result| result.item.clone())
            .collect(),
        entry.display_id,
    ))
}

/// `recipe_book_key`'s repeat: place the recipe this display id still names,
/// filling the grid (`use_max`). False when the book no longer holds it.
pub(crate) fn re_place(game: &mut Game<'_>, display: i32) -> bool {
    if game.active_book() != Some(BookType::Crafting) {
        return false;
    }
    let Some(index) = game
        .recipe_ui
        .book
        .as_ref()
        .and_then(|book| book.entry_of_display(display))
    else {
        return false;
    };
    game.place_recipe(BookType::Crafting, index, true);
    true
}
