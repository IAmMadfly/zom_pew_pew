use bevy::{math::f32, prelude::*};
use cgmath::{Angle, Rad};
use gun::Gun;
use rand::{self, Rng};

mod gun;

static MOVE_SPEED: f32 = 1.8;
static ZOM_SPEED: f32 = 2.2;
static ZOM_SIZE: f32 = 10.0;
static STRONG_ZOM_SIZE: f32 = 15.0;
static STRONG_ZOM_SPEED: f32 = 1.6;

type PeopleBorrow<'a> = (&'a Player, &'a Transform);
type ZomBorrowTransMut<'a> = (&'a Zom, &'a mut Transform);

struct Player {
    angle: Rad<f32>,
    gun: Option<Box<dyn gun::Gun>>,
}

enum ZomType {
    Default,
    Strong,
}

impl Default for ZomType {
    fn default() -> Self {
        ZomType::Default
    }
}

#[derive(Default)]
struct Zom {
    zom_type: ZomType,
}

struct Vel(Vec2);

trait Velocity {
    fn magnitude(&self) -> f32;

    fn unit_vec(&self) -> (f32, f32);

    fn between_transforms(start: &Self, end: &Self) -> Self;

    fn get_angle_to(&self, other: &Self) -> Rad<f32>;
}

impl Velocity for Vec2 {
    fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    fn unit_vec(&self) -> (f32, f32) {
        // Get Magnitude
        let mag = self.magnitude();

        // Return unit
        (self.x / mag, self.y / mag)
    }

    fn between_transforms(start: &Vec2, end: &Vec2) -> Vec2 {
        Vec2::new(end.x - start.x, end.y - start.y)
    }

    fn get_angle_to(&self, other: &Vec2) -> Rad<f32> {
        let mut angle_calc = Rad::atan((other.y - self.y) / (other.x - self.x));

        if other.x < self.x {
            angle_calc += Rad(std::f32::consts::PI);
        }

        angle_calc
    }
}

struct Bullet {}

pub struct Materials {
    bullet: Handle<ColorMaterial>,
    zom: Handle<ColorMaterial>,
    strong_zom: Handle<ColorMaterial>,
}

trait ClampMax {
    fn clamp_max_length(&mut self, max: f32);
}

impl ClampMax for Vec2 {
    fn clamp_max_length(&mut self, max: f32) {
        let curr_length = (self.x.powi(2) + self.y.powi(2)).sqrt();

        if curr_length > max {
            let ratio = max / curr_length;

            self.x *= ratio;
            self.y *= ratio;
        }
    }
}

fn main() {
    let mut app = App::build();

    app.add_plugins(DefaultPlugins);

    app.add_startup_system(load_materials.system());

    app.add_startup_system(load_player.system());
    app.add_startup_system(load_camera.system());
    app.add_startup_system(load_text.system());

    app.add_system(face_mouse.system());
    app.add_system(shoot_bullet.system());
    app.add_system(move_elements.system());
    app.add_system(move_player.system());
    app.add_system(player_input.system());
    app.add_system(spawn_zom.system());
    app.add_system(move_zom.system());
    app.add_system(zom_bullet_collision.system());
    app.add_system(despawn_bullet.system());
    app.add_system(change_player_sprite.system());
    app.add_system(update_text.system());

    app.run();
}

fn update_text(mut text_query: Query<&mut Text>, player_query: Query<&Player>) {
    if let (Ok(player), Ok(mut text)) = (player_query.single(), text_query.single_mut()) {
        if let Some(gun) = &player.gun {
            text.sections[0].value = match gun.reloading() {
                true => "RELOADING!".to_string(),
                false => format!("{} Rounds: {}", gun.name(), gun.left_in_mag()),
            };
        } else {
            text.sections[0].value = "No gun".to_string();
        }
    }
}

