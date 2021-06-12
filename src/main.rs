use bevy::{math::f32, prelude::*};
use cgmath::{Angle, Rad};
use gun::Gun;
use rand::{self, Rng};

mod gun;

static MOVE_SPEED: f32 = 0.6;
static ZOM_SIZE: f32 = 10.0;

struct Player {
    angle: Rad<f32>,
    gun: Option<Box<dyn gun::Gun>>,
}

struct Zom {}

struct Velocity {
    x: f32,
    y: f32,
}

impl Velocity {
    fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn unit_vec(&self) -> (f32, f32) {
        // Get Magnitude
        let mag = self.magnitude();

        // Return unit
        (self.x / mag, self.y / mag)
    }

    fn between_transforms(start: &Transform, end: &Transform) -> Velocity {
        Velocity {
            x: end.translation.x - start.translation.x,
            y: end.translation.y - start.translation.y,
        }
    }
}

struct Bullet {}

pub struct Materials {
    bullet: Handle<ColorMaterial>,
    zom: Handle<ColorMaterial>,
}

fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins);

    app.add_startup_system(load_materials.system());

    app.add_startup_system(load_player.system());
    app.add_startup_system(load_camera.system());

    app.add_system(face_mouse.system());
    app.add_system(shoot_bullet.system());
    app.add_system(move_elements.system());
    app.add_system(move_player.system());
    app.add_system(spawn_zom.system());
    app.add_system(move_zom.system());
    app.add_system(zom_bullet_collision.system());
    app.add_system(despawn_bullet.system());

    app.run();
}

fn face_mouse(mut player_query: Query<(&mut Player, &mut Transform)>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let cursor_loc_opt = window.cursor_position();
    if let (Ok((mut player, mut transform)), Some(cursor_location)) =
        (player_query.single_mut(), cursor_loc_opt)
    {
        let cursor_location_corrected = Vec2::new(
            cursor_location.x - (window.width() / 2.0),
            cursor_location.y - (window.height() / 2.0),
        );

        let player_location = transform.translation.clone().truncate();

        let mut angle_calc = Rad::atan(
            (cursor_location_corrected.y - player_location.y)
                / (cursor_location_corrected.x - player_location.x),
        );

        if cursor_location_corrected.x < player_location.x {
            angle_calc += Rad(std::f32::consts::PI);
        }

        transform.rotation = Quat::from_rotation_z(angle_calc.0);
        player.angle = angle_calc;
    }
}

fn move_player(input: Res<Input<KeyCode>>, mut player_query: Query<(&Player, &mut Transform)>) {
    if let Ok((_player, mut trans)) = player_query.single_mut() {
        let mut translation = Vec3::new(0.0, 0.0, 0.0);

        if input.pressed(KeyCode::W) {
            translation.y += MOVE_SPEED;
        }
        if input.pressed(KeyCode::S) {
            translation.y -= MOVE_SPEED;
        }
        if input.pressed(KeyCode::A) {
            translation.x -= MOVE_SPEED;
        }
        if input.pressed(KeyCode::D) {
            translation.x += MOVE_SPEED;
        }

        trans.translation += translation;
    }
}

fn move_zom(
    mut player_query: QuerySet<(Query<(&Player, &Transform)>, Query<(&Zom, &mut Transform)>)>,
) {
    let mut _player_transform = Transform::from_xyz(0.0, 0.0, 0.0);
    if let Ok((_player, player_trans)) = player_query.q0().single() {
        _player_transform = player_trans.clone();
    } else {
        return;
    }

    for (_zom, mut zom_trans) in player_query.q1_mut().iter_mut() {
        let unit_vec = Velocity::between_transforms(&zom_trans, &_player_transform).unit_vec();

        zom_trans.translation.x += unit_vec.0 * 1.2;
        zom_trans.translation.y += unit_vec.1 * 1.2;
        // zom_trans.rotation = Quat::from_rotation_z(
        //     zom_trans
        //         .translation
        //         .clone()
        //         .truncate()
        //         .angle_between(_player_transform.translation.clone().truncate()),
        // );
    }
}

