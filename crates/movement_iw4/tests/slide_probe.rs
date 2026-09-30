use movement_iw4::{
    AdsFracContext, AdsIntentContext, AirMoveContext, CmdScaleWalkContext, CollisionBackend,
    FlatMantleAnimLength, GroundTraceInput, JumpLaunchContext, MeleeChargeWeaponDelays, MoveBounds,
    Pml, PmoveSingleContext, SprintContext, ViewAngleClamp, WalkMoveContext, ZeroMantleRootDelta,
    get_max_sprint_time, pmove,
};
use playerstate_iw4::{ENTITYNUM_NONE, PlayerState, UserCmd, buttons, pm_flags};
use trace_iw4::{ENTITYNUM_WORLD, HITTYPE_ENTITY, Trace};

struct FlatFloor;

fn miss(end: [f32; 3]) -> Trace {
    Trace {
        fraction: 1.0,
        normal: [0.0, 0.0, 1.0],
        endpos: end,
        walkable: 1,
        hit_type: 0,
        hit_id: 0,
        allsolid: 0,
        startsolid: 0,
        ..world_trace()
    }
}

fn world_trace() -> Trace {
    Trace {
        fraction: 1.0,
        normal: [0.0, 0.0, 1.0],
        surface_flags: 0,
        contents: 1,
        material: 0,
        hit_type: 0,
        hit_id: 0,
        model_index: 0,
        part_name: 0,
        part_group: 0,
        allsolid: 0,
        startsolid: 0,
        walkable: 1,
        endpos: [0.0; 3],
    }
}

impl CollisionBackend for FlatFloor {
    fn trace(&self, input: GroundTraceInput) -> Trace {
        let start_low = input.start[2] + input.mins[2];
        let end_low = input.end[2] + input.mins[2];
        if end_low >= start_low {
            return miss(input.end);
        }
        let fraction = if start_low <= end_low {
            0.0
        } else {
            (start_low / (start_low - end_low)).clamp(0.0, 1.0)
        };
        let mut hit = world_trace();
        hit.fraction = fraction;
        hit.normal = [0.0, 0.0, 1.0];
        hit.hit_type = HITTYPE_ENTITY;
        hit.hit_id = ENTITYNUM_WORLD;
        hit.endpos = [
            input.start[0] + (input.end[0] - input.start[0]) * fraction,
            input.start[1] + (input.end[1] - input.start[1]) * fraction,
            input.start[2] + (input.end[2] - input.start[2]) * fraction,
        ];
        hit
    }
}

fn context(old_buttons: u32) -> PmoveSingleContext {
    let air = AirMoveContext {
        player_spectate_speed_scale: 1.0,
        shellshock_gravity_scale: 1.0,
        shellshock_gravity_bias: 0.0,
    };
    PmoveSingleContext {
        walk: WalkMoveContext {
            cmd_scale: CmdScaleWalkContext {
                player_back_speed_scale: 0.7,
                player_strafe_speed_scale: 0.8,
                player_sprint_speed_scale: 1.5,
                player_last_stand_crawl_speed_scale: 0.15,
                weapon_move_speed_scale: 1.0,
                weapon_ads_move_speed_scale: 1.0,
                shellshock_affects_movement: true,
            },
            weapon_move_scale: 1.0,
            old_buttons,
            jump: JumpLaunchContext {
                jump_height: 39.0,
                dive: false,
                crouch_jump_scale: 1.0,
                jump_ladder_push_vel: 128.0,
            },
            air,
        },
        air,
        bounds: MoveBounds {
            mins: [-15.0, -15.0, 0.0],
            maxs: [15.0, 15.0, 70.0],
            tracemask: 0x0281_0011,
        },
        view_angles: ViewAngleClamp {
            pitch_up: 85.0,
            pitch_down: 85.0,
            unclamped_pitch_bit: false,
        },
        sprint: SprintContext {
            weapon_max_sprint_time: get_max_sprint_time(1.0, 4.0),
            sprint_forever: false,
            min_sprint_time_seconds: 1.0,
            sprint_delay_seconds: 0.0,
            sprint_forward_minimum: 105,
            stand_up_clear: true,
            sprint_recharge_pause_seconds: 0.0,
        },
        ads_intent: AdsIntentContext {
            ads_allowed: true,
            weapon_def_scope: false,
            sprint_hold_ads: false,
        },
        ads_frac: AdsFracContext {
            aim_down_sight: false,
            ads_in_rate: 1.0,
            ads_out_rate: 1.0,
            rechamber_while_ads: false,
            ads_fire_only: false,
        },
        melee_charge: MeleeChargeWeaponDelays {
            melee_delay_ms: 0,
            melee_charge_delay_ms: 0,
        },
        player_melee_range: movement_iw4::MELEE_CHARGE_PLAYER_MELEE_RANGE_DEFAULT,
        old_buttons,
        weapon_blocks_prone: false,
    }
}