fn face_mouse(mut player_query: Query<(&mut Player, &mut Transform)>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let cursor_loc_opt = window.cursor_position();
    if let (Ok((mut player, transform)), Some(cursor_location)) =
        (player_query.single_mut(), cursor_loc_opt)
    {
        let cursor_location_corrected = Vec2::new(
            cursor_location.x - (window.width() / 2.0),
            cursor_location.y - (window.height() / 2.0),
        );

        let player_location = transform.translation.truncate();

        let angle_calc = player_location.get_angle_to(&cursor_location_corrected);

        // transform.rotation = Quat::from_rotation_z(angle_calc.0);
        player.angle = angle_calc;
    }
}

fn move_player(input: Res<Input<KeyCode>>, mut player_query: Query<(&Player, &mut Transform)>) {
    if let Ok((_player, mut trans)) = player_query.single_mut() {
        let mut translation = Vec2::new(0.0, 0.0);

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

        translation.clamp_max_length(MOVE_SPEED);

        trans.translation += Vec3::new(translation.x, translation.y, 0.0);
    }
}

fn player_input(input: Res<Input<KeyCode>>, mut player_query: Query<&mut Player>) {
    if let Ok(mut player) = player_query.single_mut() {
        if input.pressed(KeyCode::R) {
            if let Some(gun) = &mut player.gun {
                gun.reload();
            }
        }
    }
}

fn move_zom(mut player_query: QuerySet<(Query<PeopleBorrow>, Query<ZomBorrowTransMut>)>) {
    let mut _player_transform = Transform::from_xyz(0.0, 0.0, 0.0);
    if let Ok((_player, player_trans)) = player_query.q0().single() {
        _player_transform = *player_trans;
    } else {
        return;
    }

    for (zom, mut zom_trans) in player_query.q1_mut().iter_mut() {
        let unit_vec = Velocity::between_transforms(
            &zom_trans.translation.truncate(),
            &_player_transform.translation.truncate(),
        )
        .unit_vec();

        let speed = match zom.zom_type {
            ZomType::Default => ZOM_SPEED,
            ZomType::Strong => STRONG_ZOM_SPEED,
        };

        zom_trans.translation.x += unit_vec.0 * speed;
        zom_trans.translation.y += unit_vec.1 * speed;
        zom_trans.rotation = Quat::from_rotation_z(
            zom_trans
                .translation
                .truncate()
                .get_angle_to(&_player_transform.translation.truncate())
                .0,
        );
    }
}

fn zom_bullet_collision(
    bullet_query: Query<(&Bullet, &Transform, Entity)>,
    zom_query: Query<(&Zom, &Transform, Entity)>,
    mut commands: Commands,
) {
    for (zom, zom_trans, zom_entity) in zom_query.iter() {
        for (_bullet, bullet_trans, bullet_entity) in bullet_query.iter() {
            let dist = Velocity::between_transforms(
                &zom_trans.translation.truncate(),
                &bullet_trans.translation.truncate(),
            )
            .magnitude();

            match zom.zom_type {
                ZomType::Default => {
                    if dist < ZOM_SIZE {
                        commands.entity(zom_entity).despawn();
                        commands.entity(bullet_entity).despawn();
                    }
                }
                ZomType::Strong => {
                    if dist < STRONG_ZOM_SIZE {
                        commands.entity(zom_entity).despawn();
                        commands.entity(bullet_entity).despawn();
                    }
                }
            };
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

        match random.gen_range(0..10) {
            0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(ZOM_SIZE, ZOM_SIZE)),
                        material: materials.zom.clone(),
                        transform: Transform::from_xyz(translation.x, translation.y, translation.z),
                        ..Default::default()
                    })
                    .insert(Zom::default());
            }
            7 | 8 | 9 => {
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(STRONG_ZOM_SIZE, STRONG_ZOM_SIZE)),
                        material: materials.strong_zom.clone(),
                        transform: Transform::from_xyz(translation.x, translation.y, translation.z),
                        ..Default::default()
                    })
                    .insert(Zom {
                        zom_type: ZomType::Strong,
                    });
            }
            _ => {}
        }
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

fn move_elements(mut vel_query: Query<(&Vel, &mut Transform)>) {
    for (vel, mut trans) in vel_query.iter_mut() {
        trans.translation.x += vel.0.x;
        trans.translation.y += vel.0.y;
    }
}