fn zom_bullet_collision(
    bullet_query: Query<(&Bullet, &Transform, Entity)>,
    zom_query: Query<(&Zom, &Transform, Entity)>,
    mut commands: Commands,
) {
    for (_zom, zom_trans, zom_entity) in zom_query.iter() {
        for (_bullet, bullet_trans, bullet_entity) in bullet_query.iter() {
            let dist = Velocity::between_transforms(zom_trans, bullet_trans).magnitude();

            if dist < ZOM_SIZE {
                commands.entity(zom_entity).despawn();
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}

fn spawn_zom(mut commands: Commands, materials: Res<Materials>, windows: Res<Windows>) {
    let mut random = rand::thread_rng();
    if random.gen_bool(0.01) {
        let mut translation = Vec3::new(0.0, 0.0, 0.0);
        let window = windows.get_primary().unwrap();
        let window_size = (window.width(), window.height());

        // Choose which edge to spawn on
        match random.gen_range(1..=4) {
            // Left side
            1 => {
                translation.x = -window_size.0 / 2.0;
                translation.y = random.gen_range((-window_size.1 / 2.0)..(window_size.1 / 2.0));
            }
            // Top side
            2 => {
                translation.y = window_size.1 / 2.0;
                translation.x = random.gen_range((-window_size.0 / 2.0)..(window_size.0 / 2.0));
            }
            // Right side
            3 => {
                translation.x = window_size.0 / 2.0;
                translation.y = random.gen_range((-window_size.1 / 2.0)..(window_size.1 / 2.0));
            }
            // Bottom side
            4 => {
                translation.y = -window_size.1 / 2.0;
                translation.x = random.gen_range((-window_size.0 / 2.0)..(window_size.0 / 2.0));
            }
            _ => {
                panic!("What the fek? how did this happen?");
            }
        }

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite::new(Vec2::new(ZOM_SIZE, ZOM_SIZE)),
                material: materials.zom.clone(),
                transform: Transform::from_xyz(translation.x, translation.y, translation.z),
                ..Default::default()
            })
            .insert(Zom {});
    }
}

fn shoot_bullet(
    commands: Commands,
    mouse: Res<Input<MouseButton>>,
    materials: Res<Materials>,
    mut player_query: Query<(&mut Player, &Transform)>,
    time: Res<Time>,
) {
    if let Ok((mut player, trans)) = player_query.single_mut() {
        let angle = player.angle;
        if let Some(gun) = player.gun.as_mut() {
            gun.shoot(time, mouse, trans, angle, materials, commands);
        }
    }
}

fn despawn_bullet(
    mut commands: Commands,
    bullet_query: Query<(&Bullet, &Transform, Entity)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = (window.width(), window.height());

    for (_bullet, trans, entity) in bullet_query.iter() {
        if trans.translation.x.abs() > (window_size.0 / 2.0)
            || trans.translation.y.abs() > (window_size.1 / 2.0)
        {
            commands.entity(entity).despawn();
        }
    }
}

fn move_elements(mut vel_query: Query<(&Velocity, &mut Transform)>) {
    for (vel, mut trans) in vel_query.iter_mut() {
        trans.translation.x += vel.x;
        trans.translation.y += vel.y;
    }
}

// SETUP FUNCTIONS
// ----------------------------------
fn load_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn load_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(Materials {
        bullet: materials.add(Color::GRAY.into()),
        zom: materials.add(Color::RED.into()),
    });
}

fn load_player(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(15.0, 10.0)),
            material: materials.add(Color::ORANGE_RED.into()),
            transform: Transform::from_xyz(0.001, 0.001, 0.1),
            ..Default::default()
        })
        .insert(Player {
            angle: Rad(0.0),
            gun: Some(gun::Shotgun::new()),
        });
}
// -----------------------------------
