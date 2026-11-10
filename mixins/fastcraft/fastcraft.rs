// Fastcraft: craft straight from the recipe book, as OneClickCrafting does.
//
// A Ctrl click on a recipe places it and takes the result; Ctrl+Shift keeps
// placing and taking until the inventory cannot hold another craft. Holding
// the drop key (Q) drops the crafted items: with Ctrl alone the one result,
// with Ctrl+Shift whatever the inventory could not fit.
//
// On a server the result arrives a few packets after the placement, so the
// armed craft waits on the client tick until the result slot shows the
// recipe's item; locally it fires the same call it was armed.
use std::cell::RefCell;
use std::sync::Arc;

use horizon_client::menu::Input;

use super::Game;

/// What a Ctrl click asks for: `Once` takes the one result it placed,
/// `Many` keeps placing and taking until materials or room run out;
/// `drop` sends the crafted stacks out with the drop key's Throw click.
enum Mode {
    Once { drop: bool },
    Many { drop: bool },
}

impl Clone for Mode {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Mode {}

/// The armed craft: the items the recipe's display can show as the result,
/// the display id the next place is re-resolved from, and where the loop is.
struct Pending {
    expected: Vec<Arc<str>>,
    display: i32,
    mode: Mode,
    /// A take was sent; the tick checks whether the result slot drained
    /// (refill the grid) or kept a stack it could not move (inventory full).
    await_take: bool,
    /// `Many` with `drop` went past a full inventory: further rounds drop.
    draining: bool,
    ttl: u32,
}

/// Ticks an armed craft waits for its result before giving up.
const TTL: u32 = 60;

thread_local! {
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

/// Arm a craft for the recipe just placed: `all` is the book's use-max
/// (the Shift click), `drop` the drop key held at it.
pub(super) fn arm(expected: Vec<Arc<str>>, display: i32, all: bool, drop: bool) {
    let mode = if all {
        Mode::Many { drop }
    } else {
        Mode::Once { drop }
    };
    PENDING.with(|cell| {
        *cell.borrow_mut() = Some(Pending {
            expected,
            display,
            mode,
            await_take: false,
            draining: false,
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

/// Drive the armed craft: waits for the expected result to show, takes it,
/// and for `Many` keeps placing and taking until it cannot go on.
pub(super) fn fire(game: &mut Game<'_>) {
    let Some((expected, display, mode, await_take, draining)) = PENDING.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|p| (p.expected.clone(), p.display, p.mode, p.await_take, p.draining))
    }) else {
        return;
    };
    // The screen that armed is gone: the craft is cancelled.
    if !game.mod_recipe_screen() {
        return clear();
    }
    let shown = game.slot_view(200);
    let expected_here = shown
        .as_ref()
        .is_some_and(|s| expected.iter().any(|e| e == &s.item));

    if await_take {
        if expected_here {
            // A stack stayed in the result: the inventory took all it could.
            match mode {
                Mode::Many { drop: true } => {
                    throw(game, 1);
                    touch(|p| p.draining = true);
                }
                _ => clear(),
            }
        } else if shown.is_none() {
            // The grid drained: refill it for the next round.
            if crate::game::recipe_book::fastcraft_book::re_place(game, display) {
                touch(|p| p.await_take = false);
            } else {
                clear();
            }
        } else {
            clear();
        }
        return;
    }

    // Wait for the armed recipe's own result before taking.
    if !expected_here {
        return;
    }
    match mode {
        Mode::Once { drop } => {
            if drop {
                throw(game, 0);
            } else {
                game.menu_click(Some(200), 0, Input::QuickMove);
            }
            clear();
        }
        Mode::Many { .. } => {
            if draining {
                throw(game, 1);
            } else {
                game.menu_click(Some(200), 0, Input::QuickMove);
            }
            touch(|p| p.await_take = true);
        }
    }
}

fn clear() {
    PENDING.with(|cell| cell.borrow_mut().take());
}

fn touch(change: impl FnOnce(&mut Pending)) {
    PENDING.with(|cell| {
        if let Some(p) = cell.borrow_mut().as_mut() {
            change(p);
            p.ttl = TTL;
        }
    });
}

/// The crafting result slot is the screen's slot 200 (menu slot 0 of the
/// plain crafting menus): a Throw drops the crafted stacks as the drop
/// key's own click would (button 0 one batch, button 1 the stack).
fn throw(game: &mut Game<'_>, button: i32) {
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
}