struct SpriteAnimationCapture {
    x_diff: f32,
    y_diff: f32,
    start_point: [u32; 2],
}

// fn change_zom_sprite(
//     mut zom_query: Query<(&Zom, &SpriteAnimationCapture, &Handle<Mesh>)>,
//     mut mesh_access: ResMut<Assets<Mesh>>,
// ) {
//     if let Ok((zom, sprite_info, mut mesh)) = zom_query.single_mut() {
//         let angle = (zom.angle.0 * (180.0 / std::f32::consts::PI)) as i32;
//     }
// }

fn change_player_sprite(
    mut player_query: Query<(&Player, &SpriteAnimationCapture, &Handle<Mesh>)>,
    mut mesh_access: ResMut<Assets<Mesh>>,
) {
    if let Ok((player, sprite_info, mesh)) = player_query.single_mut() {
        let angle = (player.angle.0 * (180.0 / std::f32::consts::PI)) as i32;
        let sprite_mesh = match angle {
            (-45..=45) => [
                (sprite_info.start_point[0]) as f32,
                (2 + sprite_info.start_point[1]) as f32,
            ],
            (46..=135) => [
                (sprite_info.start_point[0]) as f32,
                (3 + sprite_info.start_point[1]) as f32,
            ],
            (136..=225) => [
                (sprite_info.start_point[0]) as f32,
                (1 + sprite_info.start_point[1]) as f32,
            ],
            (226..=270) => [
                (sprite_info.start_point[0]) as f32,
                (sprite_info.start_point[1]) as f32,
            ],
            (-90..=-44) => [
                (sprite_info.start_point[0]) as f32,
                (sprite_info.start_point[1]) as f32,
            ],
            _ => {
                println!("Angle at: {}", player.angle.0);
                [
                    (sprite_info.start_point[0]) as f32,
                    (2 + sprite_info.start_point[1]) as f32,
                ]
            }
        };

        let x_diff = &sprite_info.x_diff;
        let y_diff = &sprite_info.y_diff;

        let uv_vec = vec![
            [x_diff * sprite_mesh[0], y_diff * sprite_mesh[1] + y_diff],
            [x_diff * sprite_mesh[0], y_diff * sprite_mesh[1]],
            [x_diff * sprite_mesh[0] + x_diff, y_diff * sprite_mesh[1]],
            [
                x_diff * sprite_mesh[0] + x_diff,
                y_diff * sprite_mesh[1] + y_diff,
            ],
        ];

        let mesh = mesh_access
            .get_mut(mesh)
            .expect("Failed to get mesh handle for player!");

        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv_vec);
    }
}

// SETUP FUNCTIONS
// ----------------------------------
fn load_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn load_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text {
            sections: vec![TextSection {
                value: "Rounds".to_string(),
                style: TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
            ..Default::default()
        },
        ..Default::default()
    });
}

fn load_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(Materials {
        bullet: materials.add(Color::GRAY.into()),
        zom: materials.add(Color::RED.into()),
        strong_zom: materials.add(Color::CYAN.into()),
    });
}

fn load_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let texture_handle = asset_server.load("images/people/players.png");

    let mut mesh = Mesh::from(shape::Quad::new(Vec2::new(1.0, 1.0)));
    let x_diff = 1. / 12.;
    let y_diff = 1. / 8.;
    let uv_vec = vec![
        [0.0 + (x_diff * 1.), 0.5 + y_diff],
        [0.0 + (x_diff * 1.), 0.5],
        [x_diff + (x_diff * 1.), 0.5],
        [x_diff + (x_diff * 1.), 0.5 + y_diff],
    ];
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv_vec);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(30.0, 50.0)),
            material: materials.add(texture_handle.into()),
            // material: materials.add(Color::ORANGE_RED.into()),
            mesh: meshes.add(mesh),
            transform: Transform::from_xyz(0.0, 0.0, 0.1),
            ..Default::default()
        })
        .insert(SpriteAnimationCapture {
            x_diff,
            y_diff,
            start_point: [1, 4],
        })
        .insert(Player {
            angle: Rad(0.0),
            gun: Some(gun::Shotgun::new()),
        });
}
// -----------------------------------