fn sliding_ps() -> PlayerState {
    let mut ps = PlayerState::ZERO;
    ps.pm_type = 0;
    ps.pm_flags = pm_flags::SPRINTING;
    ps.origin = [0.0, 0.0, 0.0];
    ps.velocity = [260.0, 0.0, 0.0];
    ps.gravity = 800;
    ps.ground_entity_num = i32::from(ENTITYNUM_WORLD);
    ps.command_time = 0;
    ps.view_height_target = 0x3c;
    ps.view_height_current = 60.0;
    ps.sprint_start_max_length = 4000;
    ps.last_sprint_start = 0;
    ps
}

fn make_cmd(time: i32, buttons_held: u32) -> UserCmd {
    UserCmd {
        server_time: time,
        buttons: buttons_held,
        angles: [0; 3],
        forwardmove: 127,
        ..UserCmd::default()
    }
}

fn speed2d(ps: &PlayerState) -> f32 {
    libm_speed(ps.velocity[0], ps.velocity[1])
}

fn libm_speed(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}

#[test]
fn slide_starts_boosts_and_decays() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    let result = pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    let _ = result;

    assert_eq!(ps.pm_flags & pm_flags::SLIDE, pm_flags::SLIDE);
    assert_eq!(ps.pm_flags & pm_flags::CROUCH, pm_flags::CROUCH);
    assert_eq!(ps.pm_flags & pm_flags::SPRINTING, 0);
    assert_eq!(ps.pm_time, 600);
    assert_eq!(ps.view_height_target, 0x28);
    let entry = speed2d(&ps);
    assert!((295.0..339.0).contains(&entry), "entry boost {entry}");

    let mut time = 100;
    let start_x = ps.origin[0];
    let mut last_speed = entry;
    while ps.pm_time > 0 {
        let mut cmd = make_cmd(time, buttons::SPRINT | buttons::CROUCH);
        pmove(
            &mut ps,
            &mut cmd,
            context(buttons::SPRINT | buttons::CROUCH),
            &FlatFloor,
            &FlatMantleAnimLength::default(),
            &ZeroMantleRootDelta,
        );
        let now = speed2d(&ps);
        assert!(
            now <= last_speed + 0.5,
            "speed must not rise: {now} > {last_speed}"
        );
        last_speed = now;
        time += 50;
    }
    assert!(
        ps.origin[0] - start_x > 100.0,
        "slide distance {}",
        ps.origin[0] - start_x
    );
    assert!(last_speed < entry * 0.6, "decay {last_speed} from {entry}");
    assert_eq!(
        ps.pm_flags & pm_flags::SLIDE,
        0,
        "slide flag must clear on expiry"
    );
    assert_eq!(
        ps.pm_flags & pm_flags::CROUCH,
        pm_flags::CROUCH,
        "still crouched while button held"
    );
}

#[test]
fn no_slide_without_sprint() {
    let mut ps = sliding_ps();
    ps.pm_flags &= !pm_flags::SPRINTING;
    ps.velocity = [170.0, 0.0, 0.0];
    let mut cmd = make_cmd(50, buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(0),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_eq!(ps.pm_flags & pm_flags::SLIDE, 0);
    assert_eq!(ps.pm_time, 0);
}

#[test]
fn no_slide_when_crouch_already_held() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT | buttons::CROUCH),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_eq!(
        ps.pm_flags & pm_flags::SLIDE,
        0,
        "crouch already held is a plain crouch"
    );
}

#[test]
fn no_slide_when_too_slow() {
    let mut ps = sliding_ps();
    ps.velocity = [90.0, 0.0, 0.0];
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_eq!(ps.pm_flags & pm_flags::SLIDE, 0);
}

#[test]
fn attack_cancels_slide() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_ne!(ps.pm_flags & pm_flags::SLIDE, 0);
    let mut cmd = make_cmd(100, buttons::SPRINT | buttons::CROUCH | buttons::ATTACK);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT | buttons::CROUCH),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_eq!(ps.pm_flags & pm_flags::SLIDE, 0);
    assert_eq!(ps.pm_time, 0);
}

#[test]
fn slide_ends_on_ground_loss() {
    struct NoFloorHere;
    impl CollisionBackend for NoFloorHere {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            if input.start[0] > 50.0 {
                return miss(input.end);
            }
            FlatFloor.trace(input)
        }
    }

    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_ne!(ps.pm_flags & pm_flags::SLIDE, 0);
    let mut time = 100;
    while ps.pm_flags & pm_flags::SLIDE != 0 {
        let mut cmd = make_cmd(time, buttons::CROUCH);
        pmove(
            &mut ps,
            &mut cmd,
            context(buttons::CROUCH),
            &NoFloorHere,
            &FlatMantleAnimLength::default(),
            &ZeroMantleRootDelta,
        );
        time += 50;
        if time > 2000 {
            panic!("slide never ended");
        }
    }
    assert_eq!(ps.ground_entity_num, ENTITYNUM_NONE);
}

