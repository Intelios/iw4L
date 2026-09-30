use playerstate_iw4::{ENTITYNUM_NONE, PlayerState, UserCmd, buttons, pm_flags};

use crate::{Pml, end_sprint};

pub const SLIDE_TIME_MS: i32 = 650;

const SLIDE_MIN_SPEED: f32 = 100.0;

const SLIDE_BOOST: f32 = 1.3;

const SLIDE_MAX_SPEED: f32 = 420.0;

pub fn try_start(ps: &mut PlayerState, cmd: &UserCmd, old_buttons: u32) -> bool {
    if ps.pm_type != 0
        || ps.pm_time != 0
        || (ps.pm_flags & pm_flags::SLIDE) != 0
        || (ps.pm_flags & pm_flags::SPRINTING) == 0
        || (ps.pm_flags & (pm_flags::LADDER | pm_flags::MANTLE | pm_flags::MELEE_CHARGE)) != 0
    {
        return false;
    }
    if (cmd.buttons & buttons::CROUCH) == 0
        || (old_buttons & buttons::CROUCH) != 0
        || (cmd.buttons & buttons::PRONE) != 0
    {
        return false;
    }
    if ps.ground_entity_num == ENTITYNUM_NONE {
        return false;
    }
    let speed = libm::sqrtf(ps.velocity[0] * ps.velocity[0] + ps.velocity[1] * ps.velocity[1]);
    if speed < SLIDE_MIN_SPEED {
        return false;
    }

    ps.pm_flags = (ps.pm_flags & !pm_flags::PRONE) | pm_flags::CROUCH | pm_flags::SLIDE;
    ps.pm_time = SLIDE_TIME_MS;
    let boosted = (speed * SLIDE_BOOST).min(SLIDE_MAX_SPEED);
    let scale = boosted / speed;
    ps.velocity[0] *= scale;
    ps.velocity[1] *= scale;
    end_sprint(ps, cmd);
    ps.sprint_button_up_required = 0;
    true
}

pub fn cancel(ps: &mut PlayerState) {
    if (ps.pm_flags & pm_flags::SLIDE) == 0 {
        return;
    }
    ps.pm_flags &= !pm_flags::SLIDE;
    ps.pm_time = 0;
}

pub fn update(ps: &mut PlayerState, pml: &Pml, cmd: &UserCmd) {
    if (ps.pm_flags & pm_flags::SLIDE) == 0 {
        return;
    }
    if pml.walking == 0
        || (ps.pm_flags & (pm_flags::LADDER | pm_flags::MANTLE)) != 0
        || (ps.pm_flags & pm_flags::ADS_INTENT) != 0
        || (cmd.buttons & buttons::ATTACK) != 0
    {
        cancel(ps);
    }
}

#[must_use]
pub fn is_sliding(ps: &PlayerState) -> bool {
    (ps.pm_flags & pm_flags::SLIDE) != 0
}
