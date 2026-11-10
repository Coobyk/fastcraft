// Fastcraft: craft straight from the recipe book, as OneClickCrafting does.
//
// A recipe-book click places the recipe as usual and arms a pending craft
// (`arm`); once the result slot shows the recipe's result the pending craft
// takes it with the same click a Shift-click on it would send. On a server
// the result arrives a few packets after the placement, so the armed craft
// waits on the client tick until it matches or expires.
use std::cell::RefCell;
use std::sync::Arc;

use horizon_client::menu::Input;

use super::Game;

/// One armed craft: the items the recipe's display can show as the result,
/// whether it drops, and whether the click wanted the maximum placed.
struct Pending {
    expected: Vec<Arc<str>>,
    drop: bool,
    all: bool,
    ttl: u32,
}

/// Ticks an armed craft waits for its result before giving up.
const TTL: u32 = 60;

thread_local! {
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

/// Arm a craft for the recipe just placed: `all` is the book's use-max
/// (the Shift click), `drop` the drop key held at it.
pub(super) fn arm(expected: Vec<Arc<str>>, drop: bool, all: bool) {
    PENDING.with(|cell| {
        *cell.borrow_mut() = Some(Pending {
            expected,
            drop,
            all,
            ttl: TTL,
        });
    });
}

/// The client tick (it also runs while a screen holds the game paused):
/// expire the armed craft or fire it.
pub(super) fn tick(game: &mut Game<'_>) {
    let expired = PENDING.with(|cell| {
        cell.borrow_mut().as_mut().is_some_and(|p| {
            p.ttl = p.ttl.saturating_sub(1);
            p.ttl == 0
        })
    });
    if expired {
        PENDING.with(|cell| cell.borrow_mut().take());
        return;
    }
    fire(game);
}

/// Take the armed craft's result when it shows; a result that is not the
/// armed recipe's keeps it waiting, a crafting screen that closed drops it.
pub(super) fn fire(game: &mut Game<'_>) {
    let done = PENDING.with(|cell| {
        let armed = cell.borrow();
        let Some(p) = armed.as_ref() else {
            return true;
        };
        if !game.mod_recipe_screen() {
            return true;
        }
        let Some(stack) = game.slot_view(200) else {
            return false;
        };
        if !p.expected.iter().any(|e| e == &stack.item) {
            return false;
        }
        craft(game, p);
        true
    });
    if done {
        PENDING.with(|cell| cell.borrow_mut().take());
    }
}

/// The crafting result slot is the screen's slot 200 (menu slot 0 of the
/// plain crafting menus): a QuickMove takes every craft the placed recipe
/// holds; a Throw drops the crafted stacks as the drop key's click would.
fn craft(game: &mut Game<'_>, p: &Pending) {
    if p.drop {
        let button = i32::from(p.all);
        if game.remote.is_some() {
            game.menu_click(Some(200), button, Input::Throw);
        } else {
            game.menu_session(|menu, env, to_menu| {
                let Some(slot) = to_menu(200) else {
                    return Some(false);
                };
                Some(
                    menu.click(env, slot as i32, button, Input::Throw)
                        .is_ok(),
                )
            });
        }
    } else {
        game.menu_click(Some(200), 0, Input::QuickMove);
    }
}