#[test]
fn releasing_crouch_mid_slide_keeps_hull_down() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    let mut cmd = make_cmd(100, buttons::SPRINT);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT | buttons::CROUCH),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_ne!(
        ps.pm_flags & pm_flags::SLIDE,
        0,
        "slide survives crouch release"
    );
    assert_eq!(ps.pm_flags & pm_flags::CROUCH, pm_flags::CROUCH);
}

#[test]
fn sprint_stays_down_during_slide_and_resumes_after() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    let mut cmd = make_cmd(100, buttons::SPRINT);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT | buttons::CROUCH),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_eq!(
        ps.pm_flags & pm_flags::SPRINTING,
        0,
        "no sprint restart mid-slide"
    );
    assert_eq!(ps.sprint_button_up_required, 0);

    let mut time = 150;
    while ps.pm_flags & pm_flags::SLIDE != 0 {
        let mut cmd = make_cmd(time, buttons::SPRINT);
        pmove(
            &mut ps,
            &mut cmd,
            context(buttons::SPRINT),
            &FlatFloor,
            &FlatMantleAnimLength::default(),
            &ZeroMantleRootDelta,
        );
        time += 50;
    }
    let mut cmd = make_cmd(time, buttons::SPRINT);
    pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_ne!(
        ps.pm_flags & pm_flags::SPRINTING,
        0,
        "sprint resumes after slide with button held"
    );
}

#[test]
fn pml_walking_is_set_during_slide() {
    let mut ps = sliding_ps();
    let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
    let result = pmove(
        &mut ps,
        &mut cmd,
        context(buttons::SPRINT),
        &FlatFloor,
        &FlatMantleAnimLength::default(),
        &ZeroMantleRootDelta,
    );
    assert_ne!(ps.pm_flags & pm_flags::SLIDE, 0);
    assert_eq!(result.pml.walking, 1);
    let _: Pml = result.pml;
}

#[test]
fn downhill_slide_keeps_more_speed_than_flat() {
    struct Slope;

    impl CollisionBackend for Slope {
        fn trace(&self, input: GroundTraceInput) -> Trace {
            let theta = (35.0_f32).to_radians();
            let normal = [theta.sin(), 0.0, theta.cos()];
            let d0 = normal[0] * input.start[0] + normal[2] * (input.start[2] + input.mins[2]);
            let d1 = normal[0] * input.end[0] + normal[2] * (input.end[2] + input.mins[2]);
            if d1 >= d0 {
                return miss(input.end);
            }
            let fraction = if d0 <= 0.0 {
                0.0
            } else {
                (d0 / (d0 - d1)).clamp(0.0, 1.0)
            };
            let mut hit = world_trace();
            hit.fraction = fraction;
            hit.normal = normal;
            hit.hit_type = HITTYPE_ENTITY;
            hit.hit_id = ENTITYNUM_WORLD;
            hit.endpos = [
                input.start[0] + (input.end[0] - input.start[0]) * fraction,
                input.start[1] + (input.end[1] - input.start[1]) * fraction,
                input.start[2] + (input.end[2] - input.start[2]) * fraction,
            ];
            hit
        }
    }

    fn run_to_end<C: CollisionBackend>(floor: &C) -> f32 {
        let mut ps = sliding_ps();
        let mut cmd = make_cmd(50, buttons::SPRINT | buttons::CROUCH);
        pmove(
            &mut ps,
            &mut cmd,
            context(buttons::SPRINT),
            floor,
            &FlatMantleAnimLength::default(),
            &ZeroMantleRootDelta,
        );
        assert_ne!(ps.pm_flags & pm_flags::SLIDE, 0);
        let mut time = 100;
        while ps.pm_time > 0 {
            let mut cmd = make_cmd(time, buttons::SPRINT | buttons::CROUCH);
            pmove(
                &mut ps,
                &mut cmd,
                context(buttons::SPRINT | buttons::CROUCH),
                floor,
                &FlatMantleAnimLength::default(),
                &ZeroMantleRootDelta,
            );
            time += 50;
        }
        assert_eq!(ps.pm_flags & pm_flags::SLIDE, 0);
        speed2d(&ps)
    }

    let flat_end = run_to_end(&FlatFloor);
    let slope_end = run_to_end(&Slope);
    assert!(
        slope_end > flat_end + 30.0,
        "downhill slide must outkeep flat: slope {slope_end} vs flat {flat_end}"
    );
}
