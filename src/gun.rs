use bevy::prelude::*;
use cgmath::{self, Angle, Rad};
use rand::{self, Rng};

pub trait Gun: Send + Sync {
    fn new() -> Box<Self>
    where
        Self: Sized;

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        commands: Commands,
    );

    fn left_in_mag(&self) -> u16;

    fn reload(&mut self);

    fn name(&self) -> &str;

    fn reloading(&self) -> bool;
}

pub struct Shotgun {
    time_left: f32,
    mag_size: u16,
    reloading: bool,
}

pub struct Pistol {
    time_left: f32,
    mag_size: u16,
    reloading: bool,
}

impl Gun for Shotgun {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Shotgun {
            time_left: 0.2,
            mag_size: 2,
            reloading: false,
        })
    }

    fn name(&self) -> &str {
        "Shotgun"
    }

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        mut commands: Commands,
    ) {
        self.time_left -= time.delta_seconds();
        if self.time_left <= 0.0 {
            self.reloading = false;
            if mouse.just_pressed(MouseButton::Left) {
                let mut random = rand::thread_rng();
                for _index in 0..5 {
                    // #[allow(clippy::clone_on_copy)]
                    let mut transform = *player_transform;
                    transform.translation.z = 0.0;
                    transform.rotate(Quat::from_rotation_z(angle.0));

                    commands
                        .spawn()
                        .insert_bundle(SpriteBundle {
                            sprite: Sprite::new(Vec2::new(10.0, 4.0)),
                            material: materials.bullet.clone(),
                            transform,
                            ..Default::default()
                        })
                        .insert(crate::Bullet {})
                        .insert(crate::Vel(Vec2::new(
                            (angle.cos() + random.gen_range(-0.1..=0.1)) * 6.0,
                            (angle.sin() + random.gen_range(-0.1..=0.1)) * 6.0,
                        )));
                }

                self.mag_size -= 1;

                // Set time before next possible shot
                if self.mag_size == 0 {
                    self.reload();
                } else {
                    self.time_left = 0.5;
                }
            }
        }
    }

    fn reload(&mut self) {
        self.time_left = 1.0;
        self.mag_size = 2;
        self.reloading = true;
    }

    fn reloading(&self) -> bool {
        self.reloading
    }

    fn left_in_mag(&self) -> u16 {
        self.mag_size
    }
}

impl Gun for Pistol {
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Pistol {
            time_left: 0.0,
            mag_size: 7,
            reloading: false,
        })
    }

    fn name(&self) -> &str {
        "Pistol"
    }

    fn shoot(
        &mut self,
        time: Res<Time>,
        mouse: Res<Input<MouseButton>>,
        player_transform: &Transform,
        angle: Rad<f32>,
        materials: Res<crate::Materials>,
        mut commands: Commands,
    ) {
        self.time_left -= time.delta_seconds();
        if self.time_left <= 0.0 {
            self.reloading = false;
            if mouse.just_pressed(MouseButton::Left) {
                let velocity = crate::Vel(Vec2::new(angle.cos() * 6.0, angle.sin() * 6.0));

                // #[allow(clippy::clone_on_copy)]
                let mut transform = *player_transform;
                transform.translation.z = 0.0;
                transform.rotate(Quat::from_rotation_z(angle.0));

                commands
                    .spawn()
                    .insert_bundle(SpriteBundle {
                        sprite: Sprite::new(Vec2::new(10.0, 4.0)),
                        material: materials.bullet.clone(),
                        transform,
                        ..Default::default()
                    })
                    .insert(crate::Bullet {})
                    .insert(velocity);

                self.mag_size -= 1;

                // Set time before next possible shot
                if self.mag_size == 0 {
                    self.reload();
                } else {
                    self.time_left = 0.01;
                }
            }
        }
    }

    fn left_in_mag(&self) -> u16 {
        self.mag_size
    }

    fn reload(&mut self) {
        self.time_left = 0.8;
        self.mag_size = 7;
        self.reloading = true;
    }

    fn reloading(&self) -> bool {
        self.reloading
    }
}
// <a target="_blank" href="https://icons8.com/icon/35235/center-of-gravity">Center of Gravity</a> icon by <a target="_blank" href="https://icons8.com">Icons8</a>
// <a target="_blank" href="https://icons8.com/icon/YhHeUSpBoEMc/gun">Gun</a> icon by <a target="_blank" href="https://icons8.com">Icons8</a>
// <a target="_blank" href="https://icons8.com/icon/IVPN7F53YUu4/submachine-gun">Submachine Gun</a> icon by <a target="_blank" href="https://icons8.com">Icons8</a>
